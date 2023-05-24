package substrate

import (
	"testing"

	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
	"github.com/stretchr/testify/require"
)

func TestServiceContract(t *testing.T) {
	var baseFee uint64 = 1000
	var variableFee uint64 = 1000
	var variableAmount uint64 = 0
	var metadata string = "some_metadata"
	var billMetadata string = "some_bill_metadata"

	cl := startLocalConnection(t)
	defer cl.Close()

	serviceTwinID := assertCreateTwin(t, cl, AccountBob)
	consumerTwinID := assertCreateTwin(t, cl, AccountAliceStash)

	serviceIdentity, err := NewIdentityFromSr25519Phrase(BobMnemonics)
	require.NoError(t, err)

	consumerIdentity, err := NewIdentityFromSr25519Phrase(AliceStashMnemonics)
	require.NoError(t, err)

	serviceAccount, err := FromAddress(BobAddress)
	require.NoError(t, err)

	consumerAccount, err := FromAddress(AliceStashAddress)
	require.NoError(t, err)

	// 1. Create and set first service contract, then consumer reject it

	serviceContractID, err := cl.ServiceContractCreate(serviceIdentity, serviceAccount, consumerAccount)
	require.NoError(t, err)

	err = cl.ServiceContractSetMetadata(consumerIdentity, serviceContractID, metadata)
	require.NoError(t, err)

	err = cl.ServiceContractSetFees(serviceIdentity, serviceContractID, baseFee, variableFee)
	require.NoError(t, err)

	err = cl.ServiceContractReject(consumerIdentity, serviceContractID)
	require.NoError(t, err)

	// 2. Create and set second service contract, then approve it by both service and consumer

	serviceContractID, err = cl.ServiceContractCreate(serviceIdentity, serviceAccount, consumerAccount)
	require.NoError(t, err)

	err = cl.ServiceContractSetMetadata(consumerIdentity, serviceContractID, metadata)
	require.NoError(t, err)

	err = cl.ServiceContractSetFees(serviceIdentity, serviceContractID, baseFee, variableFee)
	require.NoError(t, err)

	err = cl.ServiceContractApprove(serviceIdentity, serviceContractID)
	require.NoError(t, err)

	err = cl.ServiceContractApprove(consumerIdentity, serviceContractID)
	require.NoError(t, err)

	// 3. Check if service contract is well set

	scID, err := cl.GetServiceContractID()
	require.NoError(t, err)
	require.Equal(t, serviceContractID, scID)

	serviceContract, err := cl.GetServiceContract(serviceContractID)
	require.NoError(t, err)
	require.Equal(t, serviceContract.ServiceTwinID, types.U32(serviceTwinID))
	require.Equal(t, serviceContract.ConsumerTwinID, types.U32(consumerTwinID))
	require.Equal(t, serviceContract.BaseFee, types.U64(baseFee))
	require.Equal(t, serviceContract.VariableFee, types.U64(variableFee))
	require.Equal(t, serviceContract.Metadata, metadata)
	require.Equal(t, serviceContract.AcceptedByService, true)
	require.Equal(t, serviceContract.AcceptedByService, true)
	require.Equal(t, serviceContract.State, ServiceContractState{
		IsCreated:        false,
		IsAgreementReady: false,
		IsApprovedByBoth: true,
	})

	// 4. Bill consumer for service contract
	// should be able to go to future block to test varaible amount greater than 0

	err = cl.ServiceContractBill(serviceIdentity, serviceContractID, variableAmount, billMetadata)
	require.NoError(t, err)

	err = cl.ServiceContractCancel(consumerIdentity, serviceContractID)
	require.NoError(t, err)
}
