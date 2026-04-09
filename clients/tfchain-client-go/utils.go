package substrate

import (
	"fmt"
	"time"

	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
	"github.com/pkg/errors"
	"github.com/rs/zerolog/log"
	"github.com/vedhavyas/go-subkey"
	"golang.org/x/crypto/blake2b"
)

var (
	ErrIsUsurped = fmt.Errorf("is usurped")
	Gigabyte     = 1024 * 1024 * 1024
)


type CallResponse struct {
	Hash     types.Hash
	Events   *EventRecords
	Block    *types.SignedBlock
	Identity Identity
}

// Sign signs data with the private key under the given derivation path, returning the signature. Requires the subkey
// command to be in path
func signBytes(data []byte, privateKeyURI string, scheme subkey.Scheme) ([]byte, error) {
	// if data is longer than 256 bytes, hash it first
	if len(data) > 256 {
		h := blake2b.Sum256(data)
		data = h[:]
	}

	kyr, err := subkey.DeriveKeyPair(scheme, privateKeyURI)
	if err != nil {
		return nil, err
	}

	signature, err := kyr.Sign(data)
	if err != nil {
		return nil, err
	}

	return signature, nil
}

// Sign adds a signature to the extrinsic
func (s *Substrate) sign(e *types.Extrinsic, signer Identity, o types.SignatureOptions) error {
	if e.Type() != types.ExtrinsicVersion4 {
		return fmt.Errorf("unsupported extrinsic version: %v (isSigned: %v, type: %v)", e.Version, e.IsSigned(), e.Type())
	}

	mb, err := Encode(e.Method)
	if err != nil {
		return err
	}

	era := o.Era
	if !o.Era.IsMortalEra {
		era = types.ExtrinsicEra{IsImmortalEra: true}
	}

	payload := types.ExtrinsicPayloadV4{
		ExtrinsicPayloadV3: types.ExtrinsicPayloadV3{
			Method:      mb,
			Era:         era,
			Nonce:       o.Nonce,
			Tip:         o.Tip,
			SpecVersion: o.SpecVersion,
			GenesisHash: o.GenesisHash,
			BlockHash:   o.BlockHash,
		},
		TransactionVersion: o.TransactionVersion,
	}

	signerPubKey, err := types.NewMultiAddressFromAccountID(signer.PublicKey())
	if err != nil {
		return err
	}

	b, err := Encode(payload)
	if err != nil {
		return err
	}

	sig, err := signer.Sign(b)
	if err != nil {
		return err
	}
	msig := signer.MultiSignature(sig)
	extSig := types.ExtrinsicSignatureV4{
		Signer:    signerPubKey,
		Signature: msig,
		Era:       era,
		Nonce:     o.Nonce,
		Tip:       o.Tip,
	}

	e.Signature = extSig

	// mark the extrinsic as signed
	e.Version |= types.ExtrinsicBitSigned

	return nil
}

// Call call this extrinsic and retry if Usurped
func (s *Substrate) Call(cl Conn, meta Meta, identity Identity, call types.Call) (response *CallResponse, err error) {
	for {
		hash, err := s.CallOnce(cl, meta, identity, call)

		if errors.Is(err, ErrIsUsurped) {
			continue
		}

		if err != nil {
			return nil, err
		}

		events, block, err := s.getEventRecords(cl, meta, hash)
		if err != nil {
			return nil, errors.Wrapf(err, "error extracting events from block(%s)", hash.Hex())
		}
		callResponse := CallResponse{
			Hash:     hash,
			Block:    block,
			Events:   events,
			Identity: identity,
		}
		err = s.checkForError(&callResponse)
		if err != nil {
			return nil, err
		}
		return &callResponse, err
	}
}

func (s *Substrate) CallOnce(cl Conn, meta Meta, identity Identity, call types.Call) (hash types.Hash, err error) {
	// Create the extrinsic
	ext := types.NewExtrinsic(call)

	genesisHash, err := cl.RPC.Chain.GetBlockHash(0)
	if err != nil {
		return hash, errors.Wrap(err, "failed to get genesisHash")
	}

	rv, err := cl.RPC.State.GetRuntimeVersionLatest()
	if err != nil {
		return hash, err
	}

	// node.Address =identity.PublicKey
	account, err := s.getAccount(cl, meta, identity)
	if err != nil {
		return hash, errors.Wrap(err, "failed to get account")
	}

	o := types.SignatureOptions{
		BlockHash:          genesisHash,
		Era:                types.ExtrinsicEra{IsMortalEra: false},
		GenesisHash:        genesisHash,
		Nonce:              types.NewUCompactFromUInt(uint64(account.Nonce)),
		SpecVersion:        rv.SpecVersion,
		Tip:                types.NewUCompactFromUInt(0),
		TransactionVersion: rv.TransactionVersion,
	}

	err = s.sign(&ext, identity, o)
	if err != nil {
		return hash, errors.Wrap(err, "failed to sign")
	}

	// Send the extrinsic
	sub, err := cl.RPC.Author.SubmitAndWatchExtrinsic(ext)
	if err != nil {
		return hash, errors.Wrap(err, "failed to submit extrinsic")
	}

	defer sub.Unsubscribe()

	ch := sub.Chan()
	ech := sub.Err()

loop:
	for {
		select {
		case err := <-ech:
			return hash, errors.Wrap(err, "error failed on extrinsic status")
		case <-time.After(30 * time.Second):
			return hash, fmt.Errorf("extrinsic timeout waiting for block")
		case event := <-ch:
			if event.IsReady || event.IsBroadcast {
				continue
			} else if event.IsInBlock {
				hash = event.AsInBlock
				break loop
			} else if event.IsFinalized {
				// we shouldn't hit this case
				// any more since InBlock will always
				// happen first we leave it only
				// as a safety net
				hash = event.AsFinalized
				break loop
			} else if event.IsDropped || event.IsInvalid {
				return hash, fmt.Errorf("failed to make call")
			} else if event.IsUsurped {
				return hash, ErrIsUsurped
			} else {
				log.Error().Err(err).Msgf("extrinsic block in an unhandled state: %+v", event)
			}
		}
	}

	return hash, nil
}

func (s *Substrate) getEventRecords(cl Conn, meta Meta, blockHash types.Hash) (*EventRecords, *types.SignedBlock, error) {
	key, err := types.CreateStorageKey(meta, "System", "Events", nil, nil)
	if err != nil {
		return nil, nil, errors.Wrap(err, "failed to create storage key")
	}

	raw, err := cl.RPC.State.GetStorageRaw(key, blockHash)
	if err != nil {
		return nil, nil, errors.Wrap(err, "failed to get raw storage")
	}

	block, err := cl.RPC.Chain.GetBlock(blockHash)
	if err != nil {
		return nil, nil, errors.Wrap(err, "failed to get block")
	}

	events := EventRecords{}
	err = types.EventRecordsRaw(*raw).DecodeEventRecords(meta, &events)
	if err != nil {
		return nil, nil, errors.Wrap(err, "failed to decode event")
	}

	return &events, block, nil
}

func (s *Substrate) getServiceContractIdsFromEvents(callResponse *CallResponse) ([]uint64, error) {
	var serviceContractIDs []uint64
	twinID, err := s.GetTwinByPubKey(callResponse.Identity.PublicKey())
	if err != nil {
		return serviceContractIDs, err
	}
	if len(callResponse.Events.SmartContractModule_ServiceContractCreated) > 0 {
		for _, e := range callResponse.Events.SmartContractModule_ServiceContractCreated {
			if e.ServiceContract.ServiceTwinID == types.U32(twinID) ||
				e.ServiceContract.ConsumerTwinID == types.U32(twinID) {
				serviceContractIDs = append(serviceContractIDs, uint64(e.ServiceContract.ServiceContractID))
			}
		}
	}

	return serviceContractIDs, nil
}

func (s *Substrate) checkForError(callResponse *CallResponse) error {
	if len(callResponse.Events.System_ExtrinsicFailed) > 0 {
		for _, e := range callResponse.Events.System_ExtrinsicFailed {
			who := callResponse.Block.Block.Extrinsics[e.Phase.AsApplyExtrinsic].Signature.Signer.AsID
			accId, err := types.NewAccountID(callResponse.Identity.PublicKey())
			if err != nil {
				return err
			}
			if *accId == who {
				metaErr, err := s.meta.FindError(
					types.U8(e.DispatchError.ModuleError.Index),
					e.DispatchError.ModuleError.Error,
				)
				if err != nil {
					return fmt.Errorf("module %d error index %v: %w",
						e.DispatchError.ModuleError.Index,
						e.DispatchError.ModuleError.Error,
						err,
					)
				}
				return errors.New(metaErr.Name)
			}
		}
	}

	return nil
}

// decodeSecondKey extracts and decodes the second key(Vec<u8>) from the storage key.
func decodeSecondKey(storageKey types.StorageKey, identity Identity) (key []byte, err error) {
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

	err = Decode(storageKey[offset:], &key)
	return key, err
}
