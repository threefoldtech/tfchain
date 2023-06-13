package substrate

import (
	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
)

// TODO: add all events from SmartContractModule and TfgridModule

type NodePublicConfig struct {
	Phase  types.Phase
	Node   types.U32          `json:"node_id"`
	Config OptionPublicConfig `json:"config"`
	Topics []types.Hash
}

type FarmStored struct {
	Phase  types.Phase
	Farm   Farm `json:"farm"`
	Topics []types.Hash
}

type FarmDeleted struct {
	Phase  types.Phase
	Farm   types.U32 `json:"farm_id"`
	Topics []types.Hash
}

type NodeStored struct {
	Phase  types.Phase
	Node   Node `json:"node"`
	Topics []types.Hash
}

type NodeDeleted struct {
	Phase  types.Phase
	Node   types.U32 `json:"node_id"`
	Topics []types.Hash
}

type NodeUptimeReported struct {
	Phase     types.Phase
	Node      types.U32 `json:"node_id"`
	Timestamp types.U64 `json:"timestamp"`
	Uptime    types.U64 `json:"uptime"`
	Topics    []types.Hash
}

type PowerTargetChanged struct {
	Phase       types.Phase
	Farm        types.U32 `json:"farm_id"`
	Node        types.U32 `json:"node_id"`
	PowerTarget Power     `json:"power_target"`
	Topics      []types.Hash
}

type PowerStateChanged struct {
	Phase      types.Phase
	Farm       types.U32  `json:"farm_id"`
	Node       types.U32  `json:"node_id"`
	PowerState PowerState `json:"power_state"`
	Topics     []types.Hash
}

type NodeGpuStatusChanged struct {
	Phase     types.Phase
	Node      types.U32  `json:"node_id"`
	GpuStatus types.Bool `json:"gpu_status"`
	Topics    []types.Hash
}

type EntityStored struct {
	Phase  types.Phase
	Entity Entity `json:"entity"`
	Topics []types.Hash
}

type EntityDeleted struct {
	Phase  types.Phase
	Entity types.U32 `json:"entity_id"`
	Topics []types.Hash
}

type TwinStored struct {
	Phase  types.Phase
	Twin   Twin `json:"twin"`
	Topics []types.Hash
}

type TwinDeleted struct {
	Phase  types.Phase
	Twin   types.U32 `json:"twin_id"`
	Topics []types.Hash
}

type TwinEntityStored struct {
	Phase     types.Phase
	Twin      types.U32 `json:"twin_id"`
	Entity    types.U32 `json:"entity_id"`
	Signature []byte    `json:"signature"`
	Topics    []types.Hash
}

type TwinEntityRemoved struct {
	Phase  types.Phase
	Twin   types.U32 `json:"twin_id"`
	Entity types.U32 `json:"identity_id"`
	Topics []types.Hash
}

type TwinAccountBounded struct {
	Phase   types.Phase
	Twin    types.U32 `json:"twin_id"`
	Account AccountID `json:"account_id"`
	Topics  []types.Hash
}

// numeric enum for unit
type Unit byte

type Policy struct {
	Value types.U32
	Unit  Unit
}
type PricingPolicy struct {
	Versioned
	ID                     types.U32 `json:"id"`
	Name                   string    `json:"name"`
	SU                     Policy    `json:"su"`
	CU                     Policy    `json:"cu"`
	NU                     Policy    `json:"nu"`
	IPU                    Policy    `json:"ipu"`
	UniqueName             Policy    `json:"unique_name"`
	DomainName             Policy    `json:"domain_name"`
	FoundationAccount      AccountID `json:"foundation_name"`
	CertifiedSalesAccount  AccountID `json:"certified_sales_account"`
	DedicatedNodesDiscount types.U8  `json:"dedication_nodes_discount"`
}

type PricingPolicyStored struct {
	Phase  types.Phase
	Policy PricingPolicy `json:"pricing_policy"`
	Topics []types.Hash
}

type FarmingPolicy struct {
	Versioned
	ID                types.U32         `json:"id"`
	Name              string            `json:"name"`
	CU                types.U32         `json:"cu"`
	SU                types.U32         `json:"su"`
	NU                types.U32         `json:"nu"`
	IPv4              types.U32         `json:"ipv4"`
	MinimalUptime     types.U16         `json:"minimal_uptime"`
	PolicyCreated     types.U32         `json:"policy_created"`
	PolicyEnd         types.U32         `json:"policy_end"`
	Immutable         bool              `json:"immutable"`
	Default           bool              `json:"default"`
	NodeCertification NodeCertification `json:"node_certification"`
	FarmCertification FarmCertification `json:"farm_certification"`
}

type FarmingPolicyStored struct {
	Phase  types.Phase
	Policy FarmingPolicy `json:"farming_policy"`
	Topics []types.Hash
}

type CertificationCodes struct {
	Versioned
	ID                    types.U32 `json:"id"`
	Name                  string    `json:"name"`
	Description           string    `json:"description"`
	CertificationCodeType byte      `json:"certification_code_type"`
}

type CertificationCodeStored struct {
	Phase  types.Phase
	Codes  CertificationCodes `json:"contract_id"`
	Topics []types.Hash
}

type FarmPayoutV2AddressRegistered struct {
	Phase   types.Phase
	Farm    types.U32 `json:"farm_id"`
	Address string    `json:"address"`
	Topics  []types.Hash
}

type FarmMarkedAsDedicated struct {
	Phase  types.Phase
	Farm   types.U32 `json:"farm_id"`
	Topics []types.Hash
}

type ConnectionPriceSet struct {
	Phase  types.Phase
	Price  types.U32 `json:"price"`
	Topics []types.Hash
}

type NodeCertificationSet struct {
	Phase         types.Phase
	NodeId        types.U32         `json:"node_id"`
	Certification NodeCertification `json:"certification"`
	Topics        []types.Hash
}

type NodeCertifierAdded struct {
	Phase   types.Phase
	Address AccountID `json:"address"`
	Topics  []types.Hash
}

type NodeCertifierRemoved struct {
	Phase   types.Phase
	Address AccountID `json:"address"`
	Topics  []types.Hash
}

type NodeMarkAsDedicated struct {
	Phase     types.Phase
	NodeID    types.U32 `json:"node_id"`
	Dedicated bool      `json:"dedicated"`
	Topics    []types.Hash
}

type FarmingPolicyUpdated struct {
	Phase         types.Phase
	FarmingPolicy FarmingPolicy `json:"farming_policy"`
	Topics        []types.Hash
}

type FarmingPolicySet struct {
	Phase         types.Phase
	Farm          types.U32                `json:"farm_id"`
	FarmingPolicy OptionFarmingPolicyLimit `json:"farming_policy"`
	Topics        []types.Hash
}

type FarmCertificationSet struct {
	Phase         types.Phase
	Farm          types.U32         `json:"farm_id"`
	Certification FarmCertification `json:"farm_certification"`
	Topics        []types.Hash
}

type PriceStored struct {
	Phase types.Phase
	// in rust this is a U16F16 which is a custom type of 4 bytes width to
	// represent a float point with a
	Price  types.U32 `json:"price"`
	Topics []types.Hash
}

type AveragePriceIsAboveMaxPrice struct {
	Phase   types.Phase
	Average types.U32 `json:"average"`
	Max     types.U32 `json:"max"`
	Topics  []types.Hash
}

type AveragePriceIsAboveMinPrice struct {
	Phase   types.Phase
	Average types.U32 `json:"average"`
	Min     types.U32 `json:"min"`
	Topics  []types.Hash
}

type OffchainWorkerExecuted struct {
	Phase   types.Phase
	Account AccountID `json:"account_id"`
	Topics  []types.Hash
}

type EntryEvent struct {
	Phase   types.Phase
	Account AccountID `json:"account_id"`
	Key     []byte    `json:"key"`
	Value   []byte    `json:"value"`
	Topics  []types.Hash
}

type ValidatorAdded struct {
	Phase   types.Phase
	Account AccountID `json:"account_id"`
	Topics  []types.Hash
}

type ValidatorRemoved struct {
	Phase   types.Phase
	Account AccountID `json:"account_id"`
	Topics  []types.Hash
}

type Bonded struct {
	Phase   types.Phase
	Account AccountID `json:"account_id"`
	Topics  []types.Hash
}

type ValidatorCreated struct {
	Phase     types.Phase
	Account   AccountID `json:"account_id"`
	Validator Validator `json:"validator"`
	Topics    []types.Hash
}

type ValidatorApproved struct {
	Phase     types.Phase
	Validator Validator `json:"validator"`
	Topics    []types.Hash
}

// to handle council member events that
// are not defined in base types
type MemberEvent struct {
	Phase  types.Phase
	Topics []types.Hash
}

type ZosVersionUpdated struct {
	Phase   types.Phase
	Version string `json:"version"`
	Topics  []types.Hash
}

type EventSchedulerCallUnavailable struct {
	Phase  types.Phase
	Task   types.TaskAddress
	ID     types.OptionBytes
	Topics []types.Hash
}

type EventSchedulerPeriodicFailed struct {
	Phase  types.Phase
	Task   types.TaskAddress
	ID     types.OptionBytes
	Topics []types.Hash
}

type EventSchedulerPermanentlyOverweight struct {
	Phase  types.Phase
	Task   types.TaskAddress
	ID     types.OptionBytes
	Topics []types.Hash
}

type EventBatchCompletedWithErrors struct {
	Phase  types.Phase
	Topics []types.Hash
}

type EventItemFailed struct {
	Phase  types.Phase
	Error  types.DispatchError
	Topics []types.Hash
}

type EventDispatchedAs struct {
	Phase  types.Phase
	Result types.DispatchResult
	Topics []types.Hash
}

type BalancesMinted struct {
	Phase  types.Phase
	Who    AccountID  `json:"who"`
	Amount types.U128 `json:"amount"`
	Topics []types.Hash
}

type EventWithWho struct {
	Phase  types.Phase
	Who    AccountID `json:"who"`
	Topics []types.Hash
}

type EventWithBalance struct {
	Phase  types.Phase
	Amount types.U128 `json:"amount"`
	Topics []types.Hash
}

// EventRecords is a struct that extends the default events with our events
type EventRecords struct {
	types.EventRecords
	SmartContractModule_ContractCreated              []ContractCreated              //nolint:stylecheck,golint
	SmartContractModule_ContractUpdated              []ContractUpdated              //nolint:stylecheck,golint
	SmartContractModule_NodeContractCanceled         []NodeContractCanceled         //nolint:stylecheck,golint
	SmartContractModule_NameContractCanceled         []NameContractCanceled         //nolint:stylecheck,golint
	SmartContractModule_IPsReserved                  []IPsReserved                  //nolint:stylecheck,golint
	SmartContractModule_IPsFreed                     []IPsFreed                     //nolint:stylecheck,golint
	SmartContractModule_ContractDeployed             []ContractDeployed             //nolint:stylecheck,golint
	SmartContractModule_ConsumptionReportReceived    []ConsumptionReportReceived    //nolint:stylecheck,golint
	SmartContractModule_ContractBilled               []ContractBilled               //nolint:stylecheck,golint
	SmartContractModule_TokensBurned                 []TokensBurned                 //nolint:stylecheck,golint
	SmartContractModule_UpdatedUsedResources         []UpdatedUsedResources         //nolint:stylecheck,golint
	SmartContractModule_NruConsumptionReportReceived []NruConsumptionReportReceived //nolint:stylecheck,golint
	SmartContractModule_RentContractCanceled         []RentContractCanceled         //nolint:stylecheck,golint
	SmartContractModule_ContractGracePeriodStarted   []ContractGracePeriodStarted   //nolint:stylecheck,golint
	SmartContractModule_ContractGracePeriodEnded     []ContractGracePeriodEnded     //nolint:stylecheck,golint
	SmartContractModule_NodeMarkedAsDedicated        []NodeMarkAsDedicated          //nolint:stylecheck,golint
	SmartContractModule_SolutionProviderCreated      []SolutionProviderCreated      //nolint:stylecheck,golint
	SmartContractModule_SolutionProviderApproved     []SolutionProviderApproved     //nolint:stylecheck,golint
	SmartContractModule_ServiceContractCreated       []ServiceContractCreated       //nolint:stylecheck,golint
	SmartContractModule_ServiceContractMetadataSet   []ServiceContractCreated       //nolint:stylecheck,golint
	SmartContractModule_ServiceContractFeesSet       []ServiceContractCreated       //nolint:stylecheck,golint
	SmartContractModule_ServiceContractApproved      []ServiceContractCreated       //nolint:stylecheck,golint
	SmartContractModule_ServiceContractCanceled      []ServiceContractCanceled      //nolint:stylecheck,golint
	SmartContractModule_ServiceContractBilled        []ServiceContractBilled        //nolint:stylecheck,golint
	SmartContractModule_BillingFrequencyChanged      []BillingFrequencyChanged      //nolint:stylecheck,golint

	// farm events
	TfgridModule_FarmStored  []FarmStored  //nolint:stylecheck,golint
	TfgridModule_FarmUpdated []FarmStored  //nolint:stylecheck,golint
	TfgridModule_FarmDeleted []FarmDeleted //nolint:stylecheck,golint

	// node events
	TfgridModule_NodeStored             []NodeStored           //nolint:stylecheck,golint
	TfgridModule_NodeUpdated            []NodeStored           //nolint:stylecheck,golint
	TfgridModule_NodeDeleted            []NodeDeleted          //nolint:stylecheck,golint
	TfgridModule_NodeUptimeReported     []NodeUptimeReported   //nolint:stylecheck,golint
	TfgridModule_NodePublicConfigStored []NodePublicConfig     //nolint:stylecheck,golint
	TfgridModule_PowerTargetChanged     []PowerTargetChanged   //nolint:stylecheck,golint
	TfgridModule_PowerStateChanged      []PowerStateChanged    //nolint:stylecheck,golint
	TfgridModule_NodeGpuStatusChanged   []NodeGpuStatusChanged //nolint:stylecheck,golint

	// entity events
	TfgridModule_EntityStored  []EntityStored  //nolint:stylecheck,golint
	TfgridModule_EntityUpdated []EntityStored  //nolint:stylecheck,golint
	TfgridModule_EntityDeleted []EntityDeleted //nolint:stylecheck,golint

	// twin events
	TfgridModule_TwinStored         []TwinStored         //nolint:stylecheck,golint
	TfgridModule_TwinUpdated        []TwinStored         //nolint:stylecheck,golint
	TfgridModule_TwinDeleted        []TwinDeleted        //nolint:stylecheck,golint
	TfgridModule_TwinEntityStored   []TwinEntityStored   //nolint:stylecheck,golint
	TfgridModule_TwinEntityRemoved  []TwinEntityRemoved  //nolint:stylecheck,golint
	TfgridModule_TwinAccountBounded []TwinAccountBounded //nolint:stylecheck,golint

	// policy events
	TfgridModule_PricingPolicyStored []PricingPolicyStored //nolint:stylecheck,golint
	TfgridModule_FarmingPolicyStored []FarmingPolicyStored //nolint:stylecheck,golint

	// other events
	TfgridModule_FarmPayoutV2AddressRegistered []FarmPayoutV2AddressRegistered //nolint:stylecheck,golint
	TfgridModule_FarmMarkedAsDedicated         []FarmMarkedAsDedicated         //nolint:stylecheck,golint
	TfgridModule_ConnectionPriceSet            []ConnectionPriceSet            //nolint:stylecheck,golint
	TfgridModule_NodeCertificationSet          []NodeCertificationSet          //nolint:stylecheck,golint
	TfgridModule_NodeCertifierAdded            []NodeCertifierAdded            //nolint:stylecheck,golint
	TfgridModule_NodeCertifierRemoved          []NodeCertifierRemoved          //nolint:stylecheck,golint
	TfgridModule_FarmingPolicyUpdated          []FarmingPolicyUpdated          //nolint:stylecheck,golint
	TfgridModule_FarmingPolicySet              []FarmingPolicySet              //nolint:stylecheck,golint
	TfgridModule_FarmCertificationSet          []FarmCertificationSet          //nolint:stylecheck,golint
	TfgridModule_ZosVersionUpdated             []ZosVersionUpdated             //nolint:stylecheck,golint

	// burn module events
	BurningModule_BurnTransactionCreated []BurnTransactionCreated //nolint:stylecheck,golint

	// TFT bridge module

	// mints
	TFTBridgeModule_MintTransactionProposed []MintTransactionProposed //nolint:stylecheck,golint
	TFTBridgeModule_MintTransactionVoted    []MintTransactionVoted    //nolint:stylecheck,golint
	TFTBridgeModule_MintCompleted           []MintCompleted           //nolint:stylecheck,golint
	TFTBridgeModule_MintTransactionExpired  []MintTransactionExpired  //nolint:stylecheck,golint

	// burns
	TFTBridgeModule_BurnTransactionCreated        []BridgeBurnTransactionCreated  //nolint:stylecheck,golint
	TFTBridgeModule_BurnTransactionProposed       []BurnTransactionProposed       //nolint:stylecheck,golint
	TFTBridgeModule_BurnTransactionSignatureAdded []BurnTransactionSignatureAdded //nolint:stylecheck,golint
	TFTBridgeModule_BurnTransactionReady          []BurnTransactionReady          //nolint:stylecheck,golint
	TFTBridgeModule_BurnTransactionProcessed      []BurnTransactionProcessed      //nolint:stylecheck,golint
	TFTBridgeModule_BurnTransactionExpired        []BridgeBurnTransactionExpired  //nolint:stylecheck,golint

	// refunds
	TFTBridgeModule_RefundTransactionCreated        []RefundTransactionCreated        //nolint:stylecheck,golint
	TFTBridgeModule_RefundTransactionsignatureAdded []RefundTransactionSignatureAdded //nolint:stylecheck,golint
	TFTBridgeModule_RefundTransactionReady          []RefundTransactionReady          //nolint:stylecheck,golint
	TFTBridgeModule_RefundTransactionProcessed      []RefundTransactionProcessed      //nolint:stylecheck,golint
	TFTBridgeModule_RefundTransactionExpired        []RefundTransactionCreated        //nolint:stylecheck,golint

	// TFTPrice module
	TFTPriceModule_PriceStored                 []PriceStored            //nolint:stylecheck,golint
	TFTPriceModule_AveragePriceStored          []PriceStored            //nolint:stylecheck,golint
	TFTPriceModule_OffchainWorkerExecuted      []OffchainWorkerExecuted //nolint:stylecheck,golint
	TFTPriceModule_AveragePriceIsAboveMaxPrice []AveragePriceIsAboveMaxPrice
	TFTPriceModule_AveragePriceIsBelowMinPrice []AveragePriceIsAboveMinPrice
	// KVStore
	TFKVStore_EntrySet   []EntryEvent //nolint:stylecheck,golint
	TFKVStore_EntryGot   []EntryEvent //nolint:stylecheck,golint
	TFKVStore_EntryTaken []EntryEvent //nolint:stylecheck,golint

	// Validatorset pallet
	ValidatorSet_ValidatorAdditionInitiated []ValidatorAdded   //nolint:stylecheck,golint
	ValidatorSet_ValidatorRemovalInitiated  []ValidatorRemoved //nolint:stylecheck,golint

	Validator_Bonded                   []Bonded            //nolint:stylecheck,golint
	Validator_ValidatorRequestCreated  []ValidatorCreated  //nolint:stylecheck,golint
	Validator_ValidatorRequestApproved []ValidatorApproved //nolint:stylecheck,golint
	Validator_ValidatorActivated       []ValidatorApproved //nolint:stylecheck,golint
	Validator_ValidatorRemoved         []ValidatorApproved //nolint:stylecheck,golint
	Validator_NodeValidatorChanged     []Bonded            //nolint:stylecheck,golint
	Validator_NodeValidatorRemoved     []Bonded            //nolint:stylecheck,golint

	CouncilMembership_MemberAdded    []MemberEvent //nolint:stylecheck,golint
	CouncilMembership_MemberRemoved  []MemberEvent //nolint:stylecheck,golint
	CouncilMembership_MembersSwapped []MemberEvent //nolint:stylecheck,golint
	CouncilMembership_MembersReset   []MemberEvent //nolint:stylecheck,golint
	CouncilMembership_KeyChanged     []MemberEvent //nolint:stylecheck,golint
	CouncilMembership_Dummy          []MemberEvent //nolint:stylecheck,golint

	// Dao Pallet
	Dao_Voted             []Voted             //nolint:stylecheck,golint
	Dao_Proposed          []Proposed          //nolint:stylecheck,golint
	Dao_Approved          []Approved          //nolint:stylecheck,golint
	Dao_Disapproved       []Disapproved       //nolint:stylecheck,golint
	Dao_Executed          []Executed          //nolint:stylecheck,golint
	Dao_Closed            []Closed            //nolint:stylecheck,golint
	Dao_ClosedByCouncil   []ClosedByCouncil   //nolint:stylecheck,golint
	Dao_CouncilMemberVeto []CouncilMemberVeto //nolint:stylecheck,golint

	// Scheduler Pallet (temporary until go-substrate-rpc-client is fixed: https://github.com/centrifuge/go-substrate-rpc-client/issues/344)
	Scheduler_CallUnavailable        []EventSchedulerCallUnavailable       `test-gen-blockchain:"polkadot"`
	Scheduler_PeriodicFailed         []EventSchedulerPeriodicFailed        `test-gen-blockchain:"polkadot"`
	Scheduler_PermanentlyOverweight  []EventSchedulerPermanentlyOverweight `test-gen-blockchain:"polkadot"`
	Utility_BatchCompletedWithErrors []EventBatchCompletedWithErrors       `test-gen-blockchain:"centrifuge-parachain"`
	Utility_ItemFailed               []EventItemFailed                     `test-gen-blockchain:"centrifuge-parachain"`
	Utility_DispatchedAs             []EventDispatchedAs                   `test-gen-blockchain:"centrifuge-parachain"`

	// overrides
	Balances_Minted     []BalancesMinted   //nolint:stylecheck,golint
	Balances_BalanceSet []BalancesMinted   //nolint:stylecheck,golint
	Balances_Burned     []BalancesMinted   //nolint:stylecheck,golint
	Balances_Suspended  []BalancesMinted   //nolint:stylecheck,golint
	Balances_Restored   []BalancesMinted   //nolint:stylecheck,golint
	Balances_Upgraded   []EventWithWho     //nolint:stylecheck,golint
	Balances_Issued     []EventWithBalance //nolint:stylecheck,golint
	Balances_Rescinded  []EventWithBalance //nolint:stylecheck,golint
	Balances_Locked     []BalancesMinted   //nolint:stylecheck,golint
	Balances_Unlocked   []BalancesMinted   //nolint:stylecheck,golint
	Balances_Frozen     []BalancesMinted   //nolint:stylecheck,golint
	Balances_Thawed     []BalancesMinted   //nolint:stylecheck,golint
}
