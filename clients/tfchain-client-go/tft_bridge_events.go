package substrate

import "github.com/centrifuge/go-substrate-rpc-client/v4/types"

// BridgeBurnTransactionCreated
type BridgeBurnTransactionCreated struct {
	Phase             types.Phase
	BurnTransactionID types.U64       `json:"burn_transaction_id"`
	Source            types.AccountID `json:"source"`
	Target            []byte          `json:"target"`
	Amount            types.U64       `json:"amount"`
	Topics            []types.Hash
}

// BridgeBurnTransactionExpired
type BridgeBurnTransactionExpired struct {
	Phase             types.Phase
	BurnTransactionID types.U64 `json:"burn_transaction_id"`
	Target            []byte    `json:"target"`
	Amount            types.U64 `json:"amount"`
	Topics            []types.Hash
}

// BurnTransactionReady
type BurnTransactionReady struct {
	Phase             types.Phase
	BurnTransactionID types.U64 `json:"burn_transaction_id"`
	Topics            []types.Hash
}

// BurnTransactionSignatureAdded
type BurnTransactionSignatureAdded struct {
	Phase             types.Phase
	BurnTransactionID types.U64        `json:"burn_transaction_id"`
	Signature         StellarSignature `json:"signature"`
	Topics            []types.Hash
}

// BurnTransactionProposed
type BurnTransactionProposed struct {
	Phase             types.Phase
	BurnTransactionID types.U64 `json:"burn_transaction_id"`
	Target            []byte    `json:"target"`
	Amount            types.U64 `json:"amount"`
	Topics            []types.Hash
}

// BurnTransactionProcessed
type BurnTransactionProcessed struct {
	Phase  types.Phase
	Burn   BurnTransaction `json:"burn"`
	Topics []types.Hash
}

// RefundTransactionCreated
type RefundTransactionCreated struct {
	Phase                 types.Phase
	RefundTransactionHash []byte    `json:"refund_transaction_hash"`
	Target                []byte    `json:"target"`
	Amount                types.U64 `json:"amount"`
	Topics                []types.Hash
}

// RefundTransactionSignatureAdded
type RefundTransactionSignatureAdded struct {
	Phase                 types.Phase
	RefundTransactionHash []byte           `json:"refund_transaction_hash"`
	Signature             StellarSignature `json:"signature"`
	Topics                []types.Hash
}

// RefundTransactionReady
type RefundTransactionReady struct {
	Phase                 types.Phase
	RefundTransactionHash []byte `json:"refund_transaction_hash"`
	Topics                []types.Hash
}

// RefundTransactionProcessed
type RefundTransactionProcessed struct {
	Phase                 types.Phase
	RefundTransactionHash RefundTransaction `json:"refund_transaction_hash"`
	Topics                []types.Hash
}

// MintTransactionProposed
type MintTransactionProposed struct {
	Phase  types.Phase
	TxHash string    `json:"tx_hash"`
	Target AccountID `json:"target"`
	Amount types.U64 `json:"amount"`
	Topics []types.Hash
}

// MintTransactionVoted
type MintTransactionVoted struct {
	Phase  types.Phase
	TxHash string `json:"tx_hash"`
	Topics []types.Hash
}

// MintCompleted
type MintCompleted struct {
	Phase           types.Phase
	MintTransaction MintTransaction `json:"mint_transaction"`
	Topics          []types.Hash
}

// MintTransactionExpired
type MintTransactionExpired struct {
	Phase  types.Phase
	TxHash string    `json:"tx_hash"`
	Amount types.U64 `json:"amount"`
	Target AccountID `json:"target"`
	Topics []types.Hash
}
