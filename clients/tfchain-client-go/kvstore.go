package substrate

import (
	"github.com/centrifuge/go-substrate-rpc-client/v4/types"

	"github.com/pkg/errors"
)

func (s *Substrate) KVStoreSet(identity Identity, key string, value string) error {
	cl, meta, err := s.GetClient()
	if err != nil {
		return err
	}

	c, err := types.NewCall(meta, "TFKVStore.set",
		key, value,
	)
	if err != nil {
		return errors.Wrap(err, "failed to create call")
	}

	res, err := s.Call(cl, meta, identity, c)
	if err != nil {
		return errors.Wrap(err, "failed to create contract")
	}

	if err := s.checkForError(res); err != nil {
		return err
	}

	return nil
}

func (s *Substrate) KVStoreDelete(identity Identity, key string) error {
	cl, meta, err := s.GetClient()
	if err != nil {
		return err
	}

	c, err := types.NewCall(meta, "TFKVStore.delete",
		key,
	)
	if err != nil {
		return errors.Wrap(err, "failed to create call")
	}

	res, err := s.Call(cl, meta, identity, c)
	if err != nil {
		return errors.Wrap(err, "failed to create contract")
	}

	if err := s.checkForError(res); err != nil {
		return err
	}
	return nil
}

func (s *Substrate) KVStoreGet(identity Identity, key string) ([]byte, error) {
	cl, meta, err := s.GetClient()
	if err != nil {
		return nil, err
	}

	bytes, err := Encode(key)
	if err != nil {
		return nil, err
	}

	storageKey, err := types.CreateStorageKey(meta, "TFKVStore", "TFKVStore", identity.PublicKey(), bytes)
	if err != nil {
		return nil, errors.Wrap(err, "failed to create substrate query key")
	}

	var value []byte
	ok, err := cl.RPC.State.GetStorageLatest(storageKey, &value)
	if err != nil {
		return nil, errors.Wrap(err, "failed to lookup entity")
	}

	if !ok {
		return nil, errors.Wrap(ErrNotFound, "key not found")
	}

	return value, nil
}

type Val struct {
	Id  string
	Key string
}

func (s *Substrate) KVStoreList(identity Identity) (map[string]string, error) {
	cl, meta, err := s.GetClient()
	if err != nil {
		return nil, err
	}

	storageKey, err := types.CreateStorageKey(meta, "TFKVStore", "TFKVStore", identity.PublicKey())
	if err != nil {
		return nil, errors.Wrap(err, "failed to create substrate query key")
	}

	keys, err := cl.RPC.State.GetKeysLatest(storageKey)
	if err != nil {
		return nil, errors.Wrap(err, "failed to lookup entity")
	}

	query, err := cl.RPC.State.QueryStorageAtLatest(keys)
	if err != nil {
		return nil, err
	}

	pairs := make(map[string]string)
	for _, q := range query {
		for _, c := range q.Changes {
			var key, val []byte

			key, err = decodeSecondKey(c.StorageKey, identity)
			if err != nil {
				return nil, err
			}

			err = Decode(c.StorageData, &val)
			if err != nil {
				return nil, errors.Wrapf(err, "failed to decode value %v", string(c.StorageData))
			}

			pairs[string(key)] = string(val)
		}
	}

	return pairs, nil
}
