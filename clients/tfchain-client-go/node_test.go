package substrate

import (
	"testing"

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
