package substrate

import (
	"testing"
	"time"

	"github.com/stretchr/testify/require"
)

func TestNode(t *testing.T) {
	var nodeID uint32
	var node *Node

	cl := startLocalConnection(t)
	defer cl.Close()

	identity, err := NewIdentityFromSr25519Phrase(BobMnemonics)
	require.NoError(t, err)

	farmID, twinID := assertCreateFarm(t, cl)

	nodeID = assertCreateNode(t, cl, farmID, twinID, identity)

	node, err = cl.GetNode(nodeID)
	require.NoError(t, err)
	require.Equal(t, twinID, uint32(node.TwinID))
	require.Equal(t, farmID, uint32(node.FarmID))

	nodeID, err = cl.GetNodeByTwinID(uint32(node.TwinID))
	require.NoError(t, err)
	require.Equal(t, uint32(node.ID), nodeID)

}

func TestGetNodes(t *testing.T) {

	cl := startLocalConnection(t)
	defer cl.Close()

	farmID, _ := assertCreateFarm(t, cl)

	nodes, err := cl.GetNodes(farmID)
	require.NoError(t, err)
	require.Greater(t, len(nodes), 0)
}

func TestUptimeReport(t *testing.T) {
	cl := startLocalConnection(t)
	defer cl.Close()

	identity, err := NewIdentityFromSr25519Phrase(BobMnemonics)
	require.NoError(t, err)

	farmID, twinID := assertCreateFarm(t, cl)

	_ = assertCreateNode(t, cl, farmID, twinID, identity)

	_, err = cl.UpdateNodeUptime(identity, 100)
	require.NoError(t, err)
}

func TestUptimeReportV2(t *testing.T) {
	cl := startLocalConnection(t)
	defer cl.Close()

	identity, err := NewIdentityFromSr25519Phrase(BobMnemonics)
	require.NoError(t, err)

	farmID, twinID := assertCreateFarm(t, cl)

	_ = assertCreateNode(t, cl, farmID, twinID, identity)

	_, err = cl.UpdateNodeUptimeV2(identity, 100, uint64(time.Now().Unix()))
	require.NoError(t, err)
}
