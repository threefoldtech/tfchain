package substrate

import (
	"fmt"

	"github.com/centrifuge/go-substrate-rpc-client/v4/scale"
	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
)

// ContractBill structure
type ContractBill struct {
	ContractID    types.U64
	Timestamp     types.U64
	DiscountLevel DiscountLevel
	AmountBilled  types.U128
}

// DiscountLevel enum
type DiscountLevel struct {
	IsNone    bool
	IsDefault bool
	IsBronze  bool
	IsSilver  bool
	IsGold    bool
}

// Decode implementation for the enum type
func (r *DiscountLevel) Decode(decoder scale.Decoder) error {
	b, err := decoder.ReadOneByte()
	if err != nil {
		return err
	}

	switch b {
	case 0:
		r.IsNone = true
	case 1:
		r.IsDefault = true
	case 2:
		r.IsBronze = true
	case 3:
		r.IsSilver = true
	case 4:
		r.IsGold = true
	default:
		return fmt.Errorf("unknown Contract Discount Level value")
	}

	return nil
}

// Encode implementation
func (r DiscountLevel) Encode(encoder scale.Encoder) (err error) {
	if r.IsNone {
		err = encoder.PushByte(0)
	} else if r.IsDefault {
		err = encoder.PushByte(1)
	} else if r.IsBronze {
		err = encoder.PushByte(2)
	} else if r.IsSilver {
		err = encoder.PushByte(3)
	} else if r.IsGold {
		err = encoder.PushByte(4)
	}

	return
}

// ContractCreated is the contract created event
type ContractCreated struct {
	Phase    types.Phase
	Contract Contract
	Topics   []types.Hash
}

// ContractUpdated is the contract updated event
type ContractUpdated struct {
	Phase    types.Phase
	Contract Contract
	Topics   []types.Hash
}

// NodeContractCanceled
type NodeContractCanceled struct {
	Phase      types.Phase
	ContractID types.U64
	Node       types.U32
	Twin       types.U32
	Topics     []types.Hash
}

// NameContractCanceled
type NameContractCanceled struct {
	Phase      types.Phase
	ContractID types.U64
	Topics     []types.Hash
}

// ServiceContractCreated
type ServiceContractCreated struct {
	Phase           types.Phase
	ServiceContract ServiceContract
	Topics          []types.Hash
}

// ServiceContractCanceled
type ServiceContractCanceled struct {
	Phase             types.Phase
	ServiceContractID types.U64
	Cause             DeletedState
	Topics            []types.Hash
}

// ServiceContractBilled
type ServiceContractBilled struct {
	Phase               types.Phase
	ServiceContract     ServiceContract
	ServiceContractBill ServiceContractBill
	Amount              types.U128
	Topics              []types.Hash
}

// ContractDeployed
type ContractDeployed struct {
	Phase      types.Phase
	ContractID types.U64
	AccountID  AccountID
	Topics     []types.Hash
}

// DEPRECATED
// ConsumptionReportReceived
type ConsumptionReportReceived struct {
	Phase       types.Phase
	Consumption Consumption
	Topics      []types.Hash
}

// ConsumptionReportReceived
type NruConsumptionReportReceived struct {
	Phase       types.Phase
	Consumption NruConsumption
	Topics      []types.Hash
}

// ContractBilled
type ContractBilled struct {
	Phase        types.Phase
	ContractBill ContractBill
	Topics       []types.Hash
}

// IPsReserved
type IPsReserved struct {
	Phase      types.Phase
	ContractID types.U64
	IPs        []PublicIP
	Topics     []types.Hash
}

// IPsFreed
type IPsFreed struct {
	Phase      types.Phase
	ContractID types.U64
	IPs        []PublicIP
	Topics     []types.Hash
}

// TokensBurned
type TokensBurned struct {
	Phase      types.Phase
	ContractID types.U64
	Amount     types.U128
	Topics     []types.Hash
}

type UpdatedUsedResources struct {
	Phase             types.Phase
	ContractResources ContractResources
	Topics            []types.Hash
}

// RentContractCanceled
type RentContractCanceled struct {
	Phase      types.Phase
	ContractID types.U64
	Topics     []types.Hash
}

type ContractGracePeriodStarted struct {
	Phase      types.Phase
	ContractID types.U64
	NodeID     types.U32
	TwinID     types.U32
	StartBlock types.U64
	Topics     []types.Hash
}

type ContractGracePeriodEnded struct {
	Phase      types.Phase
	ContractID types.U64
	NodeID     types.U32
	TwinID     types.U32
	Topics     []types.Hash
}

type SolutionProvider struct {
	SolutionProviderID types.U64
	Providers          []Provider
	Description        string
	Link               string
	Approved           bool
}

type Provider struct {
	Who  types.AccountID
	Take types.U8
}

type SolutionProviderCreated struct {
	Phase            types.Phase
	SolutionProvider SolutionProvider
	Topics           []types.Hash
}

type SolutionProviderApproved struct {
	Phase              types.Phase
	SolutionProviderID types.U64
	Approved           bool
	Topics             []types.Hash
}

type BillingFrequencyChanged struct {
	Phase     types.Phase
	Frequency types.U64
	Topics    []types.Hash
}
