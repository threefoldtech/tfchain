package substrate

import "github.com/centrifuge/go-substrate-rpc-client/v4/types"

// BridgeBurnTransactionCreated
type BridgeBurnTransactionCreated struct {
	Phase             types.Phase
	BurnTransactionID types.U64
	Source            types.AccountID
	Target            []byte
	Amount            types.U64
	Topics            []types.Hash
}

// BridgeBurnTransactionExpired
type BridgeBurnTransactionExpired struct {
	Phase             types.Phase
	BurnTransactionID types.U64
	Target            []byte
	Amount            types.U64
	Topics            []types.Hash
}

// BurnTransactionReady
type BurnTransactionReady struct {
	Phase             types.Phase
	BurnTransactionID types.U64
	Topics            []types.Hash
}

// BurnTransactionSignatureAdded
type BurnTransactionSignatureAdded struct {
	Phase             types.Phase
	BurnTransactionID types.U64
	Signature         StellarSignature
	Topics            []types.Hash
}

// BurnTransactionProposed
type BurnTransactionProposed struct {
	Phase             types.Phase
	BurnTransactionID types.U64
	Target            []byte
	Amount            types.U64
	Topics            []types.Hash
}

// BurnTransactionProcessed
type BurnTransactionProcessed struct {
	Phase  types.Phase
	Burn   BurnTransaction
	Topics []types.Hash
}

// RefundTransactionCreated
type RefundTransactionCreated struct {
	Phase                 types.Phase
	RefundTransactionHash []byte
	Target                []byte
	Amount                types.U64
	Topics                []types.Hash
}

// RefundTransactionSignatureAdded
type RefundTransactionSignatureAdded struct {
	Phase                 types.Phase
	RefundTransactionHash []byte
	Signature             StellarSignature
	Topics                []types.Hash
}

// RefundTransactionReady
type RefundTransactionReady struct {
	Phase                 types.Phase
	RefundTransactionHash []byte
	Topics                []types.Hash
}

// RefundTransactionProcessed
type RefundTransactionProcessed struct {
	Phase                 types.Phase
	RefundTransactionHash RefundTransaction
	Topics                []types.Hash
}

// MintTransactionProposed
type MintTransactionProposed struct {
	Phase  types.Phase
	TxHash string
	Target AccountID
	Amount types.U64
	Topics []types.Hash
}

// MintTransactionVoted
type MintTransactionVoted struct {
	Phase  types.Phase
	TxHash string
	Topics []types.Hash
}

// MintCompleted
type MintCompleted struct {
	Phase           types.Phase
	MintTransaction MintTransaction
	Topics          []types.Hash
}

// MintTransactionExpired
type MintTransactionExpired struct {
	Phase  types.Phase
	TxHash string
	Amount types.U64
	Target AccountID
	Topics []types.Hash
}
