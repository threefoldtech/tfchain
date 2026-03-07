package pkg

import (
	"encoding/json"
	"fmt"
	"strconv"

	bolt "go.etcd.io/bbolt"
)

// TxState represents the processing state of a bridge transaction.
type TxState string

const (
	// TxStateProcessing means the Stellar transaction has been (or is being) submitted,
	// but TFChain confirmation has not yet been recorded.
	TxStateProcessing TxState = "PROCESSING"
	// TxStateCompleted means both Stellar submission and TFChain confirmation are done.
	TxStateCompleted TxState = "COMPLETED"
)

var (
	bucketWithdraw = []byte("withdraw")
	bucketRefund   = []byte("refund")
)

// IdempotencyStore provides crash-safe tracking of transaction processing state
// using a bbolt (BoltDB) embedded database. It prevents double Stellar submissions
// when the bridge crashes between Stellar tx submit and TFChain confirmation.
type IdempotencyStore struct {
	db *bolt.DB
}

// NewIdempotencyStore opens or creates the bbolt database at the given path.
func NewIdempotencyStore(path string) (*IdempotencyStore, error) {
	db, err := bolt.Open(path, 0600, nil)
	if err != nil {
		return nil, fmt.Errorf("failed to open idempotency store at %s: %w", path, err)
	}

	err = db.Update(func(tx *bolt.Tx) error {
		if _, err := tx.CreateBucketIfNotExists(bucketWithdraw); err != nil {
			return err
		}
		_, err := tx.CreateBucketIfNotExists(bucketRefund)
		return err
	})
	if err != nil {
		db.Close()
		return nil, err
	}

	return &IdempotencyStore{db: db}, nil
}

// MarkWithdrawProcessing records that a withdraw is about to be submitted to Stellar.
func (s *IdempotencyStore) MarkWithdrawProcessing(txID uint64) error {
	return s.setState(bucketWithdraw, strconv.FormatUint(txID, 10), TxStateProcessing)
}

// MarkWithdrawCompleted records that a withdraw has been fully processed
// (Stellar tx submitted AND TFChain confirmation recorded).
func (s *IdempotencyStore) MarkWithdrawCompleted(txID uint64) error {
	return s.setState(bucketWithdraw, strconv.FormatUint(txID, 10), TxStateCompleted)
}

// GetWithdrawState returns the current state of a withdraw transaction.
// Returns empty string if the transaction has never been tracked.
func (s *IdempotencyStore) GetWithdrawState(txID uint64) (TxState, error) {
	return s.getState(bucketWithdraw, strconv.FormatUint(txID, 10))
}

// GetPendingWithdraws returns all withdraw transaction IDs that are in PROCESSING state.
// These are candidates for crash recovery reconciliation.
func (s *IdempotencyStore) GetPendingWithdraws() ([]uint64, error) {
	var pending []uint64
	err := s.db.View(func(tx *bolt.Tx) error {
		b := tx.Bucket(bucketWithdraw)
		return b.ForEach(func(k, v []byte) error {
			var state TxState
			if err := json.Unmarshal(v, &state); err != nil {
				return nil // skip corrupted entries
			}
			if state == TxStateProcessing {
				id, err := strconv.ParseUint(string(k), 10, 64)
				if err != nil {
					return nil // skip non-numeric keys
				}
				pending = append(pending, id)
			}
			return nil
		})
	})
	return pending, err
}

// MarkRefundProcessing records that a refund is about to be submitted to Stellar.
func (s *IdempotencyStore) MarkRefundProcessing(txHash string) error {
	return s.setState(bucketRefund, txHash, TxStateProcessing)
}

// MarkRefundCompleted records that a refund has been fully processed.
func (s *IdempotencyStore) MarkRefundCompleted(txHash string) error {
	return s.setState(bucketRefund, txHash, TxStateCompleted)
}

// GetRefundState returns the current state of a refund transaction.
func (s *IdempotencyStore) GetRefundState(txHash string) (TxState, error) {
	return s.getState(bucketRefund, txHash)
}

// GetPendingRefunds returns all refund transaction hashes that are in PROCESSING state.
func (s *IdempotencyStore) GetPendingRefunds() ([]string, error) {
	var pending []string
	err := s.db.View(func(tx *bolt.Tx) error {
		b := tx.Bucket(bucketRefund)
		return b.ForEach(func(k, v []byte) error {
			var state TxState
			if err := json.Unmarshal(v, &state); err != nil {
				return nil
			}
			if state == TxStateProcessing {
				pending = append(pending, string(k))
			}
			return nil
		})
	})
	return pending, err
}

// Close closes the underlying bbolt database.
func (s *IdempotencyStore) Close() error {
	return s.db.Close()
}

func (s *IdempotencyStore) setState(bucket []byte, key string, state TxState) error {
	return s.db.Update(func(tx *bolt.Tx) error {
		b := tx.Bucket(bucket)

		// Guard against downgrading a COMPLETED entry back to PROCESSING.
		// This should never happen via normal code paths (callers check state first),
		// but we enforce it at the store level as a safety net.
		if state == TxStateProcessing {
			existing := b.Get([]byte(key))
			if existing != nil {
				var cur TxState
				if err := json.Unmarshal(existing, &cur); err == nil && cur == TxStateCompleted {
					return fmt.Errorf("refusing to downgrade completed tx %q to PROCESSING", key)
				}
			}
		}

		val, err := json.Marshal(state)
		if err != nil {
			return err
		}
		return b.Put([]byte(key), val)
	})
}

func (s *IdempotencyStore) getState(bucket []byte, key string) (TxState, error) {
	var state TxState
	err := s.db.View(func(tx *bolt.Tx) error {
		b := tx.Bucket(bucket)
		val := b.Get([]byte(key))
		if val == nil {
			return nil // state remains zero value (empty string)
		}
		return json.Unmarshal(val, &state)
	})
	return state, err
}
