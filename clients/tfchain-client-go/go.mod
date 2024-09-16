module github.com/threefoldtech/tfchain/clients/tfchain-client-go

go 1.21

toolchain go1.21.3

require (
	github.com/cenkalti/backoff v2.2.1+incompatible
	github.com/centrifuge/go-substrate-rpc-client/v4 v4.0.12
	github.com/jbenet/go-base58 v0.0.0-20150317085156-6237cf65f3a6
	github.com/pkg/errors v0.9.1
	github.com/rs/zerolog v1.26.0
	github.com/stretchr/testify v1.7.2
	github.com/vedhavyas/go-subkey v1.0.3
	golang.org/x/crypto v0.7.0
)

require (
	github.com/ChainSafe/go-schnorrkel v1.0.0 // indirect
	github.com/StackExchange/wmi v1.2.1 // indirect
	github.com/cosmos/go-bip39 v1.0.0 // indirect
	github.com/davecgh/go-spew v1.1.1 // indirect
	github.com/deckarep/golang-set v1.8.0 // indirect
	github.com/decred/base58 v1.0.4 // indirect
	github.com/decred/dcrd/crypto/blake256 v1.0.0 // indirect
	github.com/ethereum/go-ethereum v1.10.20 // indirect
	github.com/go-ole/go-ole v1.2.6 // indirect
	github.com/go-stack/stack v1.8.1 // indirect
	github.com/gorilla/websocket v1.5.0 // indirect
	github.com/gtank/merlin v0.1.1 // indirect
	github.com/gtank/ristretto255 v0.1.2 // indirect
	github.com/kr/pretty v0.3.1 // indirect
	github.com/mimoo/StrobeGo v0.0.0-20220103164710-9a04d6ca976b // indirect
	github.com/pierrec/xxHash v0.1.5 // indirect
	github.com/pmezard/go-difflib v1.0.0 // indirect
	github.com/rs/cors v1.8.2 // indirect
	github.com/tklauser/go-sysconf v0.3.9 // indirect
	github.com/vedhavyas/go-subkey/v2 v2.0.0 // indirect
	golang.org/x/sys v0.6.0 // indirect
	gopkg.in/natefinch/npipe.v2 v2.0.0-20160621034901-c1b8fa8bdcce // indirect
	gopkg.in/yaml.v3 v3.0.1 // indirect
)

replace github.com/centrifuge/go-substrate-rpc-client/v4 => github.com/threefoldtech/go-substrate-rpc-client/v4 v4.0.0-20240916115924-b6e9bfa88b8a
