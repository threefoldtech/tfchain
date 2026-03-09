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

	// maxExecutionBatchSize caps the number of Ready events processed per cycle.
	// This limits the number of PROCESSING entries in BoltDB at any time, ensuring
	// crash recovery can find all Stellar payments within the 200-tx outgoing page.
	// 100 burns + 100 refunds = 200 max, matching the Stellar query limit.
	maxExecutionBatchSize = 100
)

// Bridge is a high lvl structure which listens on contract events and bridge-related
// stellar transactions, and handles them
type Bridge struct {
	wallet           *stellar.StellarWallet
	subClient        *subpkg.SubstrateClient
	blockPersistency *pkg.ChainPersistency
	idempotency      *pkg.IdempotencyStore
	config           *pkg.BridgeConfig
	depositFee       int64
}

func NewBridge(ctx context.Context, cfg pkg.BridgeConfig) (*Bridge, string, error) {
	subClient, err := subpkg.NewSubstrateClient(cfg.TfchainURL, cfg.TfchainSeed)
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

	// Initialize idempotency store alongside the persistency file
	idempotencyPath := cfg.PersistencyFile + ".idem.db"
	idempotency, err := pkg.NewIdempotencyStore(idempotencyPath)
	if err != nil {
		return nil, "", err
	}

	bridge := &Bridge{
		subClient:        subClient,
		blockPersistency: blockPersistency,
		idempotency:      idempotency,
		wallet:           wallet,
		config:           &cfg,
		depositFee:       depositFee,
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
	// pre-check wallet balance
	if err := bridge.preCheckBalance(ctx); err != nil {
		return err
	}

	log.Info().
		Str("event_action", "bridge_started").
		Str("event_kind", "event").
		Str("category", "availability").
		Dict("metadata", zerolog.Dict().
			Bool("rescan_flag", bridge.config.RescanBridgeAccount).
			Int64("deposit_fee", bridge.depositFee)).
		Msg("the bridge instance has started")

	// Close idempotency store when Start returns
	defer bridge.idempotency.Close()

	// Reconcile any PROCESSING transactions from a previous run that may have
	// crashed between Stellar submit and TFChain confirmation
	if err := bridge.reconcilePendingTransactions(ctx); err != nil {
		return errors.Wrap(err, "startup reconciliation failed")
	}

	height, err := bridge.blockPersistency.GetHeight()
	if err != nil {
		return errors.Wrap(err, "an error occurred while reading block height from persistency")
	}

	log.Debug().Msg("The Stellar subscription is starting")
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

			// Process Ready events — submit all to Stellar first, then batch TFChain confirmations.
			// Cap at maxExecutionBatchSize to stay within the 200-tx Stellar reconciliation window.
			var confirmedBurnIDs []uint64
			withdrawEvents := data.Events.WithdrawReadyEvents
			if len(withdrawEvents) > maxExecutionBatchSize {
				withdrawEvents = withdrawEvents[:maxExecutionBatchSize]
			}
			for _, withdrawReadyEvent := range withdrawEvents {
				txID, err := bridge.handleWithdrawReady(ctx, withdrawReadyEvent)
				if err != nil {
					if errors.Is(err, pkg.ErrTransactionAlreadyBurned) {
						continue
					}
					return errors.Wrap(err, "an error occurred while handling WithdrawReadyEvents")
				}
				if txID > 0 {
					confirmedBurnIDs = append(confirmedBurnIDs, txID)
				}
			}

			var confirmedRefundHashes []string
			refundEvents := data.Events.RefundReadyEvents
			if len(refundEvents) > maxExecutionBatchSize {
				refundEvents = refundEvents[:maxExecutionBatchSize]
			}
			for _, refundReadyEvent := range refundEvents {
				txHash, err := bridge.handleRefundReady(ctx, refundReadyEvent)
				if err != nil {
					if errors.Is(err, pkg.ErrTransactionAlreadyRefunded) {
						continue
					}
					return errors.Wrap(err, "an error occurred while handling RefundReadyEvents")
				}
				if txHash != "" {
					confirmedRefundHashes = append(confirmedRefundHashes, txHash)
				}
			}

			// Batch all TFChain confirmations into single force_batch extrinsics.
			// This confirms all burns/refunds in one block instead of N sequential blocks.
			if err := bridge.subClient.BatchSetWithdrawExecuted(ctx, confirmedBurnIDs); err != nil {
				return errors.Wrap(err, "failed to batch set withdraws executed")
			}
			for _, txID := range confirmedBurnIDs {
				if err := bridge.idempotency.MarkWithdrawCompleted(txID); err != nil {
					log.Warn().Err(err).Uint64("tx_id", txID).Msg("idempotency: failed to mark withdraw completed")
				}
				log.Info().
					Str("event_action", "withdraw_completed").
					Str("event_kind", "event").
					Str("category", "withdraw").
					Str("trace_id", fmt.Sprint(txID)).
					Msg("the withdraw has proceed")
				log.Info().
					Str("event_action", "transfer_completed").
					Str("event_kind", "event").
					Str("category", "transfer").
					Str("trace_id", fmt.Sprint(txID)).
					Dict("metadata", zerolog.Dict().
						Str("outcome", "bridged")).
					Msg("the transfer has completed")
			}

			if err := bridge.subClient.BatchSetRefundTransactionExecuted(ctx, confirmedRefundHashes); err != nil {
				return errors.Wrap(err, "failed to batch set refunds executed")
			}
			for _, txHash := range confirmedRefundHashes {
				if err := bridge.idempotency.MarkRefundCompleted(txHash); err != nil {
					log.Warn().Err(err).Str("tx_hash", txHash).Msg("idempotency: failed to mark refund completed")
				}
				log.Info().
					Str("event_action", "refund_completed").
					Str("event_kind", "event").
					Str("category", "refund").
					Str("trace_id", txHash).
					Msg("the transaction has refunded")
				log.Info().
					Str("event_action", "transfer_completed").
					Str("event_kind", "event").
					Str("category", "transfer").
					Str("trace_id", txHash).
					Dict("metadata", zerolog.Dict().
						Str("outcome", "refunded")).
					Msg("the transfer has completed")
			}

			// Batch all proposal events (BurnCreated, BurnExpired, RefundExpired)
			// into a single Utility.force_batch extrinsic. This drains backlogs from
			// bridge outages in one block rather than N sequential blocks.
			// Note: RefundCreated is intentionally excluded — see handleProposalsBatch.
			if err := bridge.handleProposalsBatch(
				ctx,
				data.Events.WithdrawCreatedEvents,
				data.Events.WithdrawExpiredEvents,
				data.Events.RefundExpiredEvents,
			); err != nil {
				return errors.Wrap(err, "an error occurred while handling proposal events")
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

// reconcilePendingTransactions handles crash recovery by checking all transactions
// that were in PROCESSING state when the bridge last shut down. For each one,
// it checks whether the Stellar tx was actually submitted, and if so, completes
// the TFChain confirmation step.
func (bridge *Bridge) reconcilePendingTransactions(ctx context.Context) error {
	log.Info().Msg("reconciling pending transactions from previous run...")

	pendingWithdraws, err := bridge.idempotency.GetPendingWithdraws()
	if err != nil {
		return errors.Wrap(err, "failed to get pending withdraws")
	}
	pendingRefunds, err := bridge.idempotency.GetPendingRefunds()
	if err != nil {
		return errors.Wrap(err, "failed to get pending refunds")
	}

	// If there are no pending transactions, skip the Horizon fetch entirely.
	if len(pendingWithdraws) == 0 && len(pendingRefunds) == 0 {
		log.Info().Msg("reconciliation complete: no pending transactions")
		return nil
	}

	// Fetch outgoing transactions once and reuse the page for all lookups,
	// avoiding one Horizon HTTP call per pending transaction.
	outgoingPage, err := bridge.wallet.FetchOutgoingTransactionsPage(ctx)
	if err != nil {
		log.Warn().Err(err).Msg("failed to fetch Horizon transactions for reconciliation, pending transactions will retry on next event")
		// Non-fatal: pending txs will be retried when the next Ready event fires.
		outgoingPage = hProtocol.TransactionsPage{}
	}

	// Reconcile pending withdraws
	for _, txID := range pendingWithdraws {
		log.Info().Uint64("tx_id", txID).Msg("reconciling pending withdraw")

		var stellarTx *hProtocol.Transaction

		// Primary: find by memo (current bridge behaviour)
		if tx := bridge.wallet.FindPaymentByMemoInPage(outgoingPage, fmt.Sprint(txID)); tx != nil {
			stellarTx = tx
		}

		// Fallback: find by sequence number (pre-upgrade compatibility, no memo)
		if stellarTx == nil {
			burnTx, err := bridge.subClient.GetBurnTransaction(types.U64(txID))
			if err != nil {
				log.Warn().Err(err).Uint64("tx_id", txID).Msg("failed to get burn tx for sequence lookup during reconciliation")
			} else if tx := bridge.wallet.FindPaymentBySequenceInPage(outgoingPage, int64(burnTx.SequenceNumber)); tx != nil {
				log.Info().Uint64("tx_id", txID).Int64("sequence_number", int64(burnTx.SequenceNumber)).
					Msg("reconcile: found pre-upgrade Stellar tx by sequence number (no memo)")
				stellarTx = tx
			}
		}

		if stellarTx != nil {
			log.Info().Uint64("tx_id", txID).Msg("found existing Stellar tx, completing TFChain confirmation")
			if err := bridge.subClient.RetrySetWithdrawExecuted(ctx, txID); err != nil {
				log.Warn().Err(err).Uint64("tx_id", txID).Msg("failed to set withdraw executed during reconciliation")
				continue
			}
			if err := bridge.idempotency.MarkWithdrawCompleted(txID); err != nil {
				log.Warn().Err(err).Uint64("tx_id", txID).Msg("failed to mark withdraw completed during reconciliation")
			}
		} else {
			log.Info().Uint64("tx_id", txID).Msg("no Stellar tx found by memo or sequence, will retry on next event")
		}
	}

	// Reconcile pending refunds
	for _, txHash := range pendingRefunds {
		log.Info().Str("tx_hash", txHash).Msg("reconciling pending refund")

		var stellarTx *hProtocol.Transaction

		// Primary: find by MemoReturn hash (current bridge behaviour)
		if tx := bridge.wallet.FindRefundByReturnHashInPage(outgoingPage, txHash); tx != nil {
			stellarTx = tx
		}

		// Fallback: find by sequence number (pre-upgrade compatibility, no memo)
		if stellarTx == nil {
			refundTx, err := bridge.subClient.GetRefundTransaction(txHash)
			if err != nil {
				log.Warn().Err(err).Str("tx_hash", txHash).Msg("failed to get refund tx for sequence lookup during reconciliation")
			} else if tx := bridge.wallet.FindPaymentBySequenceInPage(outgoingPage, int64(refundTx.SequenceNumber)); tx != nil {
				log.Info().Str("tx_hash", txHash).Int64("sequence_number", int64(refundTx.SequenceNumber)).
					Msg("reconcile: found pre-upgrade Stellar refund tx by sequence number (no memo)")
				stellarTx = tx
			}
		}

		if stellarTx != nil {
			log.Info().Str("tx_hash", txHash).Msg("found existing Stellar refund tx, completing TFChain confirmation")
			if err := bridge.subClient.RetrySetRefundTransactionExecutedTx(ctx, txHash); err != nil {
				log.Warn().Err(err).Str("tx_hash", txHash).Msg("failed to set refund executed during reconciliation")
				continue
			}
			if err := bridge.idempotency.MarkRefundCompleted(txHash); err != nil {
				log.Warn().Err(err).Str("tx_hash", txHash).Msg("failed to mark refund completed during reconciliation")
			}
		} else {
			log.Info().Str("tx_hash", txHash).Msg("no Stellar tx found by return hash or sequence, will retry on next event")
		}
	}

	log.Info().
		Int("pending_withdraws", len(pendingWithdraws)).
		Int("pending_refunds", len(pendingRefunds)).
		Msg("reconciliation complete")

	return nil
}
