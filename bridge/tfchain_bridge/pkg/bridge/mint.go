package bridge

import (
	"context"
	"math/big"
	"strconv"
	"strings"

	"github.com/pkg/errors"
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
	hProtocol "github.com/stellar/go/protocols/horizon"
	substrate "github.com/threefoldtech/tfchain/clients/tfchain-client-go"
	"github.com/threefoldtech/tfchain_bridge/pkg"
)

// mint handler for stellar
func (bridge *Bridge) mint(ctx context.Context, senders map[string]*big.Int, tx hProtocol.Transaction) error {
	logger := log.Logger.With().Str("trace_id", tx.ID).Logger()

	minted, err := bridge.subClient.IsMintedAlready(tx.Hash)
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

	if len(senders) == 0 {
		return nil
	}
	logger.Info().
		Str("event_action", "transfer_initiated").
		Str("event_kind", "event").
		Str("category", "transfer").
		Dict("metadata", zerolog.Dict().
			Str("type", "deposit")).
		Msg("a transfer has initiated")

	// only one payment in transaction is allowed
	if len(senders) > 1 {
		ctx = context.WithValue(ctx, "refund_reason", "multiple senders found")
		for sender, depositAmount := range senders {
			return bridge.refund(ctx, sender, depositAmount.Int64(), tx) // so how this should refund the multiple senders ?
		}
	}

	var receiver string
	var depositedAmount *big.Int
	for receiv, amount := range senders {
		receiver = receiv
		depositedAmount = amount
	}

	if tx.Memo == "" {
		ctx = context.WithValue(ctx, "refund_reason", "no memo in transaction")
		return bridge.refund(ctx, receiver, depositedAmount.Int64(), tx)
	}

	if tx.MemoType == "return" {
		logger.Debug().Str("tx_id", tx.Hash).Msg("the transaction is being skipped because it contains a return memo")
		// save cursor
		cursor := tx.PagingToken()
		err := bridge.blockPersistency.SaveStellarCursor(cursor)
		if err != nil {
			return errors.Wrap(err, "an error occurred while saving stellar cursor")
		}
		return nil
	}

	// if the deposited amount is lower than the deposit fee, trigger a refund
	if depositedAmount.Cmp(big.NewInt(bridge.depositFee)) <= 0 {
		ctx = context.WithValue(ctx, "refund_reason", "insufficient deposit amount to cover fee")
		return bridge.refund(ctx, receiver, depositedAmount.Int64(), tx)
	}

	destinationSubstrateAddress, err := bridge.getSubstrateAddressFromMemo(tx.Memo)
	if err != nil {
		logger.Debug().Err(err).Msg("there was an issue decoding the memo for the transaction")
		// memo is not formatted correctly, issue a refund
		ctx = context.WithValue(ctx, "refund_reason", "memo is not properly formatted")
		return bridge.refund(ctx, receiver, depositedAmount.Int64(), tx)
	}

	accountID, err := substrate.FromAddress(destinationSubstrateAddress)
	if err != nil {
		return err
	}

	err = bridge.subClient.RetryProposeMintOrVote(ctx, tx.Hash, accountID, depositedAmount)
	if err != nil {
		return err
	}

	logger.Info().
		Str("event_action", "mint_proposed").
		Str("event_kind", "event").
		Str("category", "mint").
		Dict("metadata", zerolog.Dict().
			Int64("amount", depositedAmount.Int64()).
			Str("tx_id", tx.Hash).
			Str("to", destinationSubstrateAddress)).
		Msgf("a mint has proposed with the target substrate address of %s", destinationSubstrateAddress)

	// save cursor
	cursor := tx.PagingToken()
	if err = bridge.blockPersistency.SaveStellarCursor(cursor); err != nil {
		return errors.Wrap(err, "an error occurred while saving stellar cursor")
	}

	return nil
}

func (bridge *Bridge) getSubstrateAddressFromMemo(memo string) (string, error) {
	chunks := strings.Split(memo, "_")
	if len(chunks) != 2 {
		// memo is not formatted correctly, issue a refund
		return "", errors.New("memo text is not correctly formatted")
	}

	id, err := strconv.Atoi(chunks[1])
	if err != nil {
		return "", err
	}

	switch chunks[0] {
	case "twin":
		twin, err := bridge.subClient.GetTwin(uint32(id))
		if err != nil {
			return "", err
		}
		return twin.Account.String(), nil
	case "farm":
		farm, err := bridge.subClient.GetFarm(uint32(id))
		if err != nil {
			return "", err
		}
		twin, err := bridge.subClient.GetTwin(uint32(farm.TwinID))
		if err != nil {
			return "", err
		}
		return twin.Account.String(), nil
	case "node":
		node, err := bridge.subClient.GetNode(uint32(id))
		if err != nil {
			return "", err
		}
		twin, err := bridge.subClient.GetTwin(uint32(node.TwinID))
		if err != nil {
			return "", err
		}
		return twin.Account.String(), nil
	case "entity":
		entity, err := bridge.subClient.GetEntity(uint32(id))
		if err != nil {
			return "", err
		}
		return entity.Account.String(), nil
	default:
		return "", errors.New("grid type not supported")
	}
}
