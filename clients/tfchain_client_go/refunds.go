package substrate

import (
	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
	"github.com/pkg/errors"
)

type RefundTransaction struct {
	Block          types.U32
	Amount         types.U64
	Target         string
	TxHash         string
	Signatures     []StellarSignature
	SequenceNumber types.U64
}

func (s *Substrate) CreateRefundTransactionOrAddSig(identity Identity, tx_hash string, target string, amount int64, signature string, stellarAddress string, sequence_number uint64) error {
	cl, meta, err := s.GetClient()
	if err != nil {
		return err
	}

	c, err := types.NewCall(meta, "TFTBridgeModule.create_refund_transaction_or_add_sig",
		tx_hash, target, types.U64(amount), signature, stellarAddress, sequence_number,
	)

	if err != nil {
		return errors.Wrap(err, "failed to create call")
	}

	_, err = s.Call(cl, meta, identity, c)
	if err != nil {
		return errors.Wrap(err, "failed to create refund transaction")
	}

	return nil
}

func (s *Substrate) SetRefundTransactionExecuted(identity Identity, txHash string) error {
	cl, meta, err := s.GetClient()
	if err != nil {
		return err
	}

	c, err := types.NewCall(meta, "TFTBridgeModule.set_refund_transaction_executed", txHash)

	if err != nil {
		return errors.Wrap(err, "failed to create call")
	}

	_, err = s.Call(cl, meta, identity, c)
	if err != nil {
		return errors.Wrap(err, "failed to create refund transaction")
	}

	return nil
}

func (s *Substrate) IsRefundedAlready(txHash string) (exists bool, err error) {
	cl, meta, err := s.GetClient()
	if err != nil {
		return false, err
	}

	bytes, err := types.Encode(txHash)
	if err != nil {
		return false, errors.Wrap(err, "substrate: encoding error building query arguments")
	}

	var refundTx RefundTransaction
	key, err := types.CreateStorageKey(meta, "TFTBridgeModule", "ExecutedRefundTransactions", bytes, nil)
	if err != nil {
		err = errors.Wrap(err, "failed to create storage key")
		return
	}

	ok, err := cl.RPC.State.GetStorageLatest(key, &refundTx)
	if err != nil {
		return false, err
	}

	if !ok {
		return false, nil
	}

	return true, nil
}

func (s *Substrate) GetRefundTransaction(txHash string) (*RefundTransaction, error) {
	cl, meta, err := s.GetClient()
	if err != nil {
		return nil, err
	}

	bytes, err := types.Encode(txHash)
	if err != nil {
		return nil, errors.Wrap(err, "substrate: encoding error building query arguments")
	}

	var refundTx RefundTransaction
	key, err := types.CreateStorageKey(meta, "TFTBridgeModule", "RefundTransactions", bytes, nil)
	if err != nil {
		err = errors.Wrap(err, "failed to create storage key")
		return nil, err
	}

	ok, err := cl.RPC.State.GetStorageLatest(key, &refundTx)
	if err != nil {
		return nil, err
	}

	if !ok {
		return nil, ErrBurnTransactionNotFound
	}

	return &refundTx, nil
}
