package substrate

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestSignTC(t *testing.T) {

	const (
		tcUrl  = "http://zos.tf/terms/v0.1"
		tcHash = "9021d4dee05a661e2cb6838152c67f25"
	)

	cl := startLocalConnection(t)

	identity, err := NewIdentityFromSr25519Phrase(BobMnemonics)
	require.NoError(t, err)

	err = cl.AcceptTermsAndConditions(identity, tcUrl, tcHash)
	require.NoError(t, err)
}
