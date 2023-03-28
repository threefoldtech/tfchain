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
