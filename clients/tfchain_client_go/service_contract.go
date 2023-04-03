package substrate

import (
	"fmt"

	"github.com/centrifuge/go-substrate-rpc-client/v4/scale"
	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
	"github.com/pkg/errors"
)

// ServiceContract struct
type ServiceContract struct {
	ServiceContractID  types.U64
	ServiceTwinID      types.U32
	ConsumerTwinID     types.U32
	BaseFee            types.U64
	VariableFee        types.U64
	Metadata           string
	AcceptedByService  bool
	AcceptedByConsumer bool
	LastBill           types.U64
	State              ServiceContractState
}

// ServiceContractBill struct
type ServiceContractBill struct {
	VariableAmount types.U64
	Window         types.U64
	Metadata       string
}

// ServiceContractState enum
type ServiceContractState struct {
	IsCreated        bool
	IsAgreementReady bool
	IsApprovedByBoth bool
}

// Decode implementation for the ServiceContractState enum type
func (r *ServiceContractState) Decode(decoder scale.Decoder) error {
	b, err := decoder.ReadOneByte()
	if err != nil {
		return err
	}

	switch b {
	case 0:
		r.IsCreated = true
	case 1:
		r.IsAgreementReady = true
	case 2:
		r.IsApprovedByBoth = true
	default:
		return fmt.Errorf("unknown service contract state value")
	}

	return nil
}

// Encode implementation for the ServiceContractState enum type
func (r ServiceContractState) Encode(encoder scale.Encoder) (err error) {
	if r.IsCreated {
		err = encoder.PushByte(0)
	} else if r.IsAgreementReady {
		err = encoder.PushByte(1)
	} else if r.IsApprovedByBoth {
		err = encoder.PushByte(2)
	}

	return
}

// ServiceContractCreate creates a service contract
func (s *Substrate) ServiceContractCreate(identity Identity, service AccountID, consumer AccountID) (uint64, error) {
	cl, meta, err := s.getClient()
	if err != nil {
		return 0, err
	}

	c, err := types.NewCall(meta, "SmartContractModule.service_contract_create",
		service, consumer,
	)

	if err != nil {
		return 0, errors.Wrap(err, "failed to create call")
	}

	callResponse, err := s.Call(cl, meta, identity, c)
	if err != nil {
		return 0, errors.Wrap(err, "failed to create service contract")
	}

	serviceContractIDs, err := s.getServiceContractIdsFromEvents(callResponse)
	if err != nil || len(serviceContractIDs) == 0 {
		return 0, errors.Wrap(err, "failed to get service contract id after creation")
	}

	return serviceContractIDs[len(serviceContractIDs)-1], nil
}

// ServiceContractSetMetadata sets metadata for a service contract
func (s *Substrate) ServiceContractSetMetadata(identity Identity, contract uint64, metadata string) error {
	cl, meta, err := s.getClient()
	if err != nil {
		return err
	}

	c, err := types.NewCall(meta, "SmartContractModule.service_contract_set_metadata",
		contract, metadata,
	)

	if err != nil {
		return errors.Wrap(err, "failed to create call")
	}

	_, err = s.Call(cl, meta, identity, c)
	if err != nil {
		return errors.Wrap(err, "failed to set metadata for service contract")
	}

	return nil
}

// ServiceContractSetFees sets fees for a service contract
func (s *Substrate) ServiceContractSetFees(identity Identity, contract uint64, base_fee uint64, variable_fee uint64) error {
	cl, meta, err := s.getClient()
	if err != nil {
		return err
	}

	c, err := types.NewCall(meta, "SmartContractModule.service_contract_set_fees",
		contract, base_fee, variable_fee,
	)

	if err != nil {
		return errors.Wrap(err, "failed to create call")
	}

	_, err = s.Call(cl, meta, identity, c)
	if err != nil {
		return errors.Wrap(err, "failed to set fees for service contract")
	}

	return nil
}

// ServiceContractApprove approves a service contract
func (s *Substrate) ServiceContractApprove(identity Identity, contract uint64) error {
	cl, meta, err := s.getClient()
	if err != nil {
		return err
	}

	c, err := types.NewCall(meta, "SmartContractModule.service_contract_approve",
		contract,
	)

	if err != nil {
		return errors.Wrap(err, "failed to create call")
	}

	_, err = s.Call(cl, meta, identity, c)
	if err != nil {
		return errors.Wrap(err, "failed to approve service contract")
	}

	return nil
}

// ServiceContractReject rejects a service contract
func (s *Substrate) ServiceContractReject(identity Identity, contract uint64) error {
	cl, meta, err := s.getClient()
	if err != nil {
		return err
	}

	c, err := types.NewCall(meta, "SmartContractModule.service_contract_reject",
		contract,
	)

	if err != nil {
		return errors.Wrap(err, "failed to create call")
	}

	_, err = s.Call(cl, meta, identity, c)
	if err != nil {
		return errors.Wrap(err, "failed to reject service contract")
	}

	return nil
}

// ServiceContractCancel cancels a service contract
func (s *Substrate) ServiceContractCancel(identity Identity, contract uint64) error {
	cl, meta, err := s.getClient()
	if err != nil {
		return err
	}

	c, err := types.NewCall(meta, "SmartContractModule.service_contract_cancel",
		contract,
	)

	if err != nil {
		return errors.Wrap(err, "failed to create call")
	}

	_, err = s.Call(cl, meta, identity, c)
	if err != nil {
		return errors.Wrap(err, "failed to cancel service contract")
	}

	return nil
}

// ServiceContractBill bills a service contract
func (s *Substrate) ServiceContractBill(identity Identity, contract uint64, variable_amount uint64, metadata string) error {
	cl, meta, err := s.getClient()
	if err != nil {
		return err
	}

	c, err := types.NewCall(meta, "SmartContractModule.service_contract_bill",
		contract, variable_amount, metadata,
	)

	if err != nil {
		return errors.Wrap(err, "failed to create call")
	}

	_, err = s.Call(cl, meta, identity, c)
	if err != nil {
		return errors.Wrap(err, "failed to bill service contract")
	}

	return nil
}

// GetServiceContract gets a service contract given the service contract id
func (s *Substrate) GetServiceContract(id uint64) (*ServiceContract, error) {
	cl, meta, err := s.getClient()
	if err != nil {
		return nil, err
	}

	bytes, err := types.Encode(id)
	if err != nil {
		return nil, errors.Wrap(err, "substrate: encoding error building query arguments")
	}

	key, err := types.CreateStorageKey(meta, "SmartContractModule", "ServiceContracts", bytes, nil)
	if err != nil {
		return nil, errors.Wrap(err, "failed to create substrate query key")
	}

	raw, err := cl.RPC.State.GetStorageRawLatest(key)
	if err != nil {
		return nil, errors.Wrap(err, "failed to lookup contract")
	}

	if len(*raw) == 0 {
		return nil, errors.Wrap(ErrNotFound, "service contract not found")
	}

	var contract ServiceContract
	if err := types.Decode(*raw, &contract); err != nil {
		return nil, errors.Wrap(err, "failed to load object")
	}

	return &contract, nil
}

// GetServiceContractID gets the current value of storage ServiceContractID
func (s *Substrate) GetServiceContractID() (uint64, error) {
	cl, meta, err := s.getClient()
	if err != nil {
		return 0, err
	}

	key, err := types.CreateStorageKey(meta, "SmartContractModule", "ServiceContractID", nil)
	if err != nil {
		return 0, errors.Wrap(err, "failed to create substrate query key")
	}
	var id types.U64
	ok, err := cl.RPC.State.GetStorageLatest(key, &id)
	if err != nil {
		return 0, errors.Wrap(err, "failed to lookup entity")
	}

	if !ok || id == 0 {
		return 0, errors.Wrap(ErrNotFound, "service contract id not found")
	}

	return uint64(id), nil
}
