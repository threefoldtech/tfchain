package bridge

import (
	"context"

	"github.com/pkg/errors"
	"github.com/rs/zerolog/log"
	"github.com/threefoldtech/tfchain_bridge/pkg"
	"github.com/threefoldtech/tfchain_bridge/pkg/stellar"
	subpkg "github.com/threefoldtech/tfchain_bridge/pkg/substrate"
)

const (
	BridgeNetwork = "stellar"
)

// Bridge is a high lvl structure which listens on contract events and bridge-related
// stellar transactions, and handles them
type Bridge struct {
	wallet           *stellar.StellarWallet
	subClient        *subpkg.SubstrateClient
	blockPersistency *pkg.ChainPersistency
	config           *pkg.BridgeConfig
	depositFee       int64
}

func NewBridge(ctx context.Context, cfg pkg.BridgeConfig) (*Bridge, error) {
	subClient, err := subpkg.NewSubstrateClient(cfg.TfchainURL, cfg.TfchainSeed)
	if err != nil {
		return nil, err
	}

	blockPersistency, err := pkg.InitPersist(cfg.PersistencyFile)
	if err != nil {
		return nil, err
	}

	wallet, err := stellar.NewStellarWallet(ctx, &cfg.StellarConfig)
	if err != nil {
		return nil, err
	}

	if cfg.RescanBridgeAccount {
		// saving the cursor to 0 will trigger the bridge stellar account
		// to scan for every transaction ever made on the bridge account
		// and mint accordingly
		err = blockPersistency.SaveStellarCursor("0")
		if err != nil {
			return nil, err
		}
		err = blockPersistency.SaveHeight(0)
		if err != nil {
			return nil, err
		}
	}

	// fetch the configured depositfee
	depositFee, err := subClient.GetDepositFee()
	if err != nil {
		return nil, err
	}

	bridge := &Bridge{
		subClient:        subClient,
		blockPersistency: blockPersistency,
		wallet:           wallet,
		config:           &cfg,
		depositFee:       depositFee,
	}

	return bridge, nil
}

func (bridge *Bridge) Start(ctx context.Context) error {
	height, err := bridge.blockPersistency.GetHeight()
	if err != nil {
		return errors.Wrap(err, "failed to get block height from persistency")
	}

	log.Info().Msg("starting stellar subscription...")
	stellarSub := make(chan stellar.MintEventSubscription)
	go func() {
		defer close(stellarSub)
		if err = bridge.wallet.StreamBridgeStellarTransactions(ctx, stellarSub, height.StellarCursor); err != nil {
			log.Fatal().Msgf("failed to monitor bridge account %s", err.Error())
		}
	}()

	log.Info().Msg("starting tfchain subscription...")
	tfchainSub := make(chan subpkg.EventSubscription)
	go func() {
		defer close(tfchainSub)
		if err := bridge.subClient.SubscribeTfchainBridgeEvents(ctx, tfchainSub); err != nil {
			log.Fatal().Msgf("failed to subscribe to tfchain %s", err.Error())
		}
	}()

	for {
		select {
		case data := <-tfchainSub:
			if data.Err != nil {
				return errors.Wrap(err, "failed to process events")
			}
			for _, withdrawCreatedEvent := range data.Events.WithdrawCreatedEvents {
				err := bridge.handleWithdrawCreated(ctx, withdrawCreatedEvent)
				if err != nil {
					// If the TX is already withdrawn or refunded (minted on tfchain) skip
					if errors.Is(err, pkg.ErrTransactionAlreadyBurned) || errors.Is(err, pkg.ErrTransactionAlreadyMinted) {
						continue
					}
					return errors.Wrap(err, "failed to handle withdraw created")
				}
			}
			for _, withdrawExpiredEvent := range data.Events.WithdrawExpiredEvents {
				err := bridge.handleWithdrawExpired(ctx, withdrawExpiredEvent)
				if err != nil {
					return errors.Wrap(err, "failed to handle withdraw expired")
				}
			}
			for _, withdawReadyEvent := range data.Events.WithdrawReadyEvents {
				err := bridge.handleWithdrawReady(ctx, withdawReadyEvent)
				if err != nil {
					if errors.Is(err, pkg.ErrTransactionAlreadyBurned) {
						continue
					}
					return errors.Wrap(err, "failed to handle withdraw ready")
				}
				log.Info().Uint64("ID", withdawReadyEvent.ID).Msg("withdraw processed")
			}
			for _, refundExpiredEvent := range data.Events.RefundExpiredEvents {
				err := bridge.handleRefundExpired(ctx, refundExpiredEvent)
				if err != nil {
					return errors.Wrap(err, "failed to handle refund expired")
				}
			}
			for _, refundReadyEvent := range data.Events.RefundReadyEvents {
				err := bridge.handleRefundReady(ctx, refundReadyEvent)
				if err != nil {
					if errors.Is(err, pkg.ErrTransactionAlreadyRefunded) {
						continue
					}
					return errors.Wrap(err, "failed to handle refund ready")
				}
				log.Info().Str("hash", refundReadyEvent.Hash).Msg("refund processed")
			}
		case data := <-stellarSub:
			if data.Err != nil {
				return errors.Wrap(err, "failed to get mint events")
			}

			for _, mEvent := range data.Events {
				err := bridge.mint(ctx, mEvent.Senders, mEvent.Tx)
				if err != nil {
					if errors.Is(err, pkg.ErrTransactionAlreadyMinted) {
						continue
					}
					return errors.Wrap(err, "failed to handle mint")
				}
				log.Info().Str("hash", mEvent.Tx.Hash).Msg("mint processed")
			}
		case <-ctx.Done():
			return ctx.Err()
		}
	}
}
