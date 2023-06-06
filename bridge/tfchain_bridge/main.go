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

	log.Logger = log.Output(zerolog.ConsoleWriter{Out: os.Stdout})
	zerolog.SetGlobalLevel(zerolog.InfoLevel)
	if debug {
		zerolog.SetGlobalLevel(zerolog.DebugLevel)
		log.Debug().Msg("debug mode enabled")
	}

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	timeout, timeoutCancel := context.WithTimeout(ctx, time.Second*15)
	defer timeoutCancel()

	br, err := bridge.NewBridge(timeout, bridgeCfg)
	if err != nil {
		panic(err)
	}

	sigs := make(chan os.Signal, 1)

	signal.Notify(sigs, syscall.SIGINT, syscall.SIGTERM)

	go func() {
		log.Info().Msg("awaiting signal")
		<-sigs
		log.Info().Msg("shutting now")
		cancel()
	}()

	if err = br.Start(ctx); err != nil && err != context.Canceled {
		log.Fatal().Err(err).Msg("exited unexpectedly")
	}
}
