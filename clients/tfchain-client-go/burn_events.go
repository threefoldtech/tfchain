package substrate

import "github.com/centrifuge/go-substrate-rpc-client/v4/types"

type BurnTransactionCreated struct {
	Phase  types.Phase `json:"phase"`
	Target AccountID   `json:"target"`
	// TODO check if this works ....
	Balance     types.U128   `json:"balance"`
	BlockNumber BlockNumber  `json:"block_number"`
	Message     string       `json:"message"`
	Topics      []types.Hash `json:"topics"`
}
