package stellar

import (
	"testing"

	hProtocol "github.com/stellar/go/protocols/horizon"
)

func TestIsTerminalSubmitError(t *testing.T) {
	tests := []struct {
		name  string
		codes *hProtocol.TransactionResultCodes
		want  bool
	}{
		{
			name:  "nil codes are not terminal",
			codes: nil,
			want:  false,
		},
		{
			name: "op_no_trust is terminal (target removed trustline)",
			codes: &hProtocol.TransactionResultCodes{
				TransactionCode: "tx_failed",
				OperationCodes:  []string{"op_no_trust"},
			},
			want: true,
		},
		{
			name: "op_no_destination is terminal (target account gone)",
			codes: &hProtocol.TransactionResultCodes{
				TransactionCode: "tx_failed",
				OperationCodes:  []string{"op_no_destination"},
			},
			want: true,
		},
		{
			name: "op_not_authorized is terminal (target not authorized to hold the asset)",
			codes: &hProtocol.TransactionResultCodes{
				TransactionCode: "tx_failed",
				OperationCodes:  []string{"op_not_authorized"},
			},
			want: true,
		},
		{
			name: "op_line_full is NOT terminal (target trustline momentarily full; may succeed on retry - postpone, do not forfeit)",
			codes: &hProtocol.TransactionResultCodes{
				TransactionCode: "tx_failed",
				OperationCodes:  []string{"op_line_full"},
			},
			want: false,
		},
		{
			name: "op_underfunded is NOT terminal (bridge wallet out of funds - operational, must alert not forfeit)",
			codes: &hProtocol.TransactionResultCodes{
				TransactionCode: "tx_failed",
				OperationCodes:  []string{"op_underfunded"},
			},
			want: false,
		},
		{
			name: "op_src_no_trust is NOT terminal (bridge-side trustline missing, not the target's fault)",
			codes: &hProtocol.TransactionResultCodes{
				TransactionCode: "tx_failed",
				OperationCodes:  []string{"op_src_no_trust"},
			},
			want: false,
		},
		{
			name: "tx_bad_seq is NOT terminal (transient, resolved by sequence reset)",
			codes: &hProtocol.TransactionResultCodes{
				TransactionCode: "tx_bad_seq",
				OperationCodes:  nil,
			},
			want: false,
		},
		{
			name: "terminal code mixed with others is still terminal",
			codes: &hProtocol.TransactionResultCodes{
				TransactionCode: "tx_failed",
				OperationCodes:  []string{"op_success", "op_no_trust"},
			},
			want: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := isTerminalSubmitError(tt.codes); got != tt.want {
				t.Errorf("isTerminalSubmitError() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestIsInsufficientFundsError(t *testing.T) {
	tests := []struct {
		name  string
		codes *hProtocol.TransactionResultCodes
		want  bool
	}{
		{
			name:  "nil codes are not an underfunded condition",
			codes: nil,
			want:  false,
		},
		{
			name: "op_underfunded means the bridge wallet is out of funds",
			codes: &hProtocol.TransactionResultCodes{
				TransactionCode: "tx_failed",
				OperationCodes:  []string{"op_underfunded"},
			},
			want: true,
		},
		{
			name: "op_no_trust is a target-side failure, not underfunded",
			codes: &hProtocol.TransactionResultCodes{
				TransactionCode: "tx_failed",
				OperationCodes:  []string{"op_no_trust"},
			},
			want: false,
		},
		{
			name: "op_underfunded mixed with others is still underfunded",
			codes: &hProtocol.TransactionResultCodes{
				TransactionCode: "tx_failed",
				OperationCodes:  []string{"op_success", "op_underfunded"},
			},
			want: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := isInsufficientFundsError(tt.codes); got != tt.want {
				t.Errorf("isInsufficientFundsError() = %v, want %v", got, tt.want)
			}
		})
	}
}
