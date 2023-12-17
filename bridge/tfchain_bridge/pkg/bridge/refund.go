package bridge

import (
	"context"
	"fmt"

	"github.com/pkg/errors"
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
	hProtocol "github.com/stellar/go/protocols/horizon"
	"github.com/threefoldtech/tfchain_bridge/pkg"
	_logger "github.com/threefoldtech/tfchain_bridge/pkg/logger"
	subpkg "github.com/threefoldtech/tfchain_bridge/pkg/substrate"
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
	if err = bridge.blockPersistency.SaveStellarCursor(cursor); err != nil {
		return errors.Wrap(err, "an error occurred while saving stellar cursor")
	}
	return nil
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

	reason := fmt.Sprint(_logger.GetRefundReason(ctx))
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

	// Todo, retry here?
	if err = bridge.wallet.CreateRefundPaymentWithSignaturesAndSubmit(ctx, refund.Target, uint64(refund.Amount), refund.TxHash, refund.Signatures, int64(refund.SequenceNumber)); err != nil {
		return err
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
