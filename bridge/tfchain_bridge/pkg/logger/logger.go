package logger

import (
	"os"

	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
)

const VERSION = 1

func Init_logger(isDebug bool) {
	log.Logger = zerolog.New(os.Stdout).With().Timestamp().Uint("version", VERSION).Logger()
	logLevel := zerolog.InfoLevel
	if isDebug {
		logLevel = zerolog.DebugLevel
		log.Logger = log.Logger.With().Caller().Logger()
	}

	zerolog.SetGlobalLevel(logLevel)
}

type SourceCommonLogEntry struct {
	Instance_public_key   string
	Bridge_wallet_address string
	Stellar_network       string
	Tfchain_url           string
}

// TODO: event log interfaces
