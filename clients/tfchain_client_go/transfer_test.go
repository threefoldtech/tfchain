package substrate

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestTransfer(t *testing.T) {
	cl := startLocalConnection(t)
	defer cl.Close()

	alice := Accounts[AccountAlice]
	bob := Accounts[AccountBob]

	identity, err := NewIdentityFromSr25519Phrase(alice.Phrase)
	require.NoError(t, err)

	bobAddress, err := FromAddress(bob.Address)
	require.NoError(t, err)

	err = cl.Transfer(identity, 10000000, bobAddress)
	require.NoError(t, err)
}
