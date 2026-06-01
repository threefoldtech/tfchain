package substrate

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestTwin(t *testing.T) {

	cl := startLocalConnection(t)
	defer cl.Close()

	twinID := assertCreateTwin(t, cl, AccountBob)

	twin, err := cl.GetTwin(twinID)

	require.NoError(t, err)
	require.Equal(t, twinID, uint32(twin.ID))

	id, err := cl.GetTwinByPubKey(twin.Account.PublicKey())
	require.NoError(t, err)

	require.Equal(t, uint32(twin.ID), id)
}

func TestBondTwinAccount(t *testing.T) {
	cl := startLocalConnection(t)
	defer cl.Close()

	twinID := assertCreateTwin(t, cl, AccountBob)

	// Get Alice Stash identity to act as the bonded account
	stashUser := Accounts[AccountAliceStash]
	stashIdentity, err := NewIdentityFromSr25519Phrase(stashUser.Phrase)
	require.NoError(t, err)

	err = cl.BondTwinAccount(stashIdentity, twinID)
	require.NoError(t, err)

	bondedAccount, err := cl.GetTwinBondedAccount(twinID)
	require.NoError(t, err)
	require.NotNil(t, bondedAccount)

	stashAccount, err := FromAddress(stashUser.Address)
	require.NoError(t, err)
	require.Equal(t, stashAccount.PublicKey(), bondedAccount.PublicKey())
}

func TestGetTwinBondedAccount_NotBonded(t *testing.T) {
	cl := startLocalConnection(t)
	defer cl.Close()

	twinID := assertCreateTwin(t, cl, AccountAlice)

	bondedAccount, err := cl.GetTwinBondedAccount(twinID)
	require.NoError(t, err)
	require.Nil(t, bondedAccount)
}
