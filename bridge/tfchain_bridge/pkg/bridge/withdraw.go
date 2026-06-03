package bridge

import (
	"context"
	"errors"
	"fmt"
	"math/big"

	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
	"github.com/threefoldtech/tfchain/bridge/tfchain_bridge/pkg"
	_logger "github.com/threefoldtech/tfchain/bridge/tfchain_bridge/pkg/logger"
	subpkg "github.com/threefoldtech/tfchain/bridge/tfchain_bridge/pkg/substrate"
	substrate "github.com/threefoldtech/tfchain/clients/tfchain-client-go"
)

func (bridge *Bridge) handleWithdrawCreated(ctx context.Context, withdraw subpkg.WithdrawCreatedEvent) error {
	logger := log.Logger.With().Str("trace_id", fmt.Sprint(withdraw.ID)).Logger()

	burned, err := bridge.subClient.IsBurnedAlready(types.U64(withdraw.ID))
	if err != nil {
		return err
	}

	if burned {
		logger.Info().
			Str("event_action", "withdraw_skipped").
			Str("event_kind", "event").
			Str("category", "withdraw").
			Msg("the withdraw transaction has already been processed")
		return pkg.ErrTransactionAlreadyBurned
	}

	logger.Info().
		Str("event_action", "transfer_initiated").
		Str("event_kind", "event").
		Str("category", "transfer").
		Dict("metadata", zerolog.Dict().
			Str("type", "burn")).
		Msg("a transfer has initiated")

	// check if it can hold tft : TODO check trust line TFT limit if it can receive the amount
	if err := bridge.wallet.CheckAccount(withdraw.Target); err != nil {
		ctx = _logger.WithRefundReason(ctx, err.Error())
		return bridge.handleBadWithdraw(ctx, withdraw)
	}

	signature, sequenceNumber, err := bridge.wallet.CreatePaymentAndReturnSignature(ctx, withdraw.Target, withdraw.Amount, withdraw.ID)
	if err != nil {
		return err
	}
	log.Debug().Msgf("stellar account sequence number: %d", sequenceNumber)

	err = bridge.subClient.RetryProposeWithdrawOrAddSig(ctx, withdraw.ID, withdraw.Target, big.NewInt(int64(withdraw.Amount)), signature, bridge.wallet.GetKeypair().Address(), sequenceNumber)
	if err != nil {
		return nil
	}

	logger.Info().
		Str("event_action", "withdraw_proposed").
		Str("event_kind", "event").
		Str("category", "withdraw").
		Dict("metadata", zerolog.Dict().
			Uint64("amount", withdraw.Amount).
			Str("tx_id", fmt.Sprint(withdraw.ID)).
			Str("to", withdraw.Target)).
		Msgf("a withdraw has proposed with the target stellar address of %s", withdraw.Target)
	return nil
}

func (bridge *Bridge) handleWithdrawExpired(ctx context.Context, withdrawExpired subpkg.WithdrawExpiredEvent) error {
	logger := log.Logger.With().Str("trace_id", fmt.Sprint(withdrawExpired.ID)).Logger()

	ok, source := withdrawExpired.Source.Unwrap() // transfers from the previous runtime before 147 has no source address

	if !ok {
		// This path is intended solely for processing transfers that lack a source address
		// and should be retained until the network has been verified to have no transfers from the previous runtime before 147.

		if err := bridge.wallet.CheckAccount(withdrawExpired.Target); err != nil {
			logger.Warn().
				Str("event_action", "transfer_failed").
				Str("event_kind", "alert").
				Str("category", "transfer").
				Dict("metadata", zerolog.Dict().
					Str("reason", err.Error())).
				Str("type", "burn").
				Msg("a withdraw failed with no way to refund!")
			return bridge.subClient.RetrySetWithdrawExecuted(ctx, withdrawExpired.ID)
		}

		signature, sequenceNumber, err := bridge.wallet.CreatePaymentAndReturnSignature(ctx, withdrawExpired.Target, withdrawExpired.Amount, withdrawExpired.ID)
		if err != nil {
			return err
		}
		log.Debug().Msgf("stellar account sequence number: %d", sequenceNumber)

		err = bridge.subClient.RetryProposeWithdrawOrAddSig(ctx, withdrawExpired.ID, withdrawExpired.Target, big.NewInt(int64(withdrawExpired.Amount)), signature, bridge.wallet.GetKeypair().Address(), sequenceNumber)
		if err != nil {
			return err
		}
		logger.Info().
			Str("event_action", "transfer_initiated").
			Str("event_kind", "event").
			Str("category", "transfer").
			Dict("metadata", zerolog.Dict().
				Str("type", "burn")).
			Msg("a transfer has initiated")
		logger.Info().
			Str("event_action", "withdraw_proposed").
			Str("event_kind", "event").
			Str("category", "withdraw").
			Dict("metadata", zerolog.Dict().
				Uint64("amount", withdrawExpired.Amount).
				Str("tx_id", fmt.Sprint(withdrawExpired.ID)).
				Str("to", withdrawExpired.Target)).
			Msgf("a withdraw has proposed with the target stellar address of %s", withdrawExpired.Target)
		return nil
	}

	// refundable path (starting from tfchain runtime 147)
	return bridge.handleWithdrawCreated(ctx, subpkg.WithdrawCreatedEvent{
		ID:     withdrawExpired.ID,
		Source: source,
		Target: withdrawExpired.Target,
		Amount: withdrawExpired.Amount,
	})
}

func (bridge *Bridge) handleWithdrawReady(ctx context.Context, withdrawReady subpkg.WithdrawReadyEvent) error {
	logger := log.Logger.With().Str("trace_id", fmt.Sprint(withdrawReady.ID)).Logger()
	txID := withdrawReady.ID
	txKey := fmt.Sprint(txID)

	// 1. Idempotency: if we've already fully processed this withdraw, skip it.
	state, err := bridge.idempotency.GetWithdrawState(txID)
	if err != nil {
		return err
	}
	if state == pkg.TxStateCompleted {
		logger.Info().
			Str("event_action", "withdraw_skipped").
			Str("event_kind", "event").
			Str("category", "withdraw").
			Msg("idempotency: withdraw already completed, skipping")
		return pkg.ErrTransactionAlreadyBurned
	}

	// 2. If the withdraw is PROCESSING, the Stellar payment may already have been
	// submitted before a crash. Look for an existing outgoing payment by its text memo
	// (the burn tx id), falling back to the account sequence number for payments
	// submitted by a pre-memo bridge version. If found, the funds already left the
	// bridge, so only the TFChain confirmation is outstanding — complete that instead
	// of re-submitting (which would risk a double payment).
	if state == pkg.TxStateProcessing {
		logger.Warn().
			Str("event_action", "withdraw_crash_recovery").
			Str("event_kind", "event").
			Str("category", "withdraw").
			Msg("idempotency: withdraw in PROCESSING state, checking Stellar for an existing payment")

		// Non-fatal on Horizon error: leave the tx PROCESSING and retry on the next
		// event. Returning an error here would crash the bridge during a Horizon
		// outage, which is worse than gracefully deferring.
		outgoingPage, ferr := bridge.wallet.FetchOutgoingTransactionsPage(ctx)
		if ferr != nil {
			logger.Warn().Err(ferr).Uint64("tx_id", txID).
				Msg("failed to fetch Horizon transactions for PROCESSING check; will retry on next event")
			return nil
		}

		found := bridge.wallet.FindPaymentByMemoInPage(outgoingPage, txKey) != nil
		if !found {
			// Inconclusive recovery (chain read failed) must not fall through to a
			// re-submit, so on error leave PROCESSING and retry on the next event.
			burnTxForSeq, serr := bridge.subClient.GetBurnTransaction(types.U64(txID))
			if serr != nil {
				logger.Warn().Err(serr).Uint64("tx_id", txID).
					Msg("failed to get burn tx for sequence lookup during PROCESSING check; will retry on next event")
				return nil
			}
			found = bridge.wallet.FindPaymentBySequenceInPage(outgoingPage, int64(burnTxForSeq.SequenceNumber)) != nil
		}
		if found {
			logger.Info().
				Str("event_action", "withdraw_recovered").
				Str("event_kind", "event").
				Str("category", "withdraw").
				Msg("idempotency: found existing Stellar payment, completing TFChain confirmation")
			if cerr := bridge.subClient.RetrySetWithdrawExecuted(ctx, txID); cerr != nil {
				return cerr
			}
			if merr := bridge.idempotency.MarkWithdrawCompleted(txID); merr != nil {
				logger.Warn().Err(merr).Uint64("tx_id", txID).Msg("idempotency: failed to mark withdraw completed")
			}
			return nil
		}
		// Not found. If the payment had been submitted and then scrolled out of the
		// 200-record window, a re-submit reuses the same (already-consumed) sequence
		// and Stellar rejects it with tx_bad_seq, so no double payment can occur.
		logger.Info().Msg("idempotency: no Stellar payment found by memo or sequence, safe to retry")
	}

	// 3. Already burned on chain?
	burned, err := bridge.subClient.IsBurnedAlready(types.U64(txID))
	if err != nil {
		return err
	}
	if burned {
		if merr := bridge.idempotency.MarkWithdrawCompleted(txID); merr != nil {
			logger.Warn().Err(merr).Uint64("tx_id", txID).Msg("idempotency: failed to mark withdraw completed")
		}
		logger.Info().
			Str("event_action", "withdraw_skipped").
			Str("event_kind", "event").
			Str("category", "withdraw").
			Msg("the withdraw transaction has already been processed")
		return pkg.ErrTransactionAlreadyBurned
	}

	burnTx, err := bridge.subClient.GetBurnTransaction(types.U64(txID))
	if err != nil {
		return err
	}

	if len(burnTx.Signatures) == 0 {
		logger.Info().
			Str("event_action", "withdraw_postponed").
			Str("event_kind", "event").
			Str("category", "withdraw").
			Msg("the withdraw has been postponed due to the transaction signatures being removed on the TFChain side while the bridge was processing the transaction")
		return nil
	}

	// 4. Mark PROCESSING before submitting to Stellar, so a crash between the submit
	// and the TFChain confirmation is recoverable via the PROCESSING check above.
	if err := bridge.idempotency.MarkWithdrawProcessing(txID); err != nil {
		return err
	}

	// txKey (the burn tx id) is set as the payment's text memo for traceability and
	// crash recovery; see CreatePaymentWithSignaturesAndSubmit.
	err = bridge.wallet.CreatePaymentWithSignaturesAndSubmit(ctx, burnTx.Target, uint64(burnTx.Amount), txKey, burnTx.Signatures, int64(burnTx.SequenceNumber))
	if err != nil {
		// we can log and skip here as we could depend on tfcahin retry mechanism
		// to notify us again about related burn tx. The tx stays PROCESSING and is
		// reconciled on the next attempt.
		logger.Info().
			Str("event_action", "withdraw_postponed").
			Str("event_kind", "event").
			Str("category", "withdraw").
			Dict("metadata", zerolog.Dict().
				Str("reason", err.Error())).
			Msgf("the withdraw has been postponed due to a problem in sending this transaction to the stellar network. error was %s", err.Error())
		return nil
	}

	// 5. Stellar payment submitted — confirm on TFChain, then mark COMPLETED.
	if err := bridge.subClient.RetrySetWithdrawExecuted(ctx, txID); err != nil {
		return err
	}
	if err := bridge.idempotency.MarkWithdrawCompleted(txID); err != nil {
		logger.Warn().Err(err).Uint64("tx_id", txID).Msg("idempotency: failed to mark withdraw completed")
	}

	logger.Info().
		Str("event_action", "withdraw_completed").
		Str("event_kind", "event").
		Str("category", "withdraw").
		Msg("the withdraw has proceed")
	logger.Info().
		Str("event_action", "transfer_completed").
		Str("event_kind", "event").
		Str("category", "transfer").
		Dict("metadata", zerolog.Dict().
			Str("outcome", "bridged")).
		Msg("the transfer has completed")

	return nil
}

func (bridge *Bridge) handleBadWithdraw(ctx context.Context, withdraw subpkg.WithdrawCreatedEvent) error {
	logger := log.Logger.With().Str("trace_id", fmt.Sprint(withdraw.ID)).Logger()

	if withdraw.Amount <= uint64(bridge.depositFee) {
		logger.Warn().
			Str("event_action", "transfer_failed").
			Str("event_kind", "alert").
			Str("category", "transfer").
			Dict("metadata", zerolog.Dict().
				Str("reason", _logger.GetRefundReason(ctx))).
			Str("type", "burn").
			Msg("a withdraw failed with no remainder to refund (insufficient amount to cover deposit fee)!")

		return bridge.subClient.RetrySetWithdrawExecuted(ctx, withdraw.ID)
	}

	mintID := fmt.Sprintf("refund-%d", withdraw.ID)

	minted, err := bridge.subClient.IsMintedAlready(mintID)
	if err != nil {
		if !errors.Is(err, substrate.ErrMintTransactionNotFound) {
			return err
		}
	}

	if minted {
		logger.Info().
			Str("event_action", "mint_skipped").
			Str("event_kind", "event").
			Str("category", "mint").
			Msg("the transaction has already been minted")
		return pkg.ErrTransactionAlreadyMinted
	}

	err = bridge.subClient.RetryProposeMintOrVote(ctx, mintID, substrate.AccountID(withdraw.Source), big.NewInt(int64(withdraw.Amount)))
	if err != nil {
		return err
	}

	logger.Info().
		Str("event_action", "mint_proposed").
		Str("event_kind", "event").
		Str("category", "mint").
		Dict("metadata", zerolog.Dict().
			Int64("amount", int64(withdraw.Amount)).
			Str("tx_id", fmt.Sprint(withdraw.ID)).
			Str("to", withdraw.Source.ToHexString())).
		Msgf("a mint has proposed with the target substrate address of %s", withdraw.Source.ToHexString())
	return bridge.subClient.RetrySetWithdrawExecuted(ctx, withdraw.ID)
}
