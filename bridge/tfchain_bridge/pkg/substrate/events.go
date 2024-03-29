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
	Source types.OptionAccountID
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
		return errors.Wrap(err, "an error occurred while getting substrate client")
	}

	chainHeadsSub, err := cl.RPC.Chain.SubscribeFinalizedHeads()
	if err != nil {
		return errors.Wrap(err, "an error occurred while subscribing to finalized heads")
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
				log.Warn().
					Err(err).
					Str("event_action", "fetch_finalized_Heads_failed").
					Str("event_kind", "alert").
					Str("category", "tfchain_monitor").
					Msgf("connection to chain lost, reopening connection in %s", d.String())
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
		return Events{}, errors.Wrapf(err, "an error occurred while decoding events for height %d", height)

	}
	log.Info().
		Str("event_action", "block_events_fetched").
		Str("event_kind", "event").
		Str("category", "tfchain_monitor").
		Dict("metadata", zerolog.Dict().
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
			Str("trace_id", string(e.RefundTransactionHash)).
			Str("event_action", "event_refund_tx_ready_received").
			Str("event_kind", "event").
			Str("category", "refund").
			Msg("found RefundTransactionReady event")
		refundTransactionReadyEvents = append(refundTransactionReadyEvents, RefundTransactionReadyEvent{
			Hash: string(e.RefundTransactionHash),
		})
	}

	for _, e := range events.TFTBridgeModule_RefundTransactionExpired {
		log.Info().
			Str("trace_id", string(e.RefundTransactionHash)).
			Str("event_action", "event_refund_tx_expired_received").
			Str("event_kind", "alert").
			Str("category", "refund").
			Msgf("found RefundTransactionExpired event")
		refundTransactionExpiredEvents = append(refundTransactionExpiredEvents, RefundTransactionExpiredEvent{
			Hash:   string(e.RefundTransactionHash),
			Target: string(e.Target),
			Amount: uint64(e.Amount),
		})
	}

	for _, e := range events.TFTBridgeModule_BurnTransactionCreated {
		log.Info().
			Str("trace_id", fmt.Sprint(e.BurnTransactionID)).
			Str("event_action", "event_burn_tx_created_received").
			Str("event_kind", "event").
			Str("category", "withdraw").
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
			Str("trace_id", fmt.Sprint(e.BurnTransactionID)).
			Str("event_action", "event_burn_tx_ready_received").
			Str("event_kind", "event").
			Str("category", "withdraw").
			Msg("found BurnTransactionReady event")
		withdrawReadyEvents = append(withdrawReadyEvents, WithdrawReadyEvent{
			ID: uint64(e.BurnTransactionID),
		})
	}

	for _, e := range events.TFTBridgeModule_BurnTransactionExpired {
		log.Info().
			Str("trace_id", fmt.Sprint(e.BurnTransactionID)).
			Str("event_action", "event_burn_tx_expired_received").
			Str("event_kind", "alert").
			Str("category", "withdraw").
			Msg("found BurnTransactionExpired event")
		withdrawExpiredEvents = append(withdrawExpiredEvents, WithdrawExpiredEvent{
			ID:     uint64(e.BurnTransactionID),
			Source: e.Source,
			Target: string(e.Target),
			Amount: uint64(e.Amount),
		})
	}

	for _, e := range events.TFTBridgeModule_MintCompleted {
		trace_id := e.TxHash
		logger := log.Logger.With().Str("trace_id", strings.TrimLeft(trace_id, "refund-")).Logger()
		outcome := ""
		if strings.HasPrefix(trace_id, "refund") {
			outcome = "refunded"
		} else {
			outcome = "bridged"
		}

		logger.Info().
			Str("event_action", "mint_completed").
			Str("event_kind", "event").
			Str("category", "mint").
			Msg("found MintCompleted event")

		logger.Info().
			Str("event_action", "transfer_completed").
			Str("event_kind", "event").
			Str("category", "transfer").
			Dict("metadata", zerolog.Dict().
				Str("outcome", outcome)).
			Msg("transfer has completed")
	}

	return Events{
		WithdrawCreatedEvents: withdrawCreatedEvents,
		WithdrawReadyEvents:   withdrawReadyEvents,
		WithdrawExpiredEvents: withdrawExpiredEvents,
		RefundReadyEvents:     refundTransactionReadyEvents,
		RefundExpiredEvents:   refundTransactionExpiredEvents,
	}
}
