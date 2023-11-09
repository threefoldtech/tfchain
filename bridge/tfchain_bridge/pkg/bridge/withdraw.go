package bridge

import (
	"context"
	"errors"
	"fmt"
	"math/big"

	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
	"github.com/rs/zerolog/log"
	substrate "github.com/threefoldtech/tfchain/clients/tfchain-client-go"
	"github.com/threefoldtech/tfchain_bridge/pkg"
	subpkg "github.com/threefoldtech/tfchain_bridge/pkg/substrate"
)

func (bridge *Bridge) handleWithdrawCreated(ctx context.Context, withdraw subpkg.WithdrawCreatedEvent) error {
	burned, err := bridge.subClient.IsBurnedAlready(types.U64(withdraw.ID))
	if err != nil {
		return err
	}

	if burned {
		log.Info().Uint64("ID", uint64(withdraw.ID)).Msgf("tx is burned already, skipping...")
		return pkg.ErrTransactionAlreadyBurned
	}

	if err := bridge.wallet.CheckAccount(withdraw.Target); err != nil {
		return bridge.handleBadWithdraw(ctx, withdraw)
	}

	signature, sequenceNumber, err := bridge.wallet.CreatePaymentAndReturnSignature(ctx, withdraw.Target, withdraw.Amount, withdraw.ID)
	if err != nil {
		return err
	}
	log.Debug().Msgf("stellar account sequence number: %d", sequenceNumber)

	return bridge.subClient.RetryProposeWithdrawOrAddSig(ctx, withdraw.ID, withdraw.Target, big.NewInt(int64(withdraw.Amount)), signature, bridge.wallet.GetKeypair().Address(), sequenceNumber)
}

func (bridge *Bridge) handleWithdrawExpired(ctx context.Context, withdrawExpired subpkg.WithdrawExpiredEvent) error {
	return bridge.handleWithdrawCreated(ctx, subpkg.WithdrawCreatedEvent(withdrawExpired))
}

func (bridge *Bridge) handleWithdrawReady(ctx context.Context, withdrawReady subpkg.WithdrawReadyEvent) error {
	burned, err := bridge.subClient.IsBurnedAlready(types.U64(withdrawReady.ID))
	if err != nil {
		return err
	}

	if burned {
		log.Info().Uint64("ID", uint64(withdrawReady.ID)).Msg("tx is burned already, skipping...")
		return pkg.ErrTransactionAlreadyBurned
	}

	burnTx, err := bridge.subClient.GetBurnTransaction(types.U64(withdrawReady.ID))
	if err != nil {
		return err
	}

	if len(burnTx.Signatures) == 0 {
		log.Info().Msg("found 0 signatures, aborting")
		return pkg.ErrNoSignatures
	}

	// todo add memo hash
	err = bridge.wallet.CreatePaymentWithSignaturesAndSubmit(ctx, burnTx.Target, uint64(burnTx.Amount), "", burnTx.Signatures, int64(burnTx.SequenceNumber))
	if err != nil {
		return err
	}

	return bridge.subClient.RetrySetWithdrawExecuted(ctx, withdrawReady.ID)
}

func (bridge *Bridge) handleBadWithdraw(ctx context.Context, withdraw subpkg.WithdrawCreatedEvent) error {
	log.Info().Uint64("ID", uint64(withdraw.ID)).Msg("tx is an invalid burn transaction, minting on chain again...")
	mintID := fmt.Sprintf("refund-%d", withdraw.ID)

	minted, err := bridge.subClient.IsMintedAlready(mintID)
	if err != nil {
		if !errors.Is(err, substrate.ErrMintTransactionNotFound) {
			return err
		}
	}

	if minted {
		log.Debug().Str("txHash", mintID).Msg("transaction is already minted")
		return pkg.ErrTransactionAlreadyMinted
	}

	log.Info().Str("mintID", mintID).Msg("going to propose mint transaction")
	err = bridge.subClient.RetryProposeMintOrVote(ctx, mintID, substrate.AccountID(withdraw.Source), big.NewInt(int64(withdraw.Amount)))
	if err != nil {
		return err
	}

	log.Info().Uint64("ID", uint64(withdraw.ID)).Msg("setting invalid burn transaction as executed")
	return bridge.subClient.RetrySetWithdrawExecuted(ctx, withdraw.ID)
}
