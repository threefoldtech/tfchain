package substrate

import (
	"fmt"

	"github.com/centrifuge/go-substrate-rpc-client/v4/scale"
)

type Validator struct {
	ValidatorNodeAccount AccountID             `json:"validator_node_account"`
	StashAccount         AccountID             `json:"stash_account"`
	Description          string                `json:"description"`
	TfConnectId          string                `json:"tf_connect_id"`
	Info                 string                `json:"info"`
	State                ValidatorRequestState `json:"state"`
}

type ValidatorRequestState struct {
	IsCreated    bool `json:"is_created"`
	IsApproved   bool `json:"is_approved"`
	IsValidating bool `json:"is_validating"`
}

// Decode implementation for the enum type
func (r *ValidatorRequestState) Decode(decoder scale.Decoder) error {
	b, err := decoder.ReadOneByte()
	if err != nil {
		return err
	}

	switch b {
	case 0:
		r.IsCreated = true
	case 1:
		r.IsApproved = true
	case 2:
		r.IsValidating = true
	default:
		return fmt.Errorf("unknown ValidatorRequestState value")
	}

	return nil
}
