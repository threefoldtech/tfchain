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

// handleWithdrawCreatedBatch processes all WithdrawCreated events from a single block
// by batching their proposal calls into a single Utility.batch extrinsic.
// This reduces N×6s of sequential proposal submissions to 1×6s for N events.
func (bridge *Bridge) handleWithdrawCreatedBatch(ctx context.Context, events []subpkg.WithdrawCreatedEvent) error {
	if len(events) == 0 {
		return nil
	}

	// For a single event, fall back to the non-batched path
	if len(events) == 1 {
		err := bridge.handleWithdrawCreated(ctx, events[0])
		if err != nil && (errors.Is(err, pkg.ErrTransactionAlreadyBurned) || errors.Is(err, pkg.ErrTransactionAlreadyMinted)) {
			return nil
		}
		return err
	}

	log.Info().Int("count", len(events)).Msg("batch processing WithdrawCreated events")

	// Phase 1: Pre-check each event and generate Stellar signatures for valid ones
	type validProposal struct {
		event          subpkg.WithdrawCreatedEvent
		signature      string
		sequenceNumber uint64
	}
	var proposals []validProposal

	for _, withdraw := range events {
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
			continue
		}

		// Check if the target Stellar account can receive TFT
		if err := bridge.wallet.CheckAccount(withdraw.Target); err != nil {
			ctx := _logger.WithRefundReason(ctx, err.Error())
			if err := bridge.handleBadWithdraw(ctx, withdraw); err != nil {
				if errors.Is(err, pkg.ErrTransactionAlreadyMinted) {
					continue
				}
				return err
			}
			continue
		}

		signature, sequenceNumber, err := bridge.wallet.CreatePaymentAndReturnSignature(ctx, withdraw.Target, withdraw.Amount, withdraw.ID)
		if err != nil {
			return err
		}

		proposals = append(proposals, validProposal{
			event:          withdraw,
			signature:      signature,
			sequenceNumber: sequenceNumber,
		})
	}

	if len(proposals) == 0 {
		return nil
	}

	// Phase 2: Build and submit all proposals as a single batch
	batchProposals := make([]subpkg.BurnProposal, 0, len(proposals))
	for _, p := range proposals {
		batchProposals = append(batchProposals, subpkg.BurnProposal{
			TxID:           p.event.ID,
			Target:         p.event.Target,
			Amount:         new(big.Int).SetUint64(p.event.Amount),
			Signature:      p.signature,
			StellarAddress: bridge.wallet.GetKeypair().Address(),
			SequenceNumber: p.sequenceNumber,
		})
	}

	result, err := bridge.subClient.BatchProposeWithdrawOrAddSig(ctx, batchProposals)
	if err != nil {
		// On batch failure, fall back to individual submissions
		log.Warn().Err(err).Msg("batch proposal failed, falling back to individual submissions")
		for _, p := range proposals {
			if err := bridge.subClient.RetryProposeWithdrawOrAddSig(ctx, p.event.ID, p.event.Target, new(big.Int).SetUint64(p.event.Amount), p.signature, bridge.wallet.GetKeypair().Address(), p.sequenceNumber); err != nil {
				log.Warn().Err(err).Uint64("tx_id", p.event.ID).Msg("individual proposal also failed")
			}
		}
		return nil
	}

	// Phase 3: Log results
	if result.FailedCount > 0 {
		log.Warn().
			Int("failed", result.FailedCount).
			Int("total", len(proposals)).
			Msg("some proposals failed within batch (may already be signed or expired)")
	}
	for _, p := range proposals {
		log.Info().
			Str("trace_id", fmt.Sprint(p.event.ID)).
			Str("event_action", "withdraw_proposed").
			Str("event_kind", "event").
			Str("category", "withdraw").
			Dict("metadata", zerolog.Dict().
				Uint64("amount", p.event.Amount).
				Str("tx_id", fmt.Sprint(p.event.ID)).
				Str("to", p.event.Target)).
			Msgf("a withdraw has proposed with the target stellar address of %s", p.event.Target)
	}

	log.Info().
		Int("total", len(proposals)).
		Int("succeeded", result.SuccessCount).
		Int("failed", result.FailedCount).
		Msg("batch proposal completed")

	return nil
}

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

	// 1. Check idempotency store
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

	// 2. If PROCESSING, check if Stellar tx was already submitted (crash recovery)
	if state == pkg.TxStateProcessing {
		logger.Warn().
			Str("event_action", "withdraw_crash_recovery").
			Str("event_kind", "event").
			Str("category", "withdraw").
			Msg("idempotency: withdraw in PROCESSING state (possible crash recovery)")

		// Primary check: look for a tx with matching memo (current bridge behaviour)
		stellarTx, err := bridge.wallet.FindPaymentByMemo(ctx, txKey)
		if err != nil {
			return err
		}
		if stellarTx != nil {
			logger.Info().
				Str("event_action", "withdraw_recovered").
				Str("event_kind", "event").
				Str("category", "withdraw").
				Msg("idempotency: found existing Stellar tx by memo, completing TFChain confirmation")
			if err := bridge.subClient.RetrySetWithdrawExecuted(ctx, txID); err != nil {
				return err
			}
			return bridge.idempotency.MarkWithdrawCompleted(txID)
		}

		// Fallback: look for a tx by sequence number, covering pre-upgrade submissions
		// that were made without a memo. The sequence number stored in the TFChain burn tx
		// is the exact sequence used when building the Stellar tx, uniquely identifying it.
		// Since the bridge is stopped during upgrades, no new outgoing txs can appear
		// between the old submission and this lookup, so 200 records is always sufficient.
		burnTxForSeq, err := bridge.subClient.GetBurnTransaction(types.U64(txID))
		if err != nil {
			return err
		}
		stellarTxBySeq, err := bridge.wallet.FindPaymentBySequence(ctx, int64(burnTxForSeq.SequenceNumber))
		if err != nil {
			return err
		}
		if stellarTxBySeq != nil {
			logger.Info().
				Str("event_action", "withdraw_recovered").
				Str("event_kind", "event").
				Str("category", "withdraw").
				Int64("sequence_number", int64(burnTxForSeq.SequenceNumber)).
				Msg("idempotency: found pre-upgrade Stellar tx by sequence number (no memo), completing TFChain confirmation")
			if err := bridge.subClient.RetrySetWithdrawExecuted(ctx, txID); err != nil {
				return err
			}
			return bridge.idempotency.MarkWithdrawCompleted(txID)
		}

		logger.Info().Msg("idempotency: no Stellar tx found by memo or sequence, safe to retry")
	}

	// 3. Check TFChain: already burned?
	burned, err := bridge.subClient.IsBurnedAlready(types.U64(txID))
	if err != nil {
		return err
	}
	if burned {
		_ = bridge.idempotency.MarkWithdrawCompleted(txID)
		logger.Info().
			Str("event_action", "withdraw_skipped").
			Str("event_kind", "event").
			Str("category", "withdraw").
			Msg("the withdraw transaction has already been processed")
		return pkg.ErrTransactionAlreadyBurned
	}

	// 4. Get burn tx with signatures
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

	// 5. Mark PROCESSING before Stellar submit
	if err := bridge.idempotency.MarkWithdrawProcessing(txID); err != nil {
		return err
	}

	// 6. Submit to Stellar
	err = bridge.wallet.CreatePaymentWithSignaturesAndSubmit(ctx, burnTx.Target, uint64(burnTx.Amount), txKey, burnTx.Signatures, int64(burnTx.SequenceNumber))
	if err != nil {
		logger.Info().
			Str("event_action", "withdraw_postponed").
			Str("event_kind", "event").
			Str("category", "withdraw").
			Dict("metadata", zerolog.Dict().
				Str("reason", err.Error())).
			Msgf("the withdraw has been postponed due to a problem in sending this transaction to the stellar network. error was %s", err.Error())
		return nil // leave as PROCESSING, will reconcile on next attempt
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

	// 7. Mark executed on TFChain
	if err := bridge.subClient.RetrySetWithdrawExecuted(ctx, txID); err != nil {
		return err
	}

	// 8. Mark COMPLETED
	return bridge.idempotency.MarkWithdrawCompleted(txID)
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
