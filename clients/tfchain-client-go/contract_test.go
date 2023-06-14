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

func TestCancelBatch(t *testing.T) {
	var nodeID uint32

	cl := startLocalConnection(t)
	defer cl.Close()

	identity, err := NewIdentityFromSr25519Phrase(BobMnemonics)
	require.NoError(t, err)

	farmID, twinID := assertCreateFarm(t, cl)

	nodeID = assertCreateNode(t, cl, farmID, twinID, identity)

	require.NoError(t, err)

	contractID_1, err := cl.CreateNodeContract(identity, nodeID, "", "adf503a0e016b0a74804c07494f87252", 0, nil)
	require.NoError(t, err)
	_, err = cl.GetContract(contractID_1)
	require.NoError(t, err)

	contractID_2, err := cl.CreateNodeContract(identity, nodeID, "", "adf503a0e016b0a74804c07494f87251", 0, nil)
	require.NoError(t, err)
	_, err = cl.GetContract(contractID_2)
	require.NoError(t, err)

	err = cl.BatchCancelContract(identity, []uint64{contractID_1, contractID_2})
	require.NoError(t, err)
}

func TestCreateBatch(t *testing.T) {
	var nodeID uint32

	cl := startLocalConnection(t)
	defer cl.Close()

	identity, err := NewIdentityFromSr25519Phrase(BobMnemonics)
	require.NoError(t, err)

	farmID, twinID := assertCreateFarm(t, cl)

	nodeID = assertCreateNode(t, cl, farmID, twinID, identity)
	require.NoError(t, err)

	t.Run("batch all succeeded", func(t *testing.T) {
		contracts := []BatchCreateContractData{{
			Node:               nodeID,
			Body:               "",
			Hash:               "adf503a0e0160ba74804c27494f87255",
			PublicIPs:          0,
			SolutionProviderID: nil,
		}, {
			Node:               nodeID,
			Body:               "",
			Hash:               "d20454ba65dfd4577af63d142c622cdc",
			PublicIPs:          0,
			SolutionProviderID: nil,
		}, {
			Name: "test",
		}}

		c, err := cl.BatchAllCreateContract(identity, contracts)
		require.NoError(t, err)
		require.Len(t, c, 3)

		_, err = cl.GetContract(c[0])
		require.NoError(t, err)

		_, err = cl.GetContract(c[1])
		require.NoError(t, err)

		_, err = cl.GetContract(c[2])
		require.NoError(t, err)

		err = cl.BatchCancelContract(identity, c)
		require.NoError(t, err)
	})
	t.Run("batch all failed", func(t *testing.T) {
		// second contract already exists (same hash)
		contracts := []BatchCreateContractData{{
			Node:               nodeID,
			Body:               "",
			Hash:               "adf503a0e0160ba74804c27494f87250",
			PublicIPs:          0,
			SolutionProviderID: nil,
		}, {
			Node:               nodeID,
			Body:               "",
			Hash:               "adf503a0e0160ba74804c27494f87250",
			PublicIPs:          0,
			SolutionProviderID: nil,
		}, {
			Name: "test",
		}}

		c, err := cl.BatchAllCreateContract(identity, contracts)
		require.Error(t, err)
		require.Len(t, c, 0)

		hash := NewHexHash(contracts[0].Hash)

		// first contract should be rolled back
		_, err = cl.GetContractWithHash(nodeID, hash)
		require.Error(t, err)
	})
	t.Run("batch succeeded", func(t *testing.T) {
		contracts := []BatchCreateContractData{{
			Node:               nodeID,
			Body:               "",
			Hash:               "adf503a0e0160ba74804c27494f87251",
			PublicIPs:          0,
			SolutionProviderID: nil,
		}, {
			Node:               nodeID,
			Body:               "",
			Hash:               "d20454ba65dfd4577af63d142c622cda",
			PublicIPs:          0,
			SolutionProviderID: nil,
		}, {
			Name: "test",
		}}

		c, index, err := cl.BatchCreateContract(identity, contracts)
		require.NoError(t, err)
		require.Len(t, c, 3)
		require.Nil(t, index)

		_, err = cl.GetContract(c[0])
		require.NoError(t, err)

		_, err = cl.GetContract(c[1])
		require.NoError(t, err)

		_, err = cl.GetContract(c[2])
		require.NoError(t, err)

		err = cl.BatchCancelContract(identity, c)
		require.NoError(t, err)
	})

	t.Run("batch failed", func(t *testing.T) {
		// second contract already exists (same hash)
		contracts := []BatchCreateContractData{{
			Node:               nodeID,
			Body:               "",
			Hash:               "adf503a0e0160ba74804c27494f87253",
			PublicIPs:          0,
			SolutionProviderID: nil,
		}, {
			Node:               nodeID,
			Body:               "",
			Hash:               "adf503a0e0160ba74804c27494f87253",
			PublicIPs:          0,
			SolutionProviderID: nil,
		}, {
			Name: "test",
		}}

		c, index, err := cl.BatchCreateContract(identity, contracts)
		require.Error(t, err)
		require.NotNil(t, index)
		require.Equal(t, *index, 1)
		require.Len(t, c, 1)

		// first contract should be created
		_, err = cl.GetContract(c[0])
		require.NoError(t, err)

		// third contract should not be created
		_, err = cl.GetContractIDByNameRegistration(contracts[2].Name)
		require.Error(t, err)

		err = cl.BatchCancelContract(identity, c)
		require.NoError(t, err)
	})

}
