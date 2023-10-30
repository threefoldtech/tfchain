package bridge

import (
	"context"
	"fmt"

	"github.com/pkg/errors"
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
	hProtocol "github.com/stellar/go/protocols/horizon"
	"github.com/threefoldtech/tfchain_bridge/pkg"
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
		return errors.Wrap(err, "error while saving cursor")
	}
	return nil
}

func (bridge *Bridge) handleRefundExpired(ctx context.Context, refundExpiredEvent subpkg.RefundTransactionExpiredEvent) error {
	logger := log.Logger.With().Str("span_id", refundExpiredEvent.Hash).Logger()

	refunded, err := bridge.subClient.IsRefundedAlready(refundExpiredEvent.Hash)
	if err != nil {
		return err
	}

	if refunded {
		logger.Info().
			Str("event_type", "refund_skipped").
			Msg("tx is already refunded")
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
	reason := fmt.Sprint(ctx.Value("refund_reason"))
	logger.Info().
		Str("event_type", "refund_proposed").
		Dict("event", zerolog.Dict().
			Str("reason", reason)).
		Msgf("refund initiated due to %s", reason)
	return nil
}

func (bridge *Bridge) handleRefundReady(ctx context.Context, refundReadyEvent subpkg.RefundTransactionReadyEvent) error {
	logger := log.Logger.With().Str("span_id", refundReadyEvent.Hash).Logger()
	refunded, err := bridge.subClient.IsRefundedAlready(refundReadyEvent.Hash)
	if err != nil {
		return err
	}

	if refunded {
		logger.Info().
			Str("event_type", "refund_skipped").
			Msg("tx is already refunded")
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
		Str("event_type", "refund_completed").
		Msg("refund processed")
	logger.Info().
		Str("event_type", "transfer_completed").
		Dict("event", zerolog.Dict().
			Str("outcome", "refunded")).
		Msgf("transfer with id %s completed", refundReadyEvent.Hash)

	return nil
}
