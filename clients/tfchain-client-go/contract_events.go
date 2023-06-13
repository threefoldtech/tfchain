package substrate

import (
	"fmt"

	"github.com/centrifuge/go-substrate-rpc-client/v4/scale"
	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
)

// ContractBill structure
type ContractBill struct {
	ContractID    types.U64     `json:"contract_id"`
	Timestamp     types.U64     `json:"timestamp"`
	DiscountLevel DiscountLevel `json:"discount_level"`
	AmountBilled  types.U128    `json:"amount_billed"`
}

// DiscountLevel enum
type DiscountLevel struct {
	IsNone    bool `json:"is_none"`
	IsDefault bool `json:"is_default"`
	IsBronze  bool `json:"is_bronze"`
	IsSilver  bool `json:"is_silver"`
	IsGold    bool `json:"is_gold"`
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
	Contract Contract `json:"contract"`
	Topics   []types.Hash
}

// ContractUpdated is the contract updated event
type ContractUpdated struct {
	Phase    types.Phase
	Contract Contract `json:"contract"`
	Topics   []types.Hash
}

// NodeContractCanceled
type NodeContractCanceled struct {
	Phase      types.Phase
	ContractID types.U64 `json:"contract_id"`
	Node       types.U32 `json:"node_id"`
	Twin       types.U32 `json:"twin_id"`
	Topics     []types.Hash
}

// NameContractCanceled
type NameContractCanceled struct {
	Phase      types.Phase
	ContractID types.U64 `json:"contract_id"`
	Topics     []types.Hash
}

// ServiceContractCreated
type ServiceContractCreated struct {
	Phase           types.Phase
	ServiceContract ServiceContract `json:"service_contract"`
	Topics          []types.Hash
}

// ServiceContractCanceled
type ServiceContractCanceled struct {
	Phase             types.Phase
	ServiceContractID types.U64    `json:"service_contract_id"`
	Cause             DeletedState `json:"cause"`
	Topics            []types.Hash
}

// ServiceContractBilled
type ServiceContractBilled struct {
	Phase               types.Phase
	ServiceContract     ServiceContract     `json:"service_contract"`
	ServiceContractBill ServiceContractBill `json:"service_contract_bill"`
	Amount              types.U128          `json:"amount"`
	Topics              []types.Hash
}

// ContractDeployed
type ContractDeployed struct {
	Phase      types.Phase
	ContractID types.U64 `json:"contract_id"`
	AccountID  AccountID `json:"account_id"`
	Topics     []types.Hash
}

// DEPRECATED
// ConsumptionReportReceived
type ConsumptionReportReceived struct {
	Phase       types.Phase
	Consumption Consumption `json:"consumption"`
	Topics      []types.Hash
}

// ConsumptionReportReceived
type NruConsumptionReportReceived struct {
	Phase       types.Phase
	Consumption NruConsumption `json:"consumption"`
	Topics      []types.Hash
}

// ContractBilled
type ContractBilled struct {
	Phase        types.Phase
	ContractBill ContractBill `json:"contract_bill"`
	Topics       []types.Hash
}

// IPsReserved
type IPsReserved struct {
	Phase      types.Phase
	ContractID types.U64  `json:"contract_id"`
	IPs        []PublicIP `json:"ips"`
	Topics     []types.Hash
}

// IPsFreed
type IPsFreed struct {
	Phase      types.Phase
	ContractID types.U64  `json:"contract_id"`
	IPs        []PublicIP `json:"ips"`
	Topics     []types.Hash
}

// TokensBurned
type TokensBurned struct {
	Phase      types.Phase
	ContractID types.U64  `json:"contract_id"`
	Amount     types.U128 `json:"amount"`
	Topics     []types.Hash
}

type UpdatedUsedResources struct {
	Phase             types.Phase
	ContractResources ContractResources `json:"contract_resources"`
	Topics            []types.Hash
}

// RentContractCanceled
type RentContractCanceled struct {
	Phase      types.Phase
	ContractID types.U64 `json:"contract_id"`
	Topics     []types.Hash
}

type ContractGracePeriodStarted struct {
	Phase      types.Phase
	ContractID types.U64 `json:"contract_id"`
	NodeID     types.U32 `json:"node_id"`
	TwinID     types.U32 `json:"twin_id"`
	StartBlock types.U64 `json:"start_block"`
	Topics     []types.Hash
}

type ContractGracePeriodEnded struct {
	Phase      types.Phase
	ContractID types.U64 `json:"contract_id"`
	NodeID     types.U32 `json:"node_id"`
	TwinID     types.U32 `json:"twin_id"`
	Topics     []types.Hash
}

type SolutionProvider struct {
	SolutionProviderID types.U64  `json:"solution_provider_id"`
	Providers          []Provider `json:"providers"`
	Description        string     `json:"description"`
	Link               string     `json:"link"`
	Approved           bool       `json:"approved"`
}

type Provider struct {
	Who  types.AccountID `json:"who"`
	Take types.U8        `json:"take"`
}

type SolutionProviderCreated struct {
	Phase            types.Phase
	SolutionProvider SolutionProvider `json:"solution_provider"`
	Topics           []types.Hash
}

type SolutionProviderApproved struct {
	Phase              types.Phase
	SolutionProviderID types.U64 `json:"solution_provider_id"`
	Approved           bool      `json:"approved"`
	Topics             []types.Hash
}

type BillingFrequencyChanged struct {
	Phase     types.Phase
	Frequency types.U64 `json:"frequency"`
	Topics    []types.Hash
}

type NodeExtraFeeSet struct {
	Phase    types.Phase
	NodeID   types.U32 `json:"node_id"`
	ExtraFee types.U64 `json:"extra_fee"`
	Topics   []types.Hash
}
