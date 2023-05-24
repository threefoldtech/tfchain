package substrate

import "github.com/centrifuge/go-substrate-rpc-client/v4/types"

type Voted struct {
	Phase        types.Phase
	Account      AccountID  `json:"account_id"`
	ProposalHash types.Hash `json:"proposal_hash"`
	Voted        bool       `json:"voted"`
	Yes          types.U32  `json:"yes"`
	No           types.U32  `json:"no"`
	Topics       []types.Hash
}

type Proposed struct {
	Phase         types.Phase
	Account       AccountID  `json:"account_id"`
	ProposalIndex types.U32  `json:"proposal_index"`
	ProposalHash  types.Hash `json:"proposal_hash"`
	Threshold     types.U32  `json:"threshold"`
	Topics        []types.Hash
}

type Approved struct {
	Phase        types.Phase
	ProposalHash types.Hash `json:"proposal_hash"`
	Topics       []types.Hash
}

type Disapproved struct {
	Phase        types.Phase
	ProposalHash types.Hash `json:"proposal_hash"`
	Topics       []types.Hash
}

type Executed struct {
	Phase        types.Phase
	ProposalHash types.Hash           `json:"proposal_hash"`
	Result       types.DispatchResult `json:"result"`
	Topics       []types.Hash
}

type Closed struct {
	Phase        types.Phase
	ProposalHash types.Hash `json:"proposal_hash"`
	Yes          types.U32  `json:"yes"`
	YesWeight    types.U64  `json:"yes_weight"`
	No           types.U32  `json:"no"`
	NoWeight     types.U64  `json:"no_weight"`
	Topics       []types.Hash
}

type ClosedByCouncil struct {
	Phase        types.Phase
	ProposalHash types.Hash  `json:"proposal_hash"`
	Vetos        []AccountID `json:"vetos"`
	Topics       []types.Hash
}

type CouncilMemberVeto struct {
	Phase        types.Phase
	ProposalHash types.Hash `json:"proposal_hash"`
	Who          AccountID  `json:"who"`
	Topics       []types.Hash
}
