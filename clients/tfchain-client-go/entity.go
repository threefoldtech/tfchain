package substrate

import (
	"fmt"

	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
	"github.com/pkg/errors"
)

// Entity type
type Entity struct {
	Versioned
	ID      types.U32 `json:"id"`
	Name    string    `json:"name"`
	Account AccountID `json:"account_id"`
	Country string    `json:"country"`
	City    string    `json:"city"`
}

// GetEntity gets a entity with ID
func (s *Substrate) GetEntity(id uint32) (*Entity, error) {
	cl, meta, err := s.GetClient()
	if err != nil {
		return nil, err
	}

	bytes, err := Encode(id)
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

	var entity Entity

	switch version {
	case 1:
		if err := Decode(*raw, &entity); err != nil {
			return nil, errors.Wrap(err, "failed to load object")
		}
	default:
		return nil, ErrUnknownVersion
	}

	return &entity, nil
}

// GetEntityIDByName gets an entity ID by name
func (s *Substrate) GetEntityIDByName(name string) (uint32, error) {
	cl, meta, err := s.GetClient()
	if err != nil {
		return 0, err
	}

	bytes, err := Encode(name)
	if err != nil {
		return 0, errors.Wrap(err, "substrate: encoding error building query arguments")
	}

	key, err := types.CreateStorageKey(meta, "TfgridModule", "EntityIdByName", bytes, nil)
	if err != nil {
		return 0, errors.Wrap(err, "failed to create substrate query key")
	}

	var id types.U32
	ok, err := cl.RPC.State.GetStorageLatest(key, &id)
	if err != nil {
		return 0, errors.Wrap(err, "failed to lookup entity")
	}

	if !ok || id == 0 {
		return 0, errors.Wrap(ErrNotFound, "entity not found")
	}

	return uint32(id), nil
}

// GetEntityIDByAccountID gets an entity ID by account ID
func (s *Substrate) GetEntityIDByByPubKey(pk []byte) (uint32, error) {
	cl, meta, err := s.GetClient()
	if err != nil {
		return 0, err
	}

	key, err := types.CreateStorageKey(meta, "TfgridModule", "EntityIdByAccountID", pk, nil)
	if err != nil {
		return 0, errors.Wrap(err, "failed to create substrate query key")
	}

	var id types.U32
	ok, err := cl.RPC.State.GetStorageLatest(key, &id)
	if err != nil {
		return 0, errors.Wrap(err, "failed to lookup entity")
	}

	if !ok || id == 0 {
		return 0, errors.Wrap(ErrNotFound, "entity not found")
	}

	return uint32(id), nil
}

// CreateEntity creates an entity
func (s *Substrate) CreateEntity(identity Identity, name string, account []byte, country string, city string, signature []byte) error {
	cl, meta, err := s.GetClient()
	if err != nil {
		return err
	}

	if name == "" {
		return fmt.Errorf("name cannot be empty")
	}

	c, err := types.NewCall(meta, "TfgridModule.create_entity",
		account, name, country, city, signature,
	)

	if err != nil {
		return errors.Wrap(err, "failed to create call")
	}

	if _, err := s.Call(cl, meta, identity, c); err != nil {
		return errors.Wrap(err, "failed to create entity")
	}

	return nil
}
