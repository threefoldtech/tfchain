package substrate

import "github.com/centrifuge/go-substrate-rpc-client/v4/types"

type BlockNumber types.U32

type StellarSignature struct {
	Signature      []byte
	StellarAddress []byte
}
