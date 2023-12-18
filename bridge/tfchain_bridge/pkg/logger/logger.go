package logger

import (
	"context"
	"os"

	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
)

const VERSION = 1

func InitLogger(isDebug bool) {
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

type refundReasonKey struct{}

func WithRefundReason(ctx context.Context, reason string) context.Context {
	return context.WithValue(ctx, refundReasonKey{}, reason)
}

func GetRefundReason(ctx context.Context) string {
	return ctx.Value(refundReasonKey{}).(string)
}

// TODO: event log interfaces
