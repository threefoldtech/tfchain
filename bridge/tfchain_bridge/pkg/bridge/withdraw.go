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
	logger := log.Logger.With().Str("span_id", fmt.Sprint(withdraw.ID)).Logger()

	burned, err := bridge.subClient.IsBurnedAlready(types.U64(withdraw.ID))
	if err != nil {
		return err
	}

	if burned {
		logger.Info().
			Str("event_type", "withdraw_skipped").
			Msg("tx is already withdrawn")
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
		Str("type_event", "withdraw_proposed").
		Dict("event", zerolog.Dict().
			Int64("amount", int64(withdraw.Amount)).
			Str("tx_id", fmt.Sprint(withdraw.ID)).
			Str("destination_address", withdraw.Target)).
		Msgf("withdraw proposed. target stellar address: %s", withdraw.Target)

	logger.Info().
		Str("event_type", "transfer_initiated").
		Dict("event", zerolog.Dict().
			Str("type", "burn")).
		Msgf("transfer with id %s initiated", fmt.Sprint(withdraw.ID))
	return nil
}

func (bridge *Bridge) handleWithdrawExpired(ctx context.Context, withdrawExpired subpkg.WithdrawExpiredEvent) error {
	logger := log.Logger.With().Str("span_id", fmt.Sprint(withdrawExpired.ID)).Logger()
	if err := bridge.wallet.CheckAccount(withdrawExpired.Target); err != nil {
		log.Err(err).
			Str("event_type", "refund_failed").
			Msg("setting burn as executed since we have no way to recover...") // why the event not have the source address or we don't get it by query tfcahin and refund this?
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
		Str("type_event", "withdraw_proposed").
		Dict("event", zerolog.Dict().
			Int64("amount", int64(withdrawExpired.Amount)).
			Str("tx_id", fmt.Sprint(withdrawExpired.ID)).
			Str("destination_address", withdrawExpired.Target)).
		Msgf("withdraw proposed. target stellar address: %s", withdrawExpired.Target)

	logger.Info().
		Str("event_type", "transfer_initiated").
		Dict("event", zerolog.Dict().
			Str("type", "burn")).
		Msgf("transfer with id %s initiated", fmt.Sprint(withdrawExpired.ID))
	return nil
}

func (bridge *Bridge) handleWithdrawReady(ctx context.Context, withdrawReady subpkg.WithdrawReadyEvent) error {
	logger := log.Logger.With().Str("span_id", fmt.Sprint(withdrawReady.ID)).Logger()

	burned, err := bridge.subClient.IsBurnedAlready(types.U64(withdrawReady.ID))
	if err != nil {
		return err
	}

	if burned {
		logger.Info().
			Str("event_type", "withdraw_skipped").
			Msg("tx is already withdrawn")
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
		Str("event_type", "withdraw_completed").
		Msg("withdraw completed")
	logger.Info().
		Str("event_type", "transfer_completed").
		Dict("event", zerolog.Dict().
			Str("outcome", "bridged")).
		Msgf("transfer with id %d completed", withdrawReady.ID)

	return bridge.subClient.RetrySetWithdrawExecuted(ctx, withdrawReady.ID)
}

func (bridge *Bridge) handleBadWithdraw(ctx context.Context, withdraw subpkg.WithdrawCreatedEvent) error {
	logger := log.Logger.With().Str("span_id", fmt.Sprint(withdraw.ID)).Logger()
	mintID := fmt.Sprintf("refund-%d", withdraw.ID)

	minted, err := bridge.subClient.IsMintedAlready(mintID)
	if err != nil {
		if !errors.Is(err, substrate.ErrMintTransactionNotFound) {
			return err
		}
	}

	if minted {
		logger.Info().
			Str("event_type", "mint_skipped").
			Msg("transaction is already minted")
		return pkg.ErrTransactionAlreadyMinted
	}

	err = bridge.subClient.RetryProposeMintOrVote(ctx, mintID, substrate.AccountID(withdraw.Source), big.NewInt(int64(withdraw.Amount)))
	if err != nil {
		return err
	}

	logger.Info().
		Str("type_event", "mint_proposed").
		Dict("event", zerolog.Dict().
			Int64("amount", int64(withdraw.Amount)).
			Str("tx_id", fmt.Sprint(withdraw.ID)).
			Str("destination_address", withdraw.Source.ToHexString())).
		Msgf("mint proposed. target substrate address: %s", withdraw.Source.ToHexString())
	return bridge.subClient.RetrySetWithdrawExecuted(ctx, withdraw.ID)
}
