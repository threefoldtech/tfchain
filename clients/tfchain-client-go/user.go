package substrate

import (
	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
	"github.com/pkg/errors"
)

// User from substrate
type User struct {
	Versioned
	ID        types.U32 `json:"id"`
	Name      string    `json:"name"`
	CountryID types.U32 `json:"country_id"`
	CityID    types.U32 `json:"city_id"`
	Address   AccountID `json:"address"`
}

// GetUser with id
func (s *Substrate) GetUser(id uint32) (*User, error) {
	cl, meta, err := s.GetClient()
	if err != nil {
		return nil, err
	}

	bytes, err := types.Encode(id)
	if err != nil {
		return nil, errors.Wrap(err, "substrate: encoding error building query arguments")
	}
	key, err := types.CreateStorageKey(meta, "TfgridModule", "Entities", bytes, nil)
	if err != nil {
		return nil, errors.Wrap(err, "failed to create substrate query key")
	}

	raw, err := cl.RPC.State.GetStorageRawLatest(key)
	if err != nil {
		return nil, errors.Wrap(err, "failed to lookup entity")
	}

	if len(*raw) == 0 {
		return nil, errors.Wrap(ErrNotFound, "entity not found")
	}

	version, err := s.getVersion(*raw)
	if err != nil {
		return nil, err
	}

	var user User

	switch version {
	case 1:
		if err := types.Decode(*raw, &user); err != nil {
			return nil, errors.Wrap(err, "failed to load object")
		}
	default:
		return nil, ErrUnknownVersion
	}

	return &user, nil
}
