package substrate

import (
	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
	"github.com/pkg/errors"
)

// BatchResult holds the outcome of a Utility.batch call.
type BatchResult struct {
	SuccessCount int
	FailedCount  int
	// FailedIndexes is only populated for BatchInterrupted (older runtimes),
	// where we know the exact index that caused interruption.
	FailedIndexes []int
}

// BatchCalls submits multiple calls in a single Utility.force_batch extrinsic.
// Unlike Utility.batch_all (aborts + reverts on first failure) and Utility.batch
// (stops at first failure, remaining calls not executed), Utility.force_batch
// continues through all calls regardless of individual failures. Failed calls emit
// Utility.ItemFailed events; the overall batch always completes.
// This is the correct choice for bridge proposals: a BurnSignatureExists error on
// one proposal must not prevent the remaining proposals from being submitted.
func (s *Substrate) BatchCalls(identity Identity, calls []types.Call) (*BatchResult, error) {
	if len(calls) == 0 {
		return &BatchResult{}, nil
	}

	cl, meta, err := s.GetClient()
	if err != nil {
		return nil, err
	}

	batchCall, err := types.NewCall(meta, "Utility.force_batch", calls)
	if err != nil {
		return nil, errors.Wrap(err, "failed to create force_batch call")
	}

	resp, err := s.Call(cl, meta, identity, batchCall)
	if err != nil {
		return nil, errors.Wrap(err, "failed to execute force_batch call")
	}

	result := &BatchResult{
		SuccessCount: len(calls),
	}

	if resp.Events != nil {
		// ItemFailed events are emitted by force_batch for each failed call.
		// The event payload does not carry the batch-call index — only a DispatchError.
		// We count failures but cannot reliably map them to specific call positions.
		failedCount := len(resp.Events.Utility_ItemFailed)
		if failedCount > 0 {
			result.FailedCount = failedCount
			result.SuccessCount = len(calls) - failedCount
		}
	}

	return result, nil
}
