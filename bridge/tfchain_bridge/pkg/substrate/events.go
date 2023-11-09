package substrate

import (
	"context"
	"time"

	"github.com/cenkalti/backoff/v4"
	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
	"github.com/rs/zerolog/log"
	substrate "github.com/threefoldtech/tfchain/clients/tfchain-client-go"
)

type EventSubscription struct {
	Events Events
	Err    error
}

type Events struct {
	WithdrawCreatedEvents []WithdrawCreatedEvent
	WithdrawReadyEvents   []WithdrawReadyEvent
	WithdrawExpiredEvents []WithdrawExpiredEvent
	RefundCreatedEvents   []RefundTransactionCreatedEvent
	RefundReadyEvents     []RefundTransactionReadyEvent
	RefundExpiredEvents   []RefundTransactionExpiredEvent
}

type WithdrawCreatedEvent struct {
	ID     uint64
	Source types.AccountID
	Target string
	Amount uint64
}

type WithdrawReadyEvent struct {
	ID uint64
}

type WithdrawExpiredEvent struct {
	ID     uint64
	Source types.AccountID
	Target string
	Amount uint64
}

type RefundTransactionCreatedEvent struct {
	Hash   string
	Target string
	Amount uint64
	TxID   string
}

type RefundTransactionReadyEvent struct {
	Hash string
}

type RefundTransactionExpiredEvent struct {
	Hash   string
	Target string
	Amount uint64
}

func (client *SubstrateClient) SubscribeTfchainBridgeEvents(ctx context.Context, eventChannel chan<- EventSubscription) error {
	cl, _, err := client.GetClient()
	if err != nil {
		log.Fatal().Msg("failed to get client")
	}

	chainHeadsSub, err := cl.RPC.Chain.SubscribeFinalizedHeads()
	if err != nil {
		log.Fatal().Msg("failed to subscribe to finalized heads")
	}

	for {
		select {
		case head := <-chainHeadsSub.Chan():
			events, err := client.processEventsForHeight(uint32(head.Number))
			data := EventSubscription{
				Events: events,
				Err:    err,
			}
			eventChannel <- data
		case err := <-chainHeadsSub.Err():
			log.Err(err).Msg("error with subscription")

			bo := backoff.NewExponentialBackOff()
			bo.MaxElapsedTime = time.Duration(time.Minute * 10) // 10 minutes
			_ = backoff.RetryNotify(func() error {
				chainHeadsSub, err = cl.RPC.Chain.SubscribeFinalizedHeads()
				return err
			}, bo, func(err error, d time.Duration) {
				log.Warn().Err(err).Msgf("connection to chain lost, reopening connection in %s", d.String())
			})

		case <-ctx.Done():
			chainHeadsSub.Unsubscribe()
			return ctx.Err()
		}
	}
}

func (client *SubstrateClient) processEventsForHeight(height uint32) (Events, error) {
	log.Info().Uint32("ID", height).Msg("fetching events for blockheight")
	if height == 0 {
		return Events{}, nil
	}

	records, err := client.GetEventsForBlock(height)
	if err != nil {
		log.Err(err).Uint32("ID", height).Msg("failed to decode block for height")
		return Events{}, err
	}

	return client.processEventRecords(records), nil
}

func (client *SubstrateClient) processEventRecords(events *substrate.EventRecords) Events {
	var refundTransactionReadyEvents []RefundTransactionReadyEvent
	var refundTransactionExpiredEvents []RefundTransactionExpiredEvent
	var withdrawCreatedEvents []WithdrawCreatedEvent
	var withdrawReadyEvents []WithdrawReadyEvent
	var withdrawExpiredEvents []WithdrawExpiredEvent

	for _, e := range events.TFTBridgeModule_RefundTransactionReady {
		log.Info().Str("hash", string(e.RefundTransactionHash)).Msg("found refund transaction ready event")
		refundTransactionReadyEvents = append(refundTransactionReadyEvents, RefundTransactionReadyEvent{
			Hash: string(e.RefundTransactionHash),
		})
	}

	for _, e := range events.TFTBridgeModule_RefundTransactionExpired {
		log.Info().Str("hash", string(e.RefundTransactionHash)).Msgf("found expired refund transaction")
		refundTransactionExpiredEvents = append(refundTransactionExpiredEvents, RefundTransactionExpiredEvent{
			Hash:   string(e.RefundTransactionHash),
			Target: string(e.Target),
			Amount: uint64(e.Amount),
		})
	}

	for _, e := range events.TFTBridgeModule_BurnTransactionCreated {
		log.Info().Uint64("ID", uint64(e.BurnTransactionID)).Msg("found burn transaction created event")
		withdrawCreatedEvents = append(withdrawCreatedEvents, WithdrawCreatedEvent{
			ID:     uint64(e.BurnTransactionID),
			Source: e.Source,
			Target: string(e.Target),
			Amount: uint64(e.Amount),
		})
	}

	for _, e := range events.TFTBridgeModule_BurnTransactionReady {
		log.Info().Uint64("ID", uint64(e.BurnTransactionID)).Msg("found burn transaction ready event")
		withdrawReadyEvents = append(withdrawReadyEvents, WithdrawReadyEvent{
			ID: uint64(e.BurnTransactionID),
		})
	}

	for _, e := range events.TFTBridgeModule_BurnTransactionExpired {
		log.Info().Uint64("ID", uint64(e.BurnTransactionID)).Msg("found burn transaction expired event")
		withdrawExpiredEvents = append(withdrawExpiredEvents, WithdrawExpiredEvent{
			ID:     uint64(e.BurnTransactionID),
			Target: string(e.Target),
			Amount: uint64(e.Amount),
		})
	}

	return Events{
		WithdrawCreatedEvents: withdrawCreatedEvents,
		WithdrawReadyEvents:   withdrawReadyEvents,
		WithdrawExpiredEvents: withdrawExpiredEvents,
		RefundReadyEvents:     refundTransactionReadyEvents,
		RefundExpiredEvents:   refundTransactionExpiredEvents,
	}
}
