package substrate

import (
	"encoding/binary"
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

// map from module index to error list
// https://github.com/threefoldtech/tfchain/blob/development/substrate-node/runtime/src/lib.rs#L701
var moduleErrors = [][]string{
	nil,                       // System
	nil,                       // RandomnessCollectiveFlip
	nil,                       // Timestamp
	nil,                       // Balances
	nil,                       // ValidatorSet
	nil,                       // Session
	nil,                       // Aura
	nil,                       // Grandpa
	nil,                       // TransactionPayment
	nil,                       // Sudo
	nil,                       // Authorship
	tfgridModuleErrors,        // TfgridModule
	smartContractModuleErrors, // SmartContractModule
	tftBridgeModuleErrors,     // TFTBridgeModule
	nil,                       // TFTPriceModule
	nil,                       // Scheduler
	nil,                       // BurningModule
	nil,                       // TFKVStore
	nil,                       // Council
	nil,                       // CouncilMembership
	nil,                       // RuntimeUpgrade
	nil,                       // Validator
	nil,                       // Dao
	nil,                       // Utility
}

// https://github.com/threefoldtech/tfchain_pallets/blob/bc9c5d322463aaf735212e428da4ea32b117dc24/pallet-smart-contract/src/lib.rs#L58
var smartContractModuleErrors = []string{
	"TwinNotExists",
	"NodeNotExists",
	"FarmNotExists",
	"FarmHasNotEnoughPublicIPs",
	"FarmHasNotEnoughPublicIPsFree",
	"FailedToReserveIP",
	"FailedToFreeIPs",
	"ContractNotExists",
	"TwinNotAuthorizedToUpdateContract",
	"TwinNotAuthorizedToCancelContract",
	"NodeNotAuthorizedToDeployContract",
	"NodeNotAuthorizedToComputeReport",
	"PricingPolicyNotExists",
	"ContractIsNotUnique",
	"ContractWrongBillingLoopIndex",
	"NameExists",
	"NameNotValid",
	"InvalidContractType",
	"TFTPriceValueError",
	"NotEnoughResourcesOnNode",
	"NodeNotAuthorizedToReportResources",
	"MethodIsDeprecated",
	"NodeHasActiveContracts",
	"NodeHasRentContract",
	"NodeIsNotDedicated",
	"NodeNotAvailableToDeploy",
	"CannotUpdateContractInGraceState",
	"NumOverflow",
	"OffchainSignedTxCannotSign",
	"OffchainSignedTxAlreadySent",
	"OffchainSignedTxNoLocalAccountAvailable",
	"NameContractNameTooShort",
	"NameContractNameTooLong",
	"InvalidProviderConfiguration",
	"NoSuchSolutionProvider",
	"SolutionProviderNotApproved",
	"TwinNotAuthorized",
	"ServiceContractNotExists",
	"ServiceContractCreationNotAllowed",
	"ServiceContractModificationNotAllowed",
	"ServiceContractApprovalNotAllowed",
	"ServiceContractRejectionNotAllowed",
	"ServiceContractBillingNotApprovedByBoth",
	"ServiceContractBillingVariableAmountTooHigh",
	"ServiceContractBillMetadataTooLong",
	"ServiceContractMetadataTooLong",
	"ServiceContractNotEnoughFundsToPayBill",
	"CanOnlyIncreaseFrequency",
	"IsNotAnAuthority",
	"WrongAuthority",
	"UnauthorizedToChangeSolutionProviderId",
}

// https://github.com/threefoldtech/tfchain/blob/development/substrate-node/pallets/pallet-smart-contract/src/lib.rs#L321
var tfgridModuleErrors = []string{
	"NoneValue",
	"StorageOverflow",
	"CannotCreateNode",
	"NodeNotExists",
	"NodeWithTwinIdExists",
	"CannotDeleteNode",
	"NodeDeleteNotAuthorized",
	"NodeUpdateNotAuthorized",
	"FarmExists",
	"FarmNotExists",
	"CannotCreateFarmWrongTwin",
	"CannotUpdateFarmWrongTwin",
	"CannotDeleteFarm",
	"CannotDeleteFarmWithPublicIPs",
	"CannotDeleteFarmWithNodesAssigned",
	"CannotDeleteFarmWrongTwin",
	"IpExists",
	"IpNotExists",
	"EntityWithNameExists",
	"EntityWithPubkeyExists",
	"EntityNotExists",
	"EntitySignatureDoesNotMatch",
	"EntityWithSignatureAlreadyExists",
	"CannotUpdateEntity",
	"CannotDeleteEntity",
	"SignatureLengthIsIncorrect",
	"TwinExists",
	"TwinNotExists",
	"TwinWithPubkeyExists",
	"CannotCreateTwin",
	"UnauthorizedToUpdateTwin",
	"TwinCannotBoundToItself",
	"PricingPolicyExists",
	"PricingPolicyNotExists",
	"PricingPolicyWithDifferentIdExists",
	"CertificationCodeExists",
	"FarmingPolicyAlreadyExists",
	"FarmPayoutAdressAlreadyRegistered",
	"FarmerDoesNotHaveEnoughFunds",
	"UserDidNotSignTermsAndConditions",
	"FarmerDidNotSignTermsAndConditions",
	"FarmerNotAuthorized",
	"InvalidFarmName",
	"AlreadyCertifier",
	"NotCertifier",
	"NotAllowedToCertifyNode",
	"FarmingPolicyNotExists",
	"RelayTooShort",
	"RelayTooLong",
	"InvalidRelay",
	"FarmNameTooShort",
	"FarmNameTooLong",
	"InvalidPublicIP",
	"PublicIPTooShort",
	"PublicIPTooLong",
	"GatewayIPTooShort",
	"GatewayIPTooLong",
	"IP4TooShort",
	"IP4TooLong",
	"InvalidIP4",
	"GW4TooShort",
	"GW4TooLong",
	"InvalidGW4",
	"IP6TooShort",
	"IP6TooLong",
	"InvalidIP6",
	"GW6TooShort",
	"GW6TooLong",
	"InvalidGW6",
	"DomainTooShort",
	"DomainTooLong",
	"InvalidDomain",
	"MethodIsDeprecated",
	"InterfaceNameTooShort",
	"InterfaceNameTooLong",
	"InvalidInterfaceName",
	"InterfaceMacTooShort",
	"InterfaceMacTooLong",
	"InvalidMacAddress",
	"InterfaceIpTooShort",
	"InterfaceIpTooLong",
	"InvalidInterfaceIP",
	"InvalidZosVersion",
	"FarmingPolicyExpired",
	"InvalidHRUInput",
	"InvalidSRUInput",
	"InvalidCRUInput",
	"InvalidMRUInput",
	"LatitudeInputTooShort",
	"LatitudeInputTooLong",
	"InvalidLatitudeInput",
	"LongitudeInputTooShort",
	"LongitudeInputTooLong",
	"InvalidLongitudeInput",
	"CountryNameTooShort",
	"CountryNameTooLong",
	"InvalidCountryName",
	"CityNameTooShort",
	"CityNameTooLong",
	"InvalidCityName",
	"InvalidCountryCityPair",
	"SerialNumberTooShort",
	"SerialNumberTooLong",
	"InvalidSerialNumber",
	"DocumentLinkInputTooShort",
	"DocumentLinkInputTooLong",
	"InvalidDocumentLinkInput",
	"DocumentHashInputTooShort",
	"DocumentHashInputTooLong",
	"InvalidDocumentHashInput",
	"InvalidPublicConfig",
	"UnauthorizedToChangePowerTarget",
	"InvalidRelayAddress",
	"InvalidTimestampHint",
}

var tftBridgeModuleErrors = []string{
	"ValidatorExists",
	"ValidatorNotExists",
	"TransactionValidatorExists",
	"TransactionValidatorNotExists",
	"MintTransactionExists",
	"MintTransactionAlreadyExecuted",
	"MintTransactionNotExists",
	"BurnTransactionExists",
	"BurnTransactionNotExists",
	"BurnSignatureExists",
	"EnoughBurnSignaturesPresent",
	"RefundSignatureExists",
	"BurnTransactionAlreadyExecuted",
	"RefundTransactionNotExists",
	"RefundTransactionAlreadyExecuted",
	"EnoughRefundSignaturesPresent",
	"NotEnoughBalanceToSwap",
	"AmountIsLessThanWithdrawFee",
	"AmountIsLessThanDepositFee",
	"WrongParametersProvided",
	"InvalidStellarPublicKey",
}

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

	//node.Address =identity.PublicKey
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
			b := make([]byte, 4)
			for i, v := range e.DispatchError.ModuleError.Error {
				b[i] = byte(v)
			}
			errIndex := binary.LittleEndian.Uint32(b[:])
			if *accId == who {
				if int(e.DispatchError.ModuleError.Index) < len(moduleErrors) {
					if int(errIndex) >= len(moduleErrors[e.DispatchError.ModuleError.Index]) || moduleErrors[e.DispatchError.ModuleError.Index] == nil {
						return fmt.Errorf("module error (%d) with unknown code %d occured, please update the module error list", e.DispatchError.ModuleError.Index, e.DispatchError.ModuleError.Error)
					}
					return fmt.Errorf(moduleErrors[e.DispatchError.ModuleError.Index][errIndex])
				} else {
					return fmt.Errorf("unknown module error (%d) with code %d occured, please create the module error list", e.DispatchError.ModuleError.Index, e.DispatchError.ModuleError.Error)
				}
			}
		}
	}

	return nil
}
