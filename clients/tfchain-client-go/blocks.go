package substrate

import (
	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
	"github.com/pkg/errors"
)

func (s *Substrate) GetCurrentHeight() (uint32, error) {
	cl, meta, err := s.GetClient()
	if err != nil {
		return 0, err
	}

	var blockNumber uint32
	key, err := types.CreateStorageKey(meta, "System", "Number", nil)
	if err != nil {
		err = errors.Wrap(err, "failed to create storage key")
		return 0, err
	}

	ok, err := cl.RPC.State.GetStorageLatest(key, &blockNumber)
	if err != nil {
		return 0, err
	}

	if !ok {
		return 0, errors.New("block number not found")
	}

	return blockNumber, nil
}

func (s *Substrate) FetchEventsForBlockRange(start uint32, end uint32) (types.StorageKey, []types.StorageChangeSet, error) {
	cl, meta, err := s.GetClient()
	if err != nil {
		return nil, nil, err
	}

	key, err := types.CreateStorageKey(meta, "System", "Events", nil)
	if err != nil {
		return key, nil, err
	}

	lbh, err := cl.RPC.Chain.GetBlockHash(uint64(start))
	if err != nil {
		return key, nil, err
	}

	uph, err := cl.RPC.Chain.GetBlockHash(uint64(end))
	if err != nil {
		return key, nil, err
	}

	rawSet, err := cl.RPC.State.QueryStorage([]types.StorageKey{key}, lbh, uph)
	if err != nil {
		return key, nil, err
	}

	return key, rawSet, nil
}

func (s *Substrate) GetEventsForBlock(start uint32) (*EventRecords, error) {
	cl, _, err := s.GetClient()
	if err != nil {
		return nil, err
	}

	block, err := cl.RPC.Chain.GetBlockHash(uint64(start))
	if err != nil {
		return nil, err
	}
	meta, err := cl.RPC.State.GetMetadata(block)
	if err != nil {
		return nil, err
	}

	key, err := types.CreateStorageKey(meta, "System", "Events", nil)
	if err != nil {
		return nil, err
	}

	var storageData types.StorageDataRaw
	ok, err := cl.RPC.State.GetStorage(key, &storageData, block)
	if err != nil {
		return nil, err
	}

	if !ok {
		return nil, errors.New("failed to get storage")
	}

	events := EventRecords{}
	err = types.EventRecordsRaw(storageData).DecodeEventRecords(meta, &events)
	if err != nil {
		return nil, err
	}

	return &events, nil
}

func (s *Substrate) GetBlock(block types.Hash) (*types.SignedBlock, error) {
	cl, _, err := s.GetClient()
	if err != nil {
		return nil, err
	}

	return cl.RPC.Chain.GetBlock(block)
}

func (s *Substrate) GetBlockByNumber(blockNumber types.U32) (*types.SignedBlock, error) {
	cl, _, err := s.GetClient()
	if err != nil {
		return nil, err
	}

	hash, err := cl.RPC.Chain.GetBlockHash(uint64(blockNumber))
	if err != nil {
		return nil, err
	}

	return cl.RPC.Chain.GetBlock(hash)
}
