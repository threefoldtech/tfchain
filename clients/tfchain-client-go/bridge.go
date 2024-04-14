package substrate

import (
	"fmt"
	"math/big"

	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
	"github.com/pkg/errors"
)

var (
	errValidatorNotFound = fmt.Errorf("validator not found")
)

func (s *Substrate) IsValidator(identity Identity) (exists bool, err error) {
	cl, meta, err := s.GetClient()
	if err != nil {
		return false, err
	}

	var validators []AccountID
	key, err := types.CreateStorageKey(meta, "TFTBridgeModule", "Validators")
	if err != nil {
		err = errors.Wrap(err, "failed to create storage key")
		return
	}

	ok, err := cl.RPC.State.GetStorageLatest(key, &validators)
	if err != nil || !ok {
		if !ok {
			return false, errValidatorNotFound
		}

		return
	}

	exists = false
	for _, validator := range validators {
		if validator.String() == identity.Address() {
			return true, nil
		}
	}

	return
}

func (s *Substrate) SwapToStellar(identity Identity, targetStellarAddress string, amount big.Int) error {
	cl, meta, err := s.GetClient()
	if err != nil {
		return err
	}

	c, err := types.NewCall(meta, "TFTBridgeModule.swap_to_stellar",
		targetStellarAddress, types.NewU128(amount),
	)

	if err != nil {
		return errors.Wrap(err, "failed to create call")
	}

	_, err = s.Call(cl, meta, identity, c)
	if err != nil {
		return errors.Wrap(err, "failed to swap to stellar")
	}

	return nil
}
