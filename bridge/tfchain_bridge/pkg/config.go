package pkg

import "errors"

type BridgeConfig struct {
	TfchainURLs         []string
	TfchainSeed         string
	RescanBridgeAccount bool
	PersistencyFile     string
	StellarConfig
}

type StellarConfig struct {
	// stellar account to monitor
	StellarBridgeAccount string
	// network for the stellar config
	StellarNetwork string
	// seed for the stellar bridge wallet
	StellarSeed string
	// url for stellar horizon
	StellarHorizonUrl string
}

type StellarSignature struct {
	Signature      []byte
	StellarAddress []byte
}

var ErrTransactionAlreadyRefunded = errors.New("transaction is already refunded")
var ErrTransactionAlreadyMinted = errors.New("transaction is already minted")
var ErrTransactionAlreadyBurned = errors.New("transaction is already burned")
var ErrNoSignatures = errors.New("transaction has no signatures")

// ErrStellarTransactionUndeliverable signals that a Stellar submission failed
// with a permanent, target-side error (e.g. the destination removed its
// trustline) that can never succeed on retry. The refund must be quarantined
// and the bridge must continue, rather than crashing in a retry loop.
var ErrStellarTransactionUndeliverable = errors.New("stellar transaction permanently undeliverable to target account")
