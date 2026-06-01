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
	refunded, err := bridge.subClient.IsRefundedAlready(refundReadyEvent.Hash)
	if err != nil {
		return err
	}

	if refunded {
		logger.Info().
			Str("event_action", "refund_skipped").
			Str("event_kind", "event").
			Str("category", "refund").
			Msg("the transaction has already been refunded")
		return pkg.ErrTransactionAlreadyRefunded
	}

	refund, err := bridge.subClient.GetRefundTransaction(refundReadyEvent.Hash)
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

	// Todo, retry here?
	if err = bridge.wallet.CreateRefundPaymentWithSignaturesAndSubmit(ctx, refund.Target, uint64(refund.Amount), refund.TxHash, refund.Signatures, int64(refund.SequenceNumber)); err != nil {
		// A refund submission must never crash the bridge. A single failure would
		// otherwise propagate up to a fatal exit, and the on-chain on_finalize
		// retry would re-emit the event every RetryInterval blocks, producing an
		// endless crash loop.
		if errors.Is(err, pkg.ErrStellarTransactionUndeliverable) {
			// Target-side, bridge-unresolvable (e.g. the target removed its Stellar
			// trustline). It can never be delivered, so quarantine it by marking it
			// executed on chain to stop the on-chain retries. The refunded TFT is
			// forfeited.
			logger.Warn().
				Err(err).
				Str("event_action", "refund_quarantined").
				Str("event_kind", "alert").
				Str("category", "refund").
				Dict("metadata", zerolog.Dict().
					Str("target", refund.Target).
					Uint64("amount", uint64(refund.Amount))).
				Msg("the refund is permanently undeliverable to the target account and has been quarantined; the bridge will continue")

			if execErr := bridge.subClient.RetrySetRefundTransactionExecutedTx(ctx, refund.TxHash); execErr != nil {
				return execErr
			}
			return nil
		}

		// Any other failure (transient network/sequence issue, or a recoverable
		// target condition such as op_line_full) is postponed: log and continue,
		// relying on the on-chain on_finalize retry to re-emit the event so we try
		// again later. Matches handleWithdrawReady's behaviour.
		logger.Info().
			Str("event_action", "refund_postponed").
			Str("event_kind", "event").
			Str("category", "refund").
			Dict("metadata", zerolog.Dict().
				Str("reason", err.Error())).
			Msg("the refund has been postponed due to a problem submitting the transaction to the Stellar network; it will be retried")
		return nil
	}

	err = bridge.subClient.RetrySetRefundTransactionExecutedTx(ctx, refund.TxHash)
	if err != nil {
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
