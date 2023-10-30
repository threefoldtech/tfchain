package substrate

import (
	"context"
	"fmt"
	"strings"
	"time"

	"github.com/cenkalti/backoff/v4"
	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
	"github.com/stellar/go/support/errors"
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
	logger := log.Logger.With().Str("event_type", "FetchTfchainBridgeEvents").Logger()

	cl, _, err := client.GetClient()
	if err != nil {
		return errors.Wrap(err, "failed to get client")
	}

	chainHeadsSub, err := cl.RPC.Chain.SubscribeFinalizedHeads()
	if err != nil {
		return errors.Wrap(err, "failed to subscribe to finalized heads")
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

			bo := backoff.NewExponentialBackOff()
			bo.MaxElapsedTime = time.Duration(time.Minute * 10) // 10 minutes
			_ = backoff.RetryNotify(func() error {
				chainHeadsSub, err = cl.RPC.Chain.SubscribeFinalizedHeads()
				return err
			}, bo, func(err error, d time.Duration) {
				logger.Warn().Err(err).Str("event_type", "fetch_finalizedHead_failed").Msgf("connection to chain lost, reopening connection in %s", d.String())
			})

		case <-ctx.Done():
			chainHeadsSub.Unsubscribe()
			return ctx.Err()
		}
	}
}

func (client *SubstrateClient) processEventsForHeight(height uint32) (Events, error) {

	if height == 0 {
		return Events{}, nil
	}

	records, err := client.GetEventsForBlock(height)
	if err != nil {
		return Events{}, errors.Wrapf(err, "error while decoding block for height %d", height)

	}
	log.Info().
		Str("event_type", "block_events_fetched").
		Dict("event", zerolog.Dict().
			Uint32("height", height)).
		Msg("tfchain events fetched")

	return client.processEventRecords(records), nil
}

func (client *SubstrateClient) processEventRecords(events *substrate.EventRecords) Events {
	var refundTransactionReadyEvents []RefundTransactionReadyEvent
	var refundTransactionExpiredEvents []RefundTransactionExpiredEvent
	var withdrawCreatedEvents []WithdrawCreatedEvent
	var withdrawReadyEvents []WithdrawReadyEvent
	var withdrawExpiredEvents []WithdrawExpiredEvent

	for _, e := range events.TFTBridgeModule_RefundTransactionReady {
		log.Info().
			Str("event_type", "event_refund_tx_ready_received").
			Str("span_id", string(e.RefundTransactionHash)).
			Msg("found RefundTransactionReady event")
		refundTransactionReadyEvents = append(refundTransactionReadyEvents, RefundTransactionReadyEvent{
			Hash: string(e.RefundTransactionHash),
		})
	}

	for _, e := range events.TFTBridgeModule_RefundTransactionExpired {
		log.Info().
			Str("event_type", "event_refund_tx_expired_received").
			Str("span_id", string(e.RefundTransactionHash)).
			Msgf("found RefundTransactionExpired event")
		refundTransactionExpiredEvents = append(refundTransactionExpiredEvents, RefundTransactionExpiredEvent{
			Hash:   string(e.RefundTransactionHash),
			Target: string(e.Target),
			Amount: uint64(e.Amount),
		})
	}

	for _, e := range events.TFTBridgeModule_BurnTransactionCreated {
		log.Info().
			Str("event_type", "event_burn_tx_created_received").
			Str("span_id", fmt.Sprint(e.BurnTransactionID)).
			Msg("found BurnTransactionCreated event")
		withdrawCreatedEvents = append(withdrawCreatedEvents, WithdrawCreatedEvent{
			ID:     uint64(e.BurnTransactionID),
			Source: e.Source,
			Target: string(e.Target),
			Amount: uint64(e.Amount),
		})
	}

	for _, e := range events.TFTBridgeModule_BurnTransactionReady {
		log.Info().
			Str("event_type", "event_burn_tx_ready_received").
			Str("span_id", fmt.Sprint(e.BurnTransactionID)).
			Msg("found BurnTransactionReady event")
		withdrawReadyEvents = append(withdrawReadyEvents, WithdrawReadyEvent{
			ID: uint64(e.BurnTransactionID),
		})
	}

	for _, e := range events.TFTBridgeModule_BurnTransactionExpired {
		log.Info().
			Str("event_type", "event_burn_tx_expired_received").
			Str("span_id", fmt.Sprint(e.BurnTransactionID)).
			Msg("found BurnTransactionExpired event")
		withdrawExpiredEvents = append(withdrawExpiredEvents, WithdrawExpiredEvent{
			ID:     uint64(e.BurnTransactionID),
			Target: string(e.Target),
			Amount: uint64(e.Amount),
		})
	}

	for range events.TFTBridgeModule_MintCompleted {
		span_id := "TODO" // TODO: GET tx id from the event. required tfchain update
		logger := log.Logger.With().Str("span_id", span_id).Logger()
		outcome := ""
		if strings.HasPrefix(span_id, "refund") {
			outcome = "refunded"
		} else {
			outcome = "bridged"
		}

		logger.Info().
			Str("event_type", "mint_completed").
			Msg("found MintCompleted event")
		logger.Info().
			Str("event_type", "transfer_completed").
			Dict("event", zerolog.Dict().
				Str("outcome", outcome)).
			Msgf("transfer with id %s completed", span_id)
	}

	return Events{
		WithdrawCreatedEvents: withdrawCreatedEvents,
		WithdrawReadyEvents:   withdrawReadyEvents,
		WithdrawExpiredEvents: withdrawExpiredEvents,
		RefundReadyEvents:     refundTransactionReadyEvents,
		RefundExpiredEvents:   refundTransactionExpiredEvents,
	}
}
