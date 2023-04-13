package substrate

import "github.com/centrifuge/go-substrate-rpc-client/v4/types"

type Voted struct {
	Phase        types.Phase
	Account      AccountID
	ProposalHash types.Hash
	Voted        bool
	Yes          types.U32
	No           types.U32
	Topics       []types.Hash
}

type Proposed struct {
	Phase         types.Phase
	Account       AccountID
	ProposalIndex types.U32
	ProposalHash  types.Hash
	Threshold     types.U32
	Topics        []types.Hash
}

type Approved struct {
	Phase        types.Phase
	ProposalHash types.Hash
	Topics       []types.Hash
}

type Disapproved struct {
	Phase        types.Phase
	ProposalHash types.Hash
	Topics       []types.Hash
}

type Executed struct {
	Phase        types.Phase
	ProposalHash types.Hash
	Result       types.DispatchResult
	Topics       []types.Hash
}

type Closed struct {
	Phase        types.Phase
	ProposalHash types.Hash
	Yes          types.U32
	YesWeight    types.U64
	No           types.U32
	NoWeight     types.U64
	Topics       []types.Hash
}

type ClosedByCouncil struct {
	Phase        types.Phase
	ProposalHash types.Hash
	Vetos        []AccountID
	Topics       []types.Hash
}

type CouncilMemberVeto struct {
	Phase        types.Phase
	ProposalHash types.Hash
	Who          AccountID
	Topics       []types.Hash
}
