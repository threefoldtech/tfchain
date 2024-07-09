package substrate

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestAddress(t *testing.T) {
	require := require.New(t)

	account, err := FromAddress(AliceAddress)
	require.NoError(err)

	require.Equal(AliceAddress, account.String())
}

func TestGetAccountByAddress(t *testing.T) {
	cl := startLocalConnection(t)
	defer cl.Close()

	require := require.New(t)

	account, err := FromAddress(AliceAddress)
	require.NoError(err)

	_, err = cl.GetAccountPublicInfo(account)
	require.NoError(err)
}

func TestGetBalance(t *testing.T) {
	cl := startLocalConnection(t)
	defer cl.Close()

	require := require.New(t)

	account, err := FromAddress(AliceAddress)
	require.NoError(err)

	_, err = cl.GetBalance(account)
	require.NoError(err)
}

func TestEnsureAccount_WithDownActivationService(t *testing.T) {
	cl := startLocalConnection(t)
	defer cl.Close()

	require := require.New(t)
	id, err := NewIdentityFromEd25519Phrase("paper endless radio rude wage hood cabin praise girl income chief craft")
	require.NoError(err)
	_, err = cl.EnsureAccount(id, "https://test.wrong-activation-service-url.tf", "test", "test")
	require.Error(err)
	require.IsType(ActivationServiceError{}, err)
	_, err = cl.EnsureAccount(id, "https://httpstat.us/503", "test", "test")
	require.Error(err)
	require.IsType(ActivationServiceError{}, err)
}
