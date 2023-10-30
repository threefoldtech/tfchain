package main

import (
	"context"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
	flag "github.com/spf13/pflag"
	"github.com/threefoldtech/tfchain_bridge/pkg"
	"github.com/threefoldtech/tfchain_bridge/pkg/bridge"
	"github.com/threefoldtech/tfchain_bridge/pkg/logger"
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

	log.Logger = zerolog.New(os.Stdout).With().Timestamp().Uint("version", logger.VERSION).Logger()
	if debug {
		zerolog.SetGlobalLevel(zerolog.DebugLevel)
		log.Debug().Msg("debug mode enabled")
	} else {
		zerolog.SetGlobalLevel(zerolog.InfoLevel)
	}

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	timeout, timeoutCancel := context.WithTimeout(ctx, time.Second*15)
	defer timeoutCancel()

	br, address, err := bridge.NewBridge(timeout, bridgeCfg)
	if err != nil {
		log.Fatal().
			Err(err).
			Str("event_type", "bridge_aborted").
			Dict("event", zerolog.Dict().
									Str("tfchain_url", bridgeCfg.TfchainURL).
									Str("stellar_horizon_url", bridgeCfg.StellarHorizonUrl).
									Str("stellar_network", bridgeCfg.StellarNetwork).
									Bool("Rescan_flag", bridgeCfg.RescanBridgeAccount)).
			Msg("bridge instance can not be created") // no source yet
	}
	log_source := logger.New_log_source(address, bridgeCfg)

	log.Logger = zerolog.New(os.Stdout).With().Interface("source", log_source).Logger()

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
			Str("event_type", "bridge_unexpectedly_exited").
			Msg("bridge instance exited unexpectedly")
	}
	log.Info().
		Str("event_type", "bridge_stopped").
		Msg("bridge instance stopped")
}
