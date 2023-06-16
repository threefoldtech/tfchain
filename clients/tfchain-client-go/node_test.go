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

func TestSetDedicatedNodePrice(t *testing.T) {
	var nodeID uint32

	cl := startLocalConnection(t)
	defer cl.Close()

	identity, err := NewIdentityFromSr25519Phrase(BobMnemonics)
	require.NoError(t, err)

	farmID, twinID := assertCreateFarm(t, cl)

	nodeID = assertCreateNode(t, cl, farmID, twinID, identity)

	price := 100000000
	_, err = cl.SetDedicatedNodePrice(identity, nodeID, uint64(price))
	require.NoError(t, err)

	priceSet, err := cl.GetDedicatedNodePrice(nodeID)
	require.NoError(t, err)

	require.Equal(t, uint64(price), priceSet)
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

func TestNodeUpdateGpuStatus(t *testing.T) {
	cl := startLocalConnection(t)
	defer cl.Close()

	identity, err := NewIdentityFromSr25519Phrase(BobMnemonics)
	require.NoError(t, err)

	farmID, twinID := assertCreateFarm(t, cl)

	nodeID := assertCreateNode(t, cl, farmID, twinID, identity)

	// toggle true
	_, err = cl.SetNodeGpuStatus(identity, true)
	require.NoError(t, err)

	status, err := cl.GetNodeGpuStatus(nodeID)
	require.NoError(t, err)
	require.Equal(t, true, status)

	// toggle false
	_, err = cl.SetNodeGpuStatus(identity, false)
	require.NoError(t, err)

	status, err = cl.GetNodeGpuStatus(nodeID)
	require.NoError(t, err)
	require.Equal(t, false, status)
}
