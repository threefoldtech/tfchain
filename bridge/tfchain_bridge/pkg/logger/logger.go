package logger

import (
	"github.com/threefoldtech/tfchain_bridge/pkg"
)

const VERSION = 1

type Source struct {
	Instance_public_key   string
	Bridge_wallet_address string
	Stellar_network       string
	Tfchain_url           string
}

func New_log_source(address string, config pkg.BridgeConfig) Source {
	return Source{
		address,
		config.StellarBridgeAccount,
		config.StellarNetwork,
		config.TfchainURL,
	}
}
