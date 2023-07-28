# Smart Contract for IT on the blockchain

## Proposed architecture

Two main components will play a role in achieving a decentralised consensus between a user and a farmer.

1: TFGrid Substrate Database [Pallet TFGrid](../pallet-tfgrid/readme.md)

2: TFGrid Smart Contract

The TFGrid Substrate Database will keep a record of all users, twins, nodes and farmers in the TF Grid network. It will also keep a record of all the contracts that are created between users and farmers. This pallet will also be responsible for the billing of these contracts.

check flow diagram: [flow](./flow.png)

The Smart Contract on Substrate will work as following:

### 1: The user wants to deploy a workload, he interacts with this smart contract pallet and calls: `create_contract` with the input being:

The user must instruct the chain to create the contract. A contract will always belong to a user's twin and to a node. This relationship is important because only the user's twin and the target can update the contract.

json

```
contract {
    "node_id": "some_node_id",
    "deployment_hash": "hashed_deployment_data",
	"deployment_data": "additional deployment data",
    "public_ips": NumberOfPublicIPS,
	"solution_provider_id": "Optional solution provider id",
}
```

The `node_id` field is the target node's numerical ID.

If `public_ips` is specified, the contract will reserve the number of public ips requested on the node's corresponding farm. If there are not enough ips available an error will be returned. If the contract is canceled by either the user or the node, the ips for that contract will be freed.

This pallet saves this data to storage and returns the user a `contract_id`.

### 2: The user sends the contractID through the RMB to the destination Node.

The Node reads from the [RMB](https://github.com/threefoldtech/rmb) and sees a deploy command, it reads the contractID and deployment information and fetches that Contract from this pallet's storage. It decodes the workload and does validation before it deploys the contents. 

### 3: The Node sends reports to the chain

The Node periodically sends consumption reports back to the chain for each deployed contract. The chain will compute how much is being used and will bill the user based on well defined prices (the chain can read these prices by quering the farmers storage and reading the pricing data).

## Billing

The chain will start billing contracts as soon as the contract is created. The billing will be done periodically based on the billing frequency of the contract. The billing frequency is a configuration parameter that can be changed by the council. The billing will read the pricing policies and the TFT price and will calculate the amount of TFT to bill the user. This amount is locked on the user's account and deducted every 24 hours.

## Grace period for contracts

Implements a grace period state `GracePeriod(startBlockNumber)` for all contract types
A grace period is a static amount of time defined by the runtime configuration.

Grace period is triggered if the amount due for a billing cycle is larger than the user's balance.
A grace period is removed on a contract if the next billing cycles notices that the user reloaded the balance on his account.
If this happens, the contract is set back to created state. If a user ignores a graced-out contract, the contract is deleted after the time defined by Grace Period configuration trait.

During a grace period, the deployment related to the contract will be innaccessible.

## Solution provider

See [doc](./solution_provider.md)

## Service contracts

See [doc](./service_consumer_contract_spec.md)

## Footnote

Sending the workloads encrypted to the chain makes sure that nobody except the destination Node can read the deployment's information as this can contain sensitive data. This way we also don't need to convert all the Zero OS primitive types to a Rust implementation and we can keep it relatively simple.
