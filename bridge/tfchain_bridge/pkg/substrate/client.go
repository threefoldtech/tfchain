package substrate

import (
	"context"
	"errors"
	"fmt"
	"math/big"
	"time"

	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
	"github.com/rs/zerolog/log"
	substrate "github.com/threefoldtech/tfchain/clients/tfchain-client-go"
)

var (
	//ErrInvalidVersion is returned if version 4bytes is invalid
	ErrInvalidVersion = fmt.Errorf("invalid version")
	//ErrUnknownVersion is returned if version number is not supported
	ErrUnknownVersion = fmt.Errorf("unknown version")
	//ErrNotFound is returned if an object is not found
	ErrNotFound = fmt.Errorf("object not found")
)

// Versioned base for all types
type Versioned struct {
	Version uint32
}

type SubstrateClient struct {
	*substrate.Substrate
	identity substrate.Identity
}

// NewSubstrate creates a substrate client
func NewSubstrateClient(url string, seed string) (*SubstrateClient, error) {
	mngr := substrate.NewManager(url)
	cl, err := mngr.Substrate()
	if err != nil {
		return nil, err
	}
	tfchainIdentity, err := substrate.NewIdentityFromSr25519Phrase(seed)
	if err != nil {
		return nil, err
	}

	log.Info().Msgf("key with address %s loaded", tfchainIdentity.Address())

	isValidator, err := cl.IsValidator(tfchainIdentity)
	if err != nil {
		return nil, err
	}

	if !isValidator {
		return nil, fmt.Errorf("account provided is not a validator for the bridge runtime")
	}

	return &SubstrateClient{
		cl,
		tfchainIdentity,
	}, nil
}

func (s *SubstrateClient) RetrySetWithdrawExecuted(ctx context.Context, tixd uint64) error {
	err := s.SetBurnTransactionExecuted(s.identity, tixd)
	for err != nil {
		log.Err(err).Msg("error while setting refund transaction as executed")

		select {
		case <-ctx.Done():
			return err
		case <-time.After(10 * time.Second):
			burnedAlready, bErr := s.IsBurnedAlready(types.U64(tixd))
			if bErr != nil {
				return bErr
			}

			if !burnedAlready {
				err = s.SetBurnTransactionExecuted(s.identity, tixd)
			} else {
				err = nil
			}
		}
	}

	return nil
}

func (s *SubstrateClient) RetryProposeWithdrawOrAddSig(ctx context.Context, txID uint64, target string, amount *big.Int, signature string, stellarAddress string, sequence_number uint64) error {
	err := s.ProposeBurnTransactionOrAddSig(s.identity, txID, target, amount, signature, stellarAddress, sequence_number)
	for err != nil {
		log.Err(err).Msg("error while proposing withdraw or adding signature")

		select {
		case <-ctx.Done():
			return err
		case <-time.After(10 * time.Second):
			burnedAlready, bErr := s.IsBurnedAlready(types.U64(txID))
			if bErr != nil {
				return bErr
			}

			if !burnedAlready {
				err = s.ProposeBurnTransactionOrAddSig(s.identity, txID, target, amount, signature, stellarAddress, sequence_number)
			} else {
				err = nil
			}
		}
	}

	return nil
}

func (s *SubstrateClient) RetryCreateRefundTransactionOrAddSig(ctx context.Context, txHash string, target string, amount int64, signature string, stellarAddress string, sequence_number uint64) error {
	err := s.CreateRefundTransactionOrAddSig(s.identity, txHash, target, amount, signature, stellarAddress, sequence_number)
	for err != nil {
		log.Err(err).Msg("error while creating refund tx or adding signature")

		select {
		case <-ctx.Done():
			return err
		case <-time.After(10 * time.Second):
			refundedAlready, rErr := s.IsRefundedAlready(txHash)
			if rErr != nil {
				return rErr
			}

			if !refundedAlready {
				err = s.CreateRefundTransactionOrAddSig(s.identity, txHash, target, amount, signature, stellarAddress, sequence_number)
			} else {
				err = nil
			}

		}
	}

	return nil
}

func (s *SubstrateClient) RetrySetRefundTransactionExecutedTx(ctx context.Context, txHash string) error {
	err := s.SetRefundTransactionExecuted(s.identity, txHash)
	for err != nil {
		log.Err(err).Msg("error while setting refund transaction as executed")

		select {
		case <-ctx.Done():
			return err
		case <-time.After(10 * time.Second):
			refundedAlready, rErr := s.IsRefundedAlready(txHash)
			if rErr != nil {
				return rErr
			}

			if !refundedAlready {
				err = s.SetRefundTransactionExecuted(s.identity, txHash)
			} else {
				err = nil
			}
		}
	}

	return nil
}

func (s *SubstrateClient) RetryProposeMintOrVote(ctx context.Context, txID string, target substrate.AccountID, amount *big.Int) error {
	err := s.ProposeOrVoteMintTransaction(s.identity, txID, target, amount)
	for err != nil {
		log.Err(err).Msg("error while proposing mint or voting")

		select {
		case <-ctx.Done():
			return err
		case <-time.After(10 * time.Second):
			mintedAlready, mErr := s.IsMintedAlready(txID)
			if mErr != nil {
				if !errors.Is(mErr, substrate.ErrMintTransactionNotFound) {
					return err
				}
			}

			if !mintedAlready {
				err = s.ProposeOrVoteMintTransaction(s.identity, txID, target, amount)
			} else {
				err = nil
			}
		}
	}

	return nil
}
