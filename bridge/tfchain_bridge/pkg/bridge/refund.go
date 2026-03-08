package bridge

import (
	"context"

	"github.com/pkg/errors"
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
	hProtocol "github.com/stellar/go/protocols/horizon"
	"github.com/threefoldtech/tfchain/bridge/tfchain_bridge/pkg"
	_logger "github.com/threefoldtech/tfchain/bridge/tfchain_bridge/pkg/logger"
	subpkg "github.com/threefoldtech/tfchain/bridge/tfchain_bridge/pkg/substrate"
)

// refund handler for stellar
func (bridge *Bridge) refund(ctx context.Context, destination string, amount int64, tx hProtocol.Transaction) error {
	err := bridge.handleRefundExpired(ctx, subpkg.RefundTransactionExpiredEvent{
		Hash:   tx.Hash,
		Amount: uint64(amount),
		Target: destination,
	})
	if err != nil {
		return err
	}

	// save cursor
	cursor := tx.PagingToken()
	err = bridge.blockPersistency.SaveStellarCursor(cursor)
	// no need to check for err, if err is nil, Wrap returns nil.
	return errors.Wrap(err, "an error occurred while saving stellar cursor")
}

func (bridge *Bridge) handleRefundExpired(ctx context.Context, refundExpiredEvent subpkg.RefundTransactionExpiredEvent) error {
	logger := log.Logger.With().Str("trace_id", refundExpiredEvent.Hash).Logger()

	refunded, err := bridge.subClient.IsRefundedAlready(refundExpiredEvent.Hash)
	if err != nil {
		return err
	}

	if refunded {
		logger.Info().
			Str("event_action", "refund_skipped").
			Str("event_kind", "event").
			Str("category", "refund").
			Msg("the transaction has already been refunded")
		return nil
	}

	// Sync sequence counter before signing (see SyncSequenceNumber for rationale).
	if err := bridge.wallet.SyncSequenceNumber(); err != nil {
		return err
	}
	signature, sequenceNumber, err := bridge.wallet.CreateRefundAndReturnSignature(ctx, refundExpiredEvent.Target, refundExpiredEvent.Amount, refundExpiredEvent.Hash)
	if err != nil {
		return err
	}

	err = bridge.subClient.RetryCreateRefundTransactionOrAddSig(ctx, refundExpiredEvent.Hash, refundExpiredEvent.Target, int64(refundExpiredEvent.Amount), signature, bridge.wallet.GetKeypair().Address(), sequenceNumber)
	if err != nil {
		return err
	}

	reason := _logger.GetRefundReason(ctx)
	logger.Info().
		Str("event_action", "refund_proposed").
		Str("event_kind", "event").
		Str("category", "refund").
		Dict("metadata", zerolog.Dict().
			Str("reason", reason)).
		Msgf("a refund has proposed due to %s", reason)
	return nil
}

func (bridge *Bridge) handleRefundReady(ctx context.Context, refundReadyEvent subpkg.RefundTransactionReadyEvent) error {
	logger := log.Logger.With().Str("trace_id", refundReadyEvent.Hash).Logger()
	txHash := refundReadyEvent.Hash

	// 1. Check idempotency store
	state, err := bridge.idempotency.GetRefundState(txHash)
	if err != nil {
		return err
	}
	if state == pkg.TxStateCompleted {
		logger.Info().
			Str("event_action", "refund_skipped").
			Str("event_kind", "event").
			Str("category", "refund").
			Msg("idempotency: refund already completed, skipping")
		return pkg.ErrTransactionAlreadyRefunded
	}

	// 2. If PROCESSING, check if Stellar tx was already submitted (crash recovery)
	if state == pkg.TxStateProcessing {
		logger.Warn().
			Str("event_action", "refund_crash_recovery").
			Str("event_kind", "event").
			Str("category", "refund").
			Msg("idempotency: refund in PROCESSING state (possible crash recovery)")

		// Fetch the outgoing transactions page once and reuse it for both lookups.
		// Non-fatal on error: match the reconciler's behavior. Leave tx as PROCESSING;
		// the next RefundTransactionReady event will retry the Horizon lookup.
		outgoingPage, err := bridge.wallet.FetchOutgoingTransactionsPage(ctx)
		if err != nil {
			logger.Warn().Err(err).Str("tx_hash", txHash).
				Msg("failed to fetch Horizon transactions for PROCESSING check; will retry on next event")
			return nil
		}

		// Primary check: look for a refund tx with matching MemoReturn hash (current bridge behaviour)
		if stellarTx := bridge.wallet.FindRefundByReturnHashInPage(outgoingPage, txHash); stellarTx != nil {
			logger.Info().
				Str("event_action", "refund_recovered").
				Str("event_kind", "event").
				Str("category", "refund").
				Msg("idempotency: found existing Stellar tx by return hash, completing TFChain confirmation")
			if err := bridge.subClient.RetrySetRefundTransactionExecutedTx(ctx, txHash); err != nil {
				return err
			}
			return bridge.idempotency.MarkRefundCompleted(txHash)
		}

		// Fallback: look for a tx by sequence number, covering pre-upgrade submissions
		// that were made without a memo. See FindPaymentBySequenceInPage for rationale.
		refundTxForSeq, err := bridge.subClient.GetRefundTransaction(txHash)
		if err != nil {
			return err
		}
		if stellarTxBySeq := bridge.wallet.FindPaymentBySequenceInPage(outgoingPage, int64(refundTxForSeq.SequenceNumber)); stellarTxBySeq != nil {
			logger.Info().
				Str("event_action", "refund_recovered").
				Str("event_kind", "event").
				Str("category", "refund").
				Int64("sequence_number", int64(refundTxForSeq.SequenceNumber)).
				Msg("idempotency: found pre-upgrade Stellar tx by sequence number (no memo), completing TFChain confirmation")
			if err := bridge.subClient.RetrySetRefundTransactionExecutedTx(ctx, txHash); err != nil {
				return err
			}
			return bridge.idempotency.MarkRefundCompleted(txHash)
		}

		logger.Info().Msg("idempotency: no Stellar tx found by return hash or sequence, safe to retry")
	}

	// 3. Check TFChain: already refunded?
	refunded, err := bridge.subClient.IsRefundedAlready(txHash)
	if err != nil {
		return err
	}
	if refunded {
		_ = bridge.idempotency.MarkRefundCompleted(txHash)
		logger.Info().
			Str("event_action", "refund_skipped").
			Str("event_kind", "event").
			Str("category", "refund").
			Msg("the transaction has already been refunded")
		return pkg.ErrTransactionAlreadyRefunded
	}

	// 4. Get refund tx with signatures
	refund, err := bridge.subClient.GetRefundTransaction(txHash)
	if err != nil {
		return err
	}
	if len(refund.Signatures) == 0 {
		logger.Info().
			Str("event_action", "refund_postponed").
			Str("event_kind", "event").
			Str("category", "refund").
			Msg("the refund has been postponed due to the transaction signatures being removed on the TFChain side while the bridge was processing the transaction")
		return nil
	}

	// 5. Mark PROCESSING before Stellar submit
	if err := bridge.idempotency.MarkRefundProcessing(txHash); err != nil {
		return err
	}

	// 6. Submit to Stellar
	if err = bridge.wallet.CreateRefundPaymentWithSignaturesAndSubmit(ctx, refund.Target, uint64(refund.Amount), refund.TxHash, refund.Signatures, int64(refund.SequenceNumber)); err != nil {
		logger.Info().
			Str("event_action", "refund_postponed").
			Str("event_kind", "event").
			Str("category", "refund").
			Dict("metadata", zerolog.Dict().
				Str("reason", err.Error())).
			Msgf("the refund has been postponed due to a problem in sending this transaction to the stellar network. error was %s", err.Error())
		return nil // leave as PROCESSING, will reconcile on next attempt
	}

	// 7. Mark executed on TFChain
	if err := bridge.subClient.RetrySetRefundTransactionExecutedTx(ctx, refund.TxHash); err != nil {
		return err
	}

	// 8. Mark COMPLETED
	if err := bridge.idempotency.MarkRefundCompleted(txHash); err != nil {
		return err
	}

	logger.Info().
		Str("event_action", "refund_completed").
		Str("event_kind", "event").
		Str("category", "refund").
		Msg("the transaction has refunded")
	logger.Info().
		Str("event_action", "transfer_completed").
		Str("event_kind", "event").
		Str("category", "transfer").
		Dict("metadata", zerolog.Dict().
			Str("outcome", "refunded")).
		Msg("the transfer has completed")

	return nil
}
