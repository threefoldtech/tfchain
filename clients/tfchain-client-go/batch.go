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

// BatchCalls submits multiple calls in a single Utility.batch extrinsic.
// Unlike Utility.batch_all, individual call failures do NOT abort the entire batch
// (on newer runtimes that emit ItemFailed). On older runtimes, BatchInterrupted
// stops at the first failure.
func (s *Substrate) BatchCalls(identity Identity, calls []types.Call) (*BatchResult, error) {
	if len(calls) == 0 {
		return &BatchResult{}, nil
	}

	cl, meta, err := s.GetClient()
	if err != nil {
		return nil, err
	}

	batchCall, err := types.NewCall(meta, "Utility.batch", calls)
	if err != nil {
		return nil, errors.Wrap(err, "failed to create batch call")
	}

	resp, err := s.Call(cl, meta, identity, batchCall)
	if err != nil {
		return nil, errors.Wrap(err, "failed to execute batch call")
	}

	result := &BatchResult{
		SuccessCount: len(calls),
	}

	if resp.Events != nil {
		// ItemFailed events tell us how many calls failed, but the event payload
		// does not carry the batch-call index — only a DispatchError. We count
		// failures but cannot reliably map them to specific call positions.
		failedCount := len(resp.Events.Utility_ItemFailed)
		if failedCount > 0 {
			result.FailedCount = failedCount
			result.SuccessCount = len(calls) - failedCount
		}

		// BatchInterrupted (older runtimes): stops at the first failure.
		// The Index field tells us exactly which call failed.
		if len(resp.Events.Utility_BatchInterrupted) > 0 {
			interruptedIdx := int(resp.Events.Utility_BatchInterrupted[0].Index)
			// Everything from interruptedIdx onward was not executed
			notExecuted := len(calls) - interruptedIdx
			if notExecuted > result.FailedCount {
				result.FailedCount = notExecuted
				result.SuccessCount = interruptedIdx
			}
			for i := interruptedIdx; i < len(calls); i++ {
				result.FailedIndexes = append(result.FailedIndexes, i)
			}
		}
	}

	return result, nil
}
