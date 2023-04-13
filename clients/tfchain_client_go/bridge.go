package substrate

import (
	"fmt"

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
