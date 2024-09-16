package substrate

import (
	"bytes"
	"fmt"

	"github.com/centrifuge/go-substrate-rpc-client/v4/scale"
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
			key, err := decodeSecondKey(c.StorageKey, identity)
			if err != nil {
				return nil, err
			}

			val, err := decodeVecU8Val(c.StorageData)
			if err != nil {
				return nil, errors.Wrapf(err, "failed to decode value %v", string(c.StorageData))
			}

			pairs[string(key)] = string(val)
		}
	}

	return pairs, nil
}

// decodeSecondKey extracts and decodes the second key(Vec<u8>) from the storage key.
func decodeSecondKey(storageKey types.StorageKey, identity Identity) ([]byte, error) {
	// remove 16 bytes(32 in hex) pallet and map prefixes.
	// pallet prefix (8 bytes): twox64(pallet_name)
	// map prefix (8bytes) twox64(map_name)
	prefixLen := 32

	// the storage key contains two keys (AccountID and Vec<u8>)
	// remove the length of the first key(AccountID)
	// the hasher `Blake2_128Concat` includes a 16-byte hash followed by the AccountID
	firstKeyLen := 32 + len(identity.PublicKey())

	offset := prefixLen + firstKeyLen

	if len(storageKey) < offset {
		return nil, errors.New(fmt.Sprintf("failed to decode second key, storage key len should not be less than %d bytes", offset))
	}

	return decodeVecU8Val(storageKey[offset:])
}

// decodeVecU8Val decodes a value of type (Vec<u8>) from SCALE-encoded bytes.
func decodeVecU8Val(encodedData []byte) (data []byte, err error) {
	decoder := scale.NewDecoder(bytes.NewReader(encodedData))
	err = decoder.Decode(&data)

	return
}
