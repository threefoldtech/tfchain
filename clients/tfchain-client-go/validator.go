package substrate

import (
	"fmt"

	"github.com/centrifuge/go-substrate-rpc-client/v4/scale"
)

type Validator struct {
	ValidatorNodeAccount AccountID
	StashAccount         AccountID
	Description          string
	TfConnectId          string
	Info                 string
	State                ValidatorRequestState
}

type ValidatorRequestState struct {
	IsCreated    bool
	IsApproved   bool
	IsValidating bool
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
