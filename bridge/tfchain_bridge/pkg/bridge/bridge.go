package bridge

import (
	"context"
	"fmt"
	"strconv"
	"time"

	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
	"github.com/pkg/errors"
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
	hProtocol "github.com/stellar/go/protocols/horizon"
	"github.com/threefoldtech/tfchain/bridge/tfchain_bridge/pkg"
	"github.com/threefoldtech/tfchain/bridge/tfchain_bridge/pkg/stellar"
	subpkg "github.com/threefoldtech/tfchain/bridge/tfchain_bridge/pkg/substrate"
)

const (
	BridgeNetwork  = "stellar"
	MinimumBalance = 0
)

// Bridge is a high lvl structure which listens on contract events and bridge-related
// stellar transactions, and handles them
type Bridge struct {
	wallet           *stellar.StellarWallet
	subClient        *subpkg.SubstrateClient
	blockPersistency *pkg.ChainPersistency
	config           *pkg.BridgeConfig
	depositFee       int64
	idempotency      *pkg.IdempotencyStore
}

func NewBridge(ctx context.Context, cfg pkg.BridgeConfig) (*Bridge, string, error) {
	subClient, err := subpkg.NewSubstrateClient(cfg.TfchainURLs, cfg.TfchainSeed)
	if err != nil {
		return nil, "", err
	}

	blockPersistency, err := pkg.InitPersist(cfg.PersistencyFile)
	if err != nil {
		return nil, "", err
	}

	wallet, err := stellar.NewStellarWallet(ctx, &cfg.StellarConfig)
	if err != nil {
		return nil, "", err
	}

	if cfg.RescanBridgeAccount {
		// saving the cursor to 0 will trigger the bridge stellar account
		// to scan for every transaction ever made on the bridge account
		// and mint accordingly
		err = blockPersistency.SaveStellarCursor("0")
		if err != nil {
			return nil, "", err
		}
		err = blockPersistency.SaveHeight(0)
		if err != nil {
			return nil, "", err
		}
	}

	// fetch the configured depositfee
	depositFee, err := subClient.GetDepositFee()
	if err != nil {
		return nil, "", err
	}

	// Crash-safe idempotency store, kept alongside the block persistency file.
	// Records the PROCESSING/COMPLETED state of withdraws and refunds so that a
	// crash between submitting a Stellar payment and confirming it on TFChain is
	// recovered without double-paying or double-confirming.
	idempotency, err := pkg.NewIdempotencyStore(cfg.PersistencyFile + ".idem.db")
	if err != nil {
		return nil, "", errors.Wrap(err, "failed to open idempotency store")
	}

	// The idempotency store is chain-scoped: withdraw keys are TFChain burn tx ids,
	// which restart from a low number after a chain reset and would otherwise collide
	// with stale COMPLETED entries, causing new withdraws to be wrongly skipped. The
	// rescan flag marks a fresh start (it also zeroes the Stellar cursor above), so
	// clear the store here too.
	if cfg.RescanBridgeAccount {
		if err := idempotency.Reset(); err != nil {
			return nil, "", errors.Wrap(err, "failed to reset idempotency store")
		}
	}

	bridge := &Bridge{
		subClient:        subClient,
		blockPersistency: blockPersistency,
		wallet:           wallet,
		config:           &cfg,
		depositFee:       depositFee,
		idempotency:      idempotency,
	}
	// stat deposit fee?
	return bridge, wallet.GetKeypair().Address(), nil
}

func (bridge *Bridge) preCheckBalance(ctx context.Context) error {
	balance, err := bridge.wallet.StatBridgeAccount()

	if err != nil {
		return errors.Wrap(err, "can't retrieve the wallet balance at the moment")
	}
	s, err := strconv.ParseFloat(balance, 64)
	if err != nil {
		return errors.Wrap(err, "can't parse the wallet balance")
	}

	if s < MinimumBalance {
		return errors.Errorf("wallet balance insufficient: %s", balance)
	}
	return nil
}

func (bridge *Bridge) Start(ctx context.Context) error {
	// Close the idempotency store when Start returns.
	defer func() {
		if err := bridge.idempotency.Close(); err != nil {
			log.Warn().Err(err).Msg("failed to close idempotency store")
		}
	}()

	// pre-check wallet balance
	if err := bridge.preCheckBalance(ctx); err != nil {
		return err
	}

	// Crash recovery: reconcile any transactions left in PROCESSING state by a
	// previous run before we start consuming new events. Non-fatal — unreconciled
	// transactions are retried when their Ready event fires again.
	if err := bridge.reconcilePendingTransactions(ctx); err != nil {
		return errors.Wrap(err, "startup reconciliation failed")
	}

	log.Info().
		Str("event_action", "bridge_started").
		Str("event_kind", "event").
		Str("category", "availability").
		Dict("metadata", zerolog.Dict().
			Bool("rescan_flag", bridge.config.RescanBridgeAccount).
			Int64("deposit_fee", bridge.depositFee)).
		Msg("the bridge instance has started")
	height, err := bridge.blockPersistency.GetHeight()
	if err != nil {
		return errors.Wrap(err, "an error occurred while reading block height from persistency")
	}

	log.Debug().
		Msg("The Stellar subscription is starting")
	stellarSub := make(chan stellar.MintEventSubscription)
	go func() {
		defer close(stellarSub)
		if err = bridge.wallet.StreamBridgeStellarTransactions(ctx, stellarSub, height.StellarCursor); err != nil {
			log.Fatal().
				Err(err).
				Str("event_action", "bridge_unexpectedly_exited").
				Str("event_kind", "error").
				Str("category", "availability").
				Msg("the bridge instance has exited unexpectedly")
		}
	}()

	log.Debug().
		Msg("The TFChain subscription is starting")
	tfchainSub := make(chan subpkg.EventSubscription)
	go func() {
		defer close(tfchainSub)
		if err := bridge.subClient.SubscribeTfchainBridgeEvents(ctx, tfchainSub); err != nil {
			log.Fatal().
				Err(err).
				Str("event_action", "bridge_unexpectedly_exited").
				Str("event_kind", "error").
				Str("category", "availability").
				Msg("the bridge instance has exited unexpectedly")
		}
	}()
	afterMinute := time.After(60 * time.Second)

	for {
		select {
		case data := <-tfchainSub:
			if data.Err != nil {
				return errors.Wrap(data.Err, "failed to get tfchain events")
			}
			// Ready events are processed before Created/Expired events: a Ready
			// event submits a payment to Stellar whose signatures are time-sensitive
			// (they expire), so it must not wait behind proposal/expiry handling.
			for _, withdawReadyEvent := range data.Events.WithdrawReadyEvents {
				err := bridge.handleWithdrawReady(ctx, withdawReadyEvent)
				if err != nil {
					if errors.Is(err, pkg.ErrTransactionAlreadyBurned) {
						continue
					}
					return errors.Wrap(err, "an error occurred while handling WithdrawReadyEvents")
				}
			}
			for _, refundReadyEvent := range data.Events.RefundReadyEvents {
				err := bridge.handleRefundReady(ctx, refundReadyEvent)
				if err != nil {
					if errors.Is(err, pkg.ErrTransactionAlreadyRefunded) {
						continue
					}
					return errors.Wrap(err, "an error occurred while handling RefundReadyEvents")
				}
			}
			for _, withdrawCreatedEvent := range data.Events.WithdrawCreatedEvents {
				err := bridge.handleWithdrawCreated(ctx, withdrawCreatedEvent)
				if err != nil {
					// If the TX is already withdrawn or refunded (minted on tfchain) skip
					if errors.Is(err, pkg.ErrTransactionAlreadyBurned) || errors.Is(err, pkg.ErrTransactionAlreadyMinted) {
						continue
					}
					return errors.Wrap(err, "an error occurred while handling WithdrawCreatedEvents")
				}
			}
			for _, withdrawExpiredEvent := range data.Events.WithdrawExpiredEvents {
				err := bridge.handleWithdrawExpired(ctx, withdrawExpiredEvent)
				if err != nil {
					return errors.Wrap(err, "an error occurred while handling WithdrawExpiredEvents")
				}
			}
			for _, refundExpiredEvent := range data.Events.RefundExpiredEvents {
				err := bridge.handleRefundExpired(ctx, refundExpiredEvent)
				if err != nil {
					return errors.Wrap(err, "an error occurred while handling RefundExpiredEvents")
				}
			}
		case data := <-stellarSub:
			if data.Err != nil {
				return errors.Wrap(data.Err, "failed to get stellar payments")
			}

			for _, mEvent := range data.Events {
				err := bridge.mint(ctx, mEvent.Senders, mEvent.Tx)
				if err != nil {
					if errors.Is(err, pkg.ErrTransactionAlreadyMinted) {
						continue
					}
					return errors.Wrap(err, "an error occurred while processing the payment received") // mint could be initiated already but there is a problem saving the cursor
				}
			}
			time.Sleep(0)
		case <-afterMinute:
			balance, err := bridge.wallet.StatBridgeAccount()
			if err != nil {
				log.Logger.Warn().Err(err).Msgf("Can't retrieve the wallet balance at the moment")
			}
			log.Logger.Info().
				Str("event_action", "wallet_balance").
				Str("event_kind", "metric").
				Str("category", "vault").
				Dict("metadata", zerolog.Dict().
					Str("tft", balance)).
				Msgf("TFT Balance is %s", balance)
			afterMinute = time.After(60 * time.Second)
		case <-ctx.Done():
			return ctx.Err()
		}
		time.Sleep(1 * time.Second)
	}
}

// reconcilePendingTransactions runs once at startup to recover transactions that a
// previous run left in PROCESSING state — i.e. the Stellar payment may or may not
// have been submitted before the bridge stopped. For each pending withdraw/refund it
// looks for a matching outgoing Stellar transaction (by memo, falling back to the
// sequence number for pre-memo submissions). If found, the funds already left the
// bridge, so it only completes the TFChain confirmation and marks the entry COMPLETED.
// If not found, the entry is left PROCESSING and will be retried when its Ready event
// fires again. All failures here are non-fatal: a transient Horizon/RPC problem must
// not stop the bridge from starting.
func (bridge *Bridge) reconcilePendingTransactions(ctx context.Context) error {
	pendingWithdraws, err := bridge.idempotency.GetPendingWithdraws()
	if err != nil {
		return errors.Wrap(err, "failed to get pending withdraws")
	}
	pendingRefunds, err := bridge.idempotency.GetPendingRefunds()
	if err != nil {
		return errors.Wrap(err, "failed to get pending refunds")
	}

	if len(pendingWithdraws) == 0 && len(pendingRefunds) == 0 {
		return nil
	}

	log.Info().
		Int("pending_withdraws", len(pendingWithdraws)).
		Int("pending_refunds", len(pendingRefunds)).
		Msg("reconciling pending transactions from previous run")

	// Fetch outgoing transactions once and reuse the page for all lookups, avoiding
	// one Horizon HTTP call per pending transaction.
	outgoingPage, err := bridge.wallet.FetchOutgoingTransactionsPage(ctx)
	if err != nil {
		// Non-fatal: pending txs are retried when their next Ready event fires.
		log.Warn().Err(err).Msg("failed to fetch Horizon transactions for reconciliation, pending transactions will retry on next event")
		outgoingPage = hProtocol.TransactionsPage{}
	}

	for _, txID := range pendingWithdraws {
		// Recover by the text memo (burn tx id), falling back to the account sequence
		// number for payments submitted by a pre-memo bridge version.
		stellarTx := bridge.wallet.FindPaymentByMemoInPage(outgoingPage, fmt.Sprint(txID))
		if stellarTx == nil {
			burnTx, err := bridge.subClient.GetBurnTransaction(types.U64(txID))
			if err != nil {
				log.Warn().Err(err).Uint64("tx_id", txID).Msg("failed to get burn tx for sequence lookup during reconciliation")
			} else {
				stellarTx = bridge.wallet.FindPaymentBySequenceInPage(outgoingPage, int64(burnTx.SequenceNumber))
			}
		}

		if stellarTx == nil {
			log.Info().Uint64("tx_id", txID).Msg("reconcile: no Stellar tx found by memo or sequence, will retry on next event")
			continue
		}

		log.Info().Uint64("tx_id", txID).Msg("reconcile: found existing Stellar payment, completing TFChain confirmation")
		if err := bridge.subClient.RetrySetWithdrawExecuted(ctx, txID); err != nil {
			log.Warn().Err(err).Uint64("tx_id", txID).Msg("failed to set withdraw executed during reconciliation")
			continue
		}
		if err := bridge.idempotency.MarkWithdrawCompleted(txID); err != nil {
			log.Warn().Err(err).Uint64("tx_id", txID).Msg("failed to mark withdraw completed during reconciliation")
		}
	}

	for _, txHash := range pendingRefunds {
		stellarTx := bridge.wallet.FindRefundByReturnHashInPage(outgoingPage, txHash)
		if stellarTx == nil {
			refundTx, err := bridge.subClient.GetRefundTransaction(txHash)
			if err != nil {
				log.Warn().Err(err).Str("tx_hash", txHash).Msg("failed to get refund tx for sequence lookup during reconciliation")
			} else {
				stellarTx = bridge.wallet.FindPaymentBySequenceInPage(outgoingPage, int64(refundTx.SequenceNumber))
			}
		}

		if stellarTx == nil {
			log.Info().Str("tx_hash", txHash).Msg("reconcile: no Stellar refund found by return hash or sequence, will retry on next event")
			continue
		}

		log.Info().Str("tx_hash", txHash).Msg("reconcile: found existing Stellar refund, completing TFChain confirmation")
		if err := bridge.subClient.RetrySetRefundTransactionExecutedTx(ctx, txHash); err != nil {
			log.Warn().Err(err).Str("tx_hash", txHash).Msg("failed to set refund executed during reconciliation")
			continue
		}
		if err := bridge.idempotency.MarkRefundCompleted(txHash); err != nil {
			log.Warn().Err(err).Str("tx_hash", txHash).Msg("failed to mark refund completed during reconciliation")
		}
	}

	log.Info().Msg("reconciliation complete")
	return nil
}
