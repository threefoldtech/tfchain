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

// handleProposalsBatch processes all proposal events from a single TFChain block in one
// Utility.force_batch extrinsic. This covers:
//   - BurnTransactionCreated  → propose_burn_transaction_or_add_sig
//   - BurnTransactionExpired  → same (re-sign with fresh Stellar sequence; pre-runtime-147 dropped)
//   - RefundTransactionCreated → create_refund_transaction_or_add_sig (allows validators that
//     missed the triggering Stellar deposit to add their signature without waiting for expiry)
//   - RefundTransactionExpired → same (offline-validator recovery path)
//
// Ready events (BurnTransactionReady, RefundTransactionReady) are NOT handled here — they
// involve actual Stellar submissions, not TFChain extrinsics, and remain sequential.
// Deposit-triggered refunds (mint.go → refund()) are also NOT batched here; they call
// handleRefundExpired directly so that all validators propose at the same time with a
// consistent Stellar sequence number.
func (bridge *Bridge) handleProposalsBatch(
	ctx context.Context,
	withdrawCreated []subpkg.WithdrawCreatedEvent,
	withdrawExpired []subpkg.WithdrawExpiredEvent,
	refundCreated []subpkg.RefundTransactionCreatedEvent,
	refundExpired []subpkg.RefundTransactionExpiredEvent,
) error {
	// Step 1: Convert BurnTransactionExpired (≥runtime-147) to WithdrawCreatedEvent.
	// Pre-runtime-147 events (no source address) are no longer supported and are dropped.
	for _, e := range withdrawExpired {
		ok, source := e.Source.Unwrap()
		if !ok {
			log.Warn().
				Str("event_action", "withdraw_skipped").
				Str("event_kind", "alert").
				Str("category", "withdraw").
				Uint64("tx_id", e.ID).
				Msg("ignoring pre-runtime-147 expired withdraw (no source address); network should have no such transfers")
			continue
		}
		withdrawCreated = append(withdrawCreated, subpkg.WithdrawCreatedEvent{
			ID:     e.ID,
			Source: source,
			Target: e.Target,
			Amount: e.Amount,
		})
	}

	// Step 2: Normalise refund events — Created and Expired have identical fields.
	type refundItem struct {
		Hash   string
		Target string
		Amount uint64
	}
	var allRefunds []refundItem
	for _, e := range refundCreated {
		allRefunds = append(allRefunds, refundItem{e.Hash, e.Target, e.Amount})
	}
	for _, e := range refundExpired {
		allRefunds = append(allRefunds, refundItem{e.Hash, e.Target, e.Amount})
	}

	// Step 3: Early return if nothing to do.
	if len(withdrawCreated) == 0 && len(allRefunds) == 0 {
		return nil
	}

	log.Info().
		Str("event_action", "batch_proposal_started").
		Str("event_kind", "event").
		Str("category", "bridge").
		Int("withdraws", len(withdrawCreated)).
		Int("refunds", len(allRefunds)).
		Msg("batch processing proposal events")

	// Step 4: Sync the Stellar sequence counter ONCE before signing anything.
	// All proposals in this batch get consecutive sequence numbers from this base.
	// Syncing once (rather than per-proposal) is critical: proposals are TFChain
	// extrinsics, not Stellar submissions — the Stellar account sequence does not
	// advance between signing calls, so all signers of a given proposal must use
	// the same sequence. A fresh sync here ensures we start from the current live
	// account sequence, not a value that may have been advanced by a prior Ready event.
	if err := bridge.wallet.SyncSequenceNumber(); err != nil {
		return err
	}

	// Step 5: Build burn proposals.
	var burnProposals []subpkg.BurnProposal
	// Track which WithdrawCreatedEvent each proposal came from (for logging).
	type burnMeta struct {
		event subpkg.WithdrawCreatedEvent
		index int // index into burnProposals
	}
	var burnMetas []burnMeta

	for _, w := range withdrawCreated {
		logger := log.Logger.With().Str("trace_id", fmt.Sprint(w.ID)).Logger()

		burned, err := bridge.subClient.IsBurnedAlready(types.U64(w.ID))
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

		if err := bridge.wallet.CheckAccount(w.Target); err != nil {
			ctx := _logger.WithRefundReason(ctx, err.Error())
			if err := bridge.handleBadWithdraw(ctx, w); err != nil {
				if errors.Is(err, pkg.ErrTransactionAlreadyMinted) {
					continue
				}
				return err
			}
			continue
		}

		sig, seqNum, err := bridge.wallet.CreatePaymentAndReturnSignature(ctx, w.Target, w.Amount, w.ID)
		if err != nil {
			logger.Warn().Err(err).Msg("failed to create Stellar signature for withdraw proposal, skipping")
			continue
		}

		burnMetas = append(burnMetas, burnMeta{event: w, index: len(burnProposals)})
		burnProposals = append(burnProposals, subpkg.BurnProposal{
			TxID:           w.ID,
			Target:         w.Target,
			Amount:         new(big.Int).SetUint64(w.Amount),
			Signature:      sig,
			StellarAddress: bridge.wallet.GetKeypair().Address(),
			SequenceNumber: seqNum,
		})
	}

	// Step 6: Build refund proposals.
	var refundProposals []subpkg.RefundProposal
	type refundMeta struct {
		item  refundItem
		index int // index into refundProposals (offset by len(burnProposals) in the batch)
	}
	var refundMetas []refundMeta

	for _, r := range allRefunds {
		logger := log.Logger.With().Str("trace_id", r.Hash).Logger()

		refunded, err := bridge.subClient.IsRefundedAlready(r.Hash)
		if err != nil {
			return err
		}
		if refunded {
			logger.Info().
				Str("event_action", "refund_skipped").
				Str("event_kind", "event").
				Str("category", "refund").
				Msg("the transaction has already been refunded")
			continue
		}

		sig, seqNum, err := bridge.wallet.CreateRefundAndReturnSignature(ctx, r.Target, r.Amount, r.Hash)
		if err != nil {
			logger.Warn().Err(err).Msg("failed to create Stellar signature for refund proposal, skipping")
			continue
		}

		refundMetas = append(refundMetas, refundMeta{item: r, index: len(refundProposals)})
		refundProposals = append(refundProposals, subpkg.RefundProposal{
			TxHash:         r.Hash,
			Target:         r.Target,
			Amount:         int64(r.Amount),
			Signature:      sig,
			StellarAddress: bridge.wallet.GetKeypair().Address(),
			SequenceNumber: seqNum,
		})
	}

	// Step 7: Submit unified force_batch.
	if len(burnProposals) == 0 && len(refundProposals) == 0 {
		return nil
	}

	result, err := bridge.subClient.BatchProposeAll(ctx, burnProposals, refundProposals)
	if err != nil {
		// Wholesale RPC failure — sequential fallback would also fail.
		// BurnTransactionExpired / RefundTransactionExpired will re-emit and retry.
		log.Warn().Err(err).Msg("force_batch proposal failed; proposals will be retried via expiry events")
		return nil
	}

	// Step 8: Log per-proposal outcomes.
	// Calls are ordered: burns[0..N-1], refunds[N..N+M-1].
	// FailedIndexes is populated only for BatchInterrupted (older runtimes that use
	// Utility.batch); with force_batch we get ItemFailed events without call indices,
	// so FailedIndexes will be empty and we fall back to the FailedCount flag.
	failedSet := make(map[int]bool, len(result.FailedIndexes))
	for _, idx := range result.FailedIndexes {
		failedSet[idx] = true
	}
	batchHadFailures := result.FailedCount > 0

	if batchHadFailures {
		log.Warn().
			Int("failed", result.FailedCount).
			Int("total", len(burnProposals)+len(refundProposals)).
			Msg("some proposals failed within batch (may already be signed or expired)")
	}

	for _, m := range burnMetas {
		if failedSet[m.index] {
			log.Warn().
				Str("event_action", "withdraw_proposal_failed").
				Str("event_kind", "alert").
				Str("category", "withdraw").
				Uint64("tx_id", m.event.ID).
				Msg("withdraw proposal failed within batch")
			continue
		}
		log.Info().
			Str("trace_id", fmt.Sprint(m.event.ID)).
			Str("event_action", "withdraw_proposed").
			Str("event_kind", "event").
			Str("category", "withdraw").
			Bool("batch_had_failures", batchHadFailures && len(result.FailedIndexes) == 0).
			Dict("metadata", zerolog.Dict().
				Uint64("amount", m.event.Amount).
				Str("tx_id", fmt.Sprint(m.event.ID)).
				Str("to", m.event.Target)).
			Msgf("a withdraw has proposed with the target stellar address of %s", m.event.Target)
	}

	// Refund indices in the batch are offset by the number of burn proposals.
	burnOffset := len(burnProposals)
	for _, m := range refundMetas {
		batchIdx := burnOffset + m.index
		if failedSet[batchIdx] {
			log.Warn().
				Str("event_action", "refund_proposal_failed").
				Str("event_kind", "alert").
				Str("category", "refund").
				Str("tx_hash", m.item.Hash).
				Msg("refund proposal failed within batch")
			continue
		}
		log.Info().
			Str("trace_id", m.item.Hash).
			Str("event_action", "refund_proposed").
			Str("event_kind", "event").
			Str("category", "refund").
			Bool("batch_had_failures", batchHadFailures && len(result.FailedIndexes) == 0).
			Dict("metadata", zerolog.Dict().
				Uint64("amount", m.item.Amount).
				Str("tx_hash", m.item.Hash).
				Str("to", m.item.Target)).
			Msgf("a refund has proposed for target stellar address %s", m.item.Target)
	}

	log.Info().
		Str("event_action", "batch_proposal_completed").
		Str("event_kind", "event").
		Str("category", "bridge").
		Int("total", len(burnProposals)+len(refundProposals)).
		Int("succeeded", result.SuccessCount).
		Int("failed", result.FailedCount).
		Msg("batch proposal completed")

	return nil
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

		// Fetch the outgoing transactions page once and reuse it for both lookups.
		// Non-fatal on error: match the reconciler's behavior. Leave tx as PROCESSING;
		// the next BurnTransactionReady event will retry the Horizon lookup.
		// Returning an error here would crash the bridge on every PROCESSING event
		// during a Horizon outage, which is worse than gracefully skipping.
		outgoingPage, err := bridge.wallet.FetchOutgoingTransactionsPage(ctx)
		if err != nil {
			logger.Warn().Err(err).Uint64("tx_id", txID).
				Msg("failed to fetch Horizon transactions for PROCESSING check; will retry on next event")
			return nil
		}

		// Primary check: look for a tx with matching memo (current bridge behaviour)
		if stellarTx := bridge.wallet.FindPaymentByMemoInPage(outgoingPage, txKey); stellarTx != nil {
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
		// NOTE: adding memo to Stellar txs is a breaking change — all validators must be
		// upgraded together. A mixed-version cluster will produce invalid signature sets.
		burnTxForSeq, err := bridge.subClient.GetBurnTransaction(types.U64(txID))
		if err != nil {
			return err
		}
		if stellarTxBySeq := bridge.wallet.FindPaymentBySequenceInPage(outgoingPage, int64(burnTxForSeq.SequenceNumber)); stellarTxBySeq != nil {
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

	// 7. Mark executed on TFChain — must complete before logging withdraw_completed
	// so that ops logs accurately reflect the full transaction lifecycle.
	if err := bridge.subClient.RetrySetWithdrawExecuted(ctx, txID); err != nil {
		return err
	}

	// 8. Mark COMPLETED in idempotency store
	if err := bridge.idempotency.MarkWithdrawCompleted(txID); err != nil {
		return err
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
