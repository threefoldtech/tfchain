package substrate

import (
	"fmt"
	"math/big"

	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
	"github.com/pkg/errors"
)

var (
	ErrBurnTransactionNotFound   = fmt.Errorf("burn tx not found")
	ErrRefundTransactionNotFound = fmt.Errorf("refund tx not found")
	ErrFailedToDecode            = fmt.Errorf("failed to decode events, skipping")
)

type BurnTransaction struct {
	Block          types.U32          `json:"block"`
	Amount         types.U64          `json:"amount"`
	Source         types.OptionAccountID    `json:"source"`
	Target         string             `json:"target"`
	Signatures     []StellarSignature `json:"signatures"`
	SequenceNumber types.U64          `json:"sequence_number"`
}

func (s *Substrate) ProposeBurnTransactionOrAddSig(identity Identity, txID uint64, target string, amount *big.Int, signature string, stellarAddress string, sequence_number uint64) error {
	cl, meta, err := s.GetClient()
	if err != nil {
		return err
	}

	c, err := types.NewCall(meta, "TFTBridgeModule.propose_burn_transaction_or_add_sig",
		txID, target, types.U64(amount.Uint64()), signature, stellarAddress, sequence_number,
	)

	if err != nil {
		return errors.Wrap(err, "failed to create call")
	}

	_, err = s.Call(cl, meta, identity, c)
	if err != nil {
		return errors.Wrap(err, "failed to propose burn transaction")
	}

	return nil
}

func (s *Substrate) SetBurnTransactionExecuted(identity Identity, txID uint64) error {
	cl, meta, err := s.GetClient()
	if err != nil {
		return err
	}

	c, err := types.NewCall(meta, "TFTBridgeModule.set_burn_transaction_executed", txID)

	if err != nil {
		return errors.Wrap(err, "failed to create call")
	}

	_, err = s.Call(cl, meta, identity, c)
	if err != nil {
		return errors.Wrap(err, "failed to set burn transaction executed")
	}

	return nil
}

func (s *Substrate) GetBurnTransaction(burnTransactionID types.U64) (*BurnTransaction, error) {
	cl, meta, err := s.GetClient()
	if err != nil {
		return nil, err
	}

	bytes, err := Encode(burnTransactionID)
	if err != nil {
		return nil, errors.Wrap(err, "substrate: encoding error building query arguments")
	}

	var burnTx BurnTransaction
	key, err := types.CreateStorageKey(meta, "TFTBridgeModule", "BurnTransactions", bytes, nil)
	if err != nil {
		err = errors.Wrap(err, "failed to create storage key")
		return nil, err
	}

	ok, err := cl.RPC.State.GetStorageLatest(key, &burnTx)
	if err != nil {
		return nil, err
	}

	if !ok {
		return nil, ErrBurnTransactionNotFound
	}

	return &burnTx, nil
}

func (s *Substrate) IsBurnedAlready(burnTransactionID types.U64) (exists bool, err error) {
	cl, meta, err := s.GetClient()
	if err != nil {
		return false, err
	}

	bytes, err := Encode(burnTransactionID)
	if err != nil {
		return false, errors.Wrap(err, "substrate: encoding error building query arguments")
	}

	var burnTx BurnTransaction
	key, err := types.CreateStorageKey(meta, "TFTBridgeModule", "ExecutedBurnTransactions", bytes, nil)
	if err != nil {
		err = errors.Wrap(err, "failed to create storage key")
		return
	}

	ok, err := cl.RPC.State.GetStorageLatest(key, &burnTx)
	if err != nil {
		return false, err
	}

	if !ok {
		return false, nil
	}

	return true, nil
}
