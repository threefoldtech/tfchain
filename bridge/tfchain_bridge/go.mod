module github.com/threefoldtech/tfchain/bridge/tfchain_bridge

go 1.17

require (
	github.com/centrifuge/go-substrate-rpc-client/v4 v4.0.12
	github.com/pkg/errors v0.9.1
	github.com/spf13/pflag v1.0.5
)

require (
	github.com/ethereum/go-ethereum v1.10.17 // indirect
	github.com/klauspost/compress v1.9.5 // indirect
	github.com/konsorten/go-windows-terminal-sequences v1.0.2 // indirect
	github.com/rs/zerolog v1.26.0
	github.com/sirupsen/logrus v1.4.2 // indirect
	github.com/stellar/go v0.0.0-20210922122349-e6f322c047c5
	github.com/stretchr/objx v0.3.0 // indirect
	github.com/vedhavyas/go-subkey v1.0.3
)

require github.com/cenkalti/backoff/v4 v4.1.3

require (
	github.com/ChainSafe/go-schnorrkel v1.0.0 // indirect
	github.com/cenkalti/backoff v2.2.1+incompatible // indirect
	github.com/cosmos/go-bip39 v1.0.0 // indirect
	github.com/davecgh/go-spew v1.1.1 // indirect
	github.com/deckarep/golang-set v1.8.0 // indirect
	github.com/decred/base58 v1.0.3 // indirect
	github.com/decred/dcrd/crypto/blake256 v1.0.0 // indirect
	github.com/go-chi/chi v4.0.3+incompatible // indirect
	github.com/go-errors/errors v0.0.0-20150906023321-a41850380601 // indirect
	github.com/go-stack/stack v1.8.1 // indirect
	github.com/gorilla/schema v1.1.0 // indirect
	github.com/gorilla/websocket v1.5.0 // indirect
	github.com/gtank/merlin v0.1.1 // indirect
	github.com/gtank/ristretto255 v0.1.2 // indirect
	github.com/jbenet/go-base58 v0.0.0-20150317085156-6237cf65f3a6 // indirect
	github.com/manucorporat/sse v0.0.0-20160126180136-ee05b128a739 // indirect
	github.com/mimoo/StrobeGo v0.0.0-20210601165009-122bf33a46e0 // indirect
	github.com/pierrec/xxHash v0.1.5 // indirect
	github.com/pmezard/go-difflib v1.0.0 // indirect
	github.com/rs/cors v1.8.2 // indirect
	github.com/segmentio/go-loggly v0.5.1-0.20171222203950-eb91657e62b2 // indirect
	github.com/stellar/go-xdr v0.0.0-20201028102745-f80a23dac78a // indirect
	github.com/stretchr/testify v1.7.0 // indirect
	github.com/threefoldtech/tfchain/clients/tfchain-client-go v0.0.0-20230607082553-5605bca61c79 // indirect
	golang.org/x/crypto v0.0.0-20211117183948-ae814b36b871 // indirect
	golang.org/x/sys v0.0.0-20211124211545-fe61309f8881 // indirect
	gopkg.in/natefinch/npipe.v2 v2.0.0-20160621034901-c1b8fa8bdcce // indirect
	gopkg.in/yaml.v3 v3.0.0-20210107192922-496545a6307b // indirect
)

replace github.com/threefoldtech/tfchain/clients/tfchain-client-go => ../../clients/tfchain-client-go
