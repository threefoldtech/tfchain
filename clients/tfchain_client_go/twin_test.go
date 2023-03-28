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
