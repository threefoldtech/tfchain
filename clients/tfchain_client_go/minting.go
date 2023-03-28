package substrate

import (
	"fmt"
	"math/big"

	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
	"github.com/pkg/errors"
)

var (
	ErrMintTransactionNotFound = fmt.Errorf("mint tx not found")
)

type MintTransaction struct {
	Amount types.U64
	Target types.AccountID
	Block  types.U32
	Votes  types.U32
}

func (s *Substrate) IsMintedAlready(mintTxID string) (exists bool, err error) {
	cl, meta, err := s.getClient()
	if err != nil {
		return false, err
	}

	bytes, err := types.Encode(mintTxID)
	if err != nil {
		return false, errors.Wrap(err, "substrate: encoding error building query arguments")
	}

	var mintTX MintTransaction
	key, err := types.CreateStorageKey(meta, "TFTBridgeModule", "ExecutedMintTransactions", bytes, nil)
	if err != nil {
		err = errors.Wrap(err, "failed to create storage key")
		return
	}

	ok, err := cl.RPC.State.GetStorageLatest(key, &mintTX)
	if err != nil {
		return false, err
	}

	if !ok {
		return false, ErrMintTransactionNotFound
	}

	return true, nil
}

func (s *Substrate) ProposeOrVoteMintTransaction(identity Identity, txID string, target AccountID, amount *big.Int) error {
	cl, meta, err := s.getClient()
	if err != nil {
		return err
	}

	c, err := types.NewCall(meta, "TFTBridgeModule.propose_or_vote_mint_transaction",
		txID, target, types.U64(amount.Uint64()),
	)

	if err != nil {
		return errors.Wrap(err, "failed to create call")
	}

	_, err = s.Call(cl, meta, identity, c)
	if err != nil {
		return errors.Wrap(err, "failed to propose mint transaction")
	}

	return nil
}
