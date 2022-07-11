# Smart Contract for IT on the blockchain

## Proposed architecture

Two main components will play a role in achieving a decentralised consensus between a user and a farmer.

1: TFGrid Substrate Database [Pallet TFGrid](../pallet-tfgrid/readme.md)

2: TFGrid Smart Contract

The TFGrid Substrate Database will keep a record of all users, twins, nodes and farmers in the TF Grid network. This makes it easy to integrate the Smart Contract on Substrate as well since we can read from that storage in Runtime.

check flow diagram: [flow](./flow.png)

The Smart Contract on Substrate will work as following:

## 1: The user wants to deploy a workload, he interacts with this smart contract pallet and calls: `create_contract` with the input being:

The user must instruct his twin to create the contract. A contract will always belong to a twin and to a node. This relationship is important because only the user's twin can update the contract as well as only the target node can update as well.

json
```
contract {
    "workload": "encrypted_workload_data",
    "node_address": "some_node_address",
    "public_ips": NumberOfPublicIPS
}
```
The `node_address` field is the target node's ss58 address. A user can do lookup for a node to find it's corresponding address.

The workload data is encrypted by the user and contains the workload definition for the node.

If `public_ips` is specified, the contract will reserve the number of public ips requested on the node's corresponding farm. If there are not enough ips available an error will be returned. If the contract is canceled by either the user or the node, the ips for that contract will be freed.

This pallet saves this data to storage and returns the user a `contract_id`.

## 2: The user sends the contractID through the RMB to the destination Node.

The Node reads from the [RMB](https://github.com/threefoldtech/rmb) and sees a deploy command, it reads the contractID and fetches that Contract from this pallet's storage. It decodes the workload and does validation before it deploys the contents. If successfull it sets the Contract to state `deployed` on the chain. Else the contract is removed.

## 3: The Node sends consumption reports to the chain

The Node periodically sends consumption reports back to the chain for each deployed contract. The chain will compute how much is being used and will bill the user based on the farmers prices (the chain can read these prices by quering the farmers storage and reading the pricing data). See [PricingPolicy](https://github.com/threefoldtech/substrate-pallets/blob/03a5823ce79200709d525ec182036b47a60952ef/pallet-tfgrid/src/types.rs#L120).

A report looks like:

json
```
{
	"contract_id": contractID,
	"cru": cpus,
	"sru": ssdInBytes,
	"hru": hddInBytes,
	"mru": memInBytes,
	"nru": trafficInBytes
}
```

The node can call `add_reports` on this module to submit reports in batches.

Usage of SU, CU and NU will be computed based on the prices and the rules that Threefold set out for cloud pricing.

Billing will be done in Database Tokens and will be send to the corresponding farmer. If the user runs out of funds the chain will set the contract state to `canceled` or it will be removed from storage. The Node needs to act on this contact canceled event and decomission the workload. 

The main currency of this chain. More information on this is explained here: TODO

## Grace period for contracts

Implements a grace period state `GracePeriod(startBlockNumber)` for all contract types
A grace period is a static amount of time defined by the runtime configuration.


Grace period is triggered if the amount due for a billing cycle is larger than the user's balance. 
A grace period is removed on a contract if the next billing cycles notices that the user reloaded the balance on his account. 
If this happens, the contract is set back to created state. If a user ignores a graced-out contract, the contract is deleted after the time defined by Grace Period configuration trait.

## Solution provider

A "solution" is something running on the grid, created by a community member. This can be brought forward to the council, who can vote on it to recognize it as a solution. On contract creation, a recognized solution can be referenced, in which case part of the payment goes toward the address coupled to the solution. On chain a solution looks as follows:

- Description (should be some text, limited in length. Limit should be rather low, if a longer one is desired a link can be inserted. 160 characters should be enough imo).
- Up to 5 payout addresses, each with a payout percentage. This is the percentage of the payout received by the associated address. The amount is deducted from the payout to the treasury and specified as percentage of the total contract cost. As such, the sum of these percentages can never exceed 50%. If this value is not 50%, the remainder is payed to the treasure. Example: 10% payout percentage to addr 1, 5% payout to addr 2. This means 15% goes to the 2 listed addresses combined and 35% goes to the treasury (instead of usual 50). Rest remains as is. If the cost would be 10TFT, 1TFT goes to the address1, 0.5TFT goes to address 2, 3.5TFT goes to the treasury, instead of the default 5TFT to the treasury
- A unique code. This code is used to link a solution to the contract (numeric ID).

This means contracts need to carry an optional solution code. If the code is not specified (default), the 50% goes entirely to the treasury (as is always the case today).

A solution can be created by calling the extrinsic `smartContractModule` -> `createSolutionProvider` with parameters:

- description
- link (to website)
- list of providers

Provider: 

- who (account id)
- take (amount of take this account should get) specified as an integer of max 50. example: 25

A forum post should be created with the details of the created solution provider, the dao can vote to approve this or not. If the solution provider get's approved, it can be referenced on contract creation.

Note that a solution can be deleted. In this case, existing contracts should fall back to the default behavior (i.e. if code not found -> default). 

### Changes to contract creation

When creating a contract, a `solution_provider_id` can be passed. An error will be returned if an invalid or non-approved solution provider id is passed.

## Footnote

Sending the workloads encrypted to the chain makes sure that nobody except the destination Node can read the deployment's information as this can contain sensitive data. This way we also don't need to convert all the Zero OS primitive types to a Rust implementation and we can keep it relatively simple.