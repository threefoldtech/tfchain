package bridge

import (
	"context"
	"errors"
	"fmt"
	"math/big"

	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
	substrate "github.com/threefoldtech/tfchain/clients/tfchain-client-go"
	"github.com/threefoldtech/tfchain_bridge/pkg"
	subpkg "github.com/threefoldtech/tfchain_bridge/pkg/substrate"
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
	// check if it can hold tft : TODO check trust line TFT limit if it can receive the amount
	if err := bridge.wallet.CheckAccount(withdraw.Target); err != nil {
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
			Uint64("amount", withdraw.Amount).
			Str("tx_id", fmt.Sprint(withdraw.ID)).
			Str("to", withdraw.Target)).
		Msgf("a withdraw has proposed with the target stellar address of %s", withdraw.Target)
	return nil
}

func (bridge *Bridge) handleWithdrawExpired(ctx context.Context, withdrawExpired subpkg.WithdrawExpiredEvent) error {
	logger := log.Logger.With().Str("trace_id", fmt.Sprint(withdrawExpired.ID)).Logger()
	if err := bridge.wallet.CheckAccount(withdrawExpired.Target); err != nil {
		log.Warn().
			Str("event_action", "transfer_failed").
			Str("event_kind", "alert").
			Str("category", "transfer").
			Dict("metadata", zerolog.Dict().
				Str("reason", err.Error())).
				Str("type", "burn").
			Msg("a withdraw failed with no way to refund!") // why the event not have the source address or we don't get it by query tfcahin and refund this?
		return bridge.subClient.RetrySetWithdrawExecuted(ctx, withdrawExpired.ID)
	}

	signature, sequenceNumber, err := bridge.wallet.CreatePaymentAndReturnSignature(ctx, withdrawExpired.Target, withdrawExpired.Amount, withdrawExpired.ID)
	if err != nil {
		return err
	}
	log.Debug().Msgf("stellar account sequence number: %d", sequenceNumber)

	err = bridge.subClient.RetryProposeWithdrawOrAddSig(ctx, withdrawExpired.ID, withdrawExpired.Target, big.NewInt(int64(withdrawExpired.Amount)), signature, bridge.wallet.GetKeypair().Address(), sequenceNumber)
	if err != nil {
		return nil
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
			Str("destination_address", withdrawExpired.Target)).
		Msgf("a withdraw has proposed with the target stellar address of %s", withdrawExpired.Target)
	return nil
}

func (bridge *Bridge) handleWithdrawReady(ctx context.Context, withdrawReady subpkg.WithdrawReadyEvent) error {
	logger := log.Logger.With().Str("trace_id", fmt.Sprint(withdrawReady.ID)).Logger()

	burned, err := bridge.subClient.IsBurnedAlready(types.U64(withdrawReady.ID))
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

	burnTx, err := bridge.subClient.GetBurnTransaction(types.U64(withdrawReady.ID))
	if err != nil {
		return err
	}

	if len(burnTx.Signatures) == 0 {
		return pkg.ErrNoSignatures
	}

	// todo add memo hash
	err = bridge.wallet.CreatePaymentWithSignaturesAndSubmit(ctx, burnTx.Target, uint64(burnTx.Amount), "", burnTx.Signatures, int64(burnTx.SequenceNumber))
	if err != nil {
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

	return bridge.subClient.RetrySetWithdrawExecuted(ctx, withdrawReady.ID)
}

func (bridge *Bridge) handleBadWithdraw(ctx context.Context, withdraw subpkg.WithdrawCreatedEvent) error {
	logger := log.Logger.With().Str("trace_id", fmt.Sprint(withdraw.ID)).Logger()
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
