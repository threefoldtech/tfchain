package main

import (
	"context"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/rs/zerolog/log"
	flag "github.com/spf13/pflag"
	"github.com/threefoldtech/tfchain/bridge/tfchain_bridge/pkg"
	"github.com/threefoldtech/tfchain/bridge/tfchain_bridge/pkg/bridge"
	"github.com/threefoldtech/tfchain/bridge/tfchain_bridge/pkg/logger"
)

func main() {
	var bridgeCfg pkg.BridgeConfig

	var debug bool
	flag.StringVar(&bridgeCfg.TfchainURL, "tfchainurl", "", "Tfchain websocket url")
	flag.StringVar(&bridgeCfg.TfchainSeed, "tfchainseed", "", "Tfchain secret seed")
	flag.StringVar(&bridgeCfg.StellarBridgeAccount, "bridgewallet", "", "stellar bridge wallet")
	flag.StringVar(&bridgeCfg.StellarSeed, "secret", "", "stellar secret")
	flag.StringVar(&bridgeCfg.StellarNetwork, "network", "testnet", "stellar network url")
	flag.StringVar(&bridgeCfg.PersistencyFile, "persistency", "./node.json", "file where last seen blockheight and stellar account cursor is stored")
	flag.BoolVar(&bridgeCfg.RescanBridgeAccount, "rescan", false, "if true is provided, we rescan the bridge stellar account and mint all transactions again")
	flag.StringVar(&bridgeCfg.StellarHorizonUrl, "horizon", "", "stellar horizon url endpoint")
	flag.BoolVar(&debug, "debug", false, "sets debug level log output")

	flag.Parse()

	logger.InitLogger(debug)

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	timeout, timeoutCancel := context.WithTimeout(ctx, time.Second*15)
	defer timeoutCancel()

	br, address, err := bridge.NewBridge(timeout, bridgeCfg)
	if err != nil {
		log.Fatal().
			Err(err).
			Str("event_action", "bridge_init_aborted").
			Str("event_kind", "error").
			Str("category", "availability").
			Msg("the bridge instance cannot be started")
	}
	sourceLogEntry := logger.SourceCommonLogEntry{
		Instance_public_key:   address,
		Bridge_wallet_address: bridgeCfg.StellarBridgeAccount,
		Stellar_network:       bridgeCfg.StellarNetwork,
		Tfchain_url:           bridgeCfg.TfchainURL,
	}

	log.Logger = log.Logger.With().Interface("source", sourceLogEntry).Logger()

	sigs := make(chan os.Signal, 1)

	signal.Notify(sigs, syscall.SIGINT, syscall.SIGTERM)

	go func() {
		log.Debug().Msg("awaiting signal")
		<-sigs
		log.Debug().Msg("shutting now")
		cancel()
	}()

	if err = br.Start(ctx); err != nil && err != context.Canceled {
		log.Fatal().
			Err(err).
			Str("event_action", "bridge_unexpectedly_exited").
			Str("event_kind", "error").
			Str("category", "availability").
			Msg("the bridge instance has exited unexpectedly")
	}
	log.Info().
		Str("event_action", "bridge_stopped").
		Str("event_kind", "event").
		Str("category", "availability").
		Msg("the bridge instance has stopped")
}
