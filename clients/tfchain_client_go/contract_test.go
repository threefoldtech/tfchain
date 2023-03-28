package substrate

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestNameContract(t *testing.T) {
	var contractID uint64
	var nameContractID uint64

	cl := startLocalConnection(t)
	defer cl.Close()

	identity, err := NewIdentityFromSr25519Phrase(BobMnemonics)
	require.NoError(t, err)

	assertCreateFarm(t, cl)

	contractID, err = cl.CreateNameContract(identity, testName)
	require.NoError(t, err)

	nameContractID, err = cl.GetContractIDByNameRegistration(testName)
	require.NoError(t, err)
	require.Equal(t, contractID, nameContractID)

	err = cl.CancelContract(identity, contractID)
	require.NoError(t, err)

}

func TestNodeContract(t *testing.T) {
	var nodeID uint32
	var contractID uint64
	var contractIDWithHash uint64
	var contract *Contract

	cl := startLocalConnection(t)
	defer cl.Close()

	identity, err := NewIdentityFromSr25519Phrase(BobMnemonics)
	require.NoError(t, err)

	farmID, twinID := assertCreateFarm(t, cl)

	nodeID = assertCreateNode(t, cl, farmID, twinID, identity)

	require.NoError(t, err)

	contractID, err = cl.CreateNodeContract(identity, nodeID, "", "", 0, nil)
	require.NoError(t, err)

	contract, err = cl.GetContract(contractID)
	require.NoError(t, err)

	contractIDWithHash, err = cl.GetContractWithHash(uint32(
		contract.ContractType.NodeContract.Node),
		contract.ContractType.NodeContract.DeploymentHash)

	require.NoError(t, err)
	require.Equal(t, contractID, contractIDWithHash)

	err = cl.CancelContract(identity, contractID)
	require.NoError(t, err)
}

func TestGetRentContract(t *testing.T) {
	var nodeID uint32
	var contractID uint64

	cl := startLocalConnection(t)
	defer cl.Close()

	identity, err := NewIdentityFromSr25519Phrase(BobMnemonics)
	require.NoError(t, err)

	farmID, twinID := assertCreateFarm(t, cl)

	nodeID = assertCreateNode(t, cl, farmID, twinID, identity)

	// if node had a previous contact from another test, make sure to cancel it
	if prev, err := cl.GetNodeRentContract(nodeID); err == nil {
		err = cl.CancelContract(identity, prev)
		require.NoError(t, err)
	}

	contractID, err = cl.CreateRentContract(identity, nodeID, nil)
	require.NoError(t, err)

	rentContract, err := cl.GetNodeRentContract(nodeID)
	require.NoError(t, err)
	require.Equal(t, contractID, rentContract)

	_, err = cl.GetContract(contractID)
	require.NoError(t, err)

	err = cl.CancelContract(identity, contractID)
	require.NoError(t, err)
}
