package substrate

import (
	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
	"github.com/pkg/errors"
)

// GetZosVersion gets the latest version for each network
func (s *Substrate) GetZosVersion() (string, error) {
	cl, meta, err := s.GetClient()
	if err != nil {
		return "", err
	}

	key, err := types.CreateStorageKey(meta, "TfgridModule", "ZosVersion", nil)
	if err != nil {
		return "", errors.Wrap(err, "failed to create substrate query key")
	}

	raw, err := cl.RPC.State.GetStorageRawLatest(key)
	if err != nil {
		return "", errors.Wrap(err, "failed to lookup entity")
	}

	if len(*raw) == 0 {
		return "", errors.Wrap(ErrNotFound, "zos version not found")
	}

	var zosVersion string

	if err := types.Decode(*raw, &zosVersion); err != nil {
		return "", errors.Wrap(err, "failed to load object")
	}

	return zosVersion, nil
}
