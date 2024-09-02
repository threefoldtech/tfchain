package substrate

import (
	"math/big"

	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
	"github.com/pkg/errors"
)

func (s *Substrate) Transfer(identity Identity, amount uint64, destination AccountID) error {
	cl, meta, err := s.GetClient()
	if err != nil {
		return err
	}

	dest, err := types.NewMultiAddressFromAccountID(destination[:])
	if err != nil {
		return err
	}
	bal := big.NewInt(int64(amount))

	c, err := types.NewCall(meta, "Balances.transfer_keep_alive", dest, types.NewUCompact(bal))
	if err != nil {
		panic(err)
	}

	if err != nil {
		return errors.Wrap(err, "failed to create call")
	}

	_, err = s.Call(cl, meta, identity, c)
	if err != nil {
		return errors.Wrap(err, "failed to transfer")
	}

	return nil
}
