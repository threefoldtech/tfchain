# Smart Contract for IT on the blockchain

## Architecture

Two main components will play a role in achieving a decentralised consensus between a user and a node.

1: TFGrid Substrate Database [Pallet TFGrid](../pallet-tfgrid/readme.md)

2: TFGrid Smart Contract

The TFGrid Substrate Database will keep a record of all users, twins, nodes and farmers in the TF Grid network. This makes it easy to integrate the Smart Contract on Substrate as well since we can read from that storage in Runtime.

check flow diagram: [flow](./flow.png)

## Concepts

### Node Contract

A `NodeContact` represents a workload that the user wants to deploy on a node. The `deployment_hash` and `deployment_data` holds the workload definition. A user needs to create a workload definition using one our tools ([Terraform](https://github.com/threefoldtech/terraform-provider-grid), [Client](https://github.com/threefoldtech/grid3_client_ts), ..) and sign it. It's important that the user checks up front if the node can host his deployment (also done with client tools), if the node can accept his deployment a contract can be created and this will be picked up by the node once he also sends his deployment data to the node using [RMB](https://github.com/threefoldtech/rmb-rs). A deployment can also have a Public IP (ipv4) which is also configurable on contract create.

When the contract is deployed on the Node, the Node will report the used resources by that contract. From that moment on, billing for that contract is enabled.

### Rent Contract

A contract between a user and node for renting an entire node. A user can only select nodes that are `dedicated`. A farmer can mark his nodes as dedicated by calling `set_node_dedicated` on this module. When a node is not dedicated and has active contracts an error will be returned. If the node is marked as dedicated and a farmer wants to mark at is non-dedicated, the same call will decomission all active node/rent contracts on that node.

When a user creates a `RentContract` he pays for the entire capacity on that node. Any subsequent `NodeContract` he deploys on that node is free of charge (because he is already paying for the entire capacity).

### Name Contract

Is a contract to reserve a unique name that can be used on the Threefold Web Gateways. A name is unique and is bound to the creator.

### Billing

Any contract type (NodeContract, RentContract, NameContract) when created is inserted in a billing loop. The frequency of which this contract gets billed is configurable by a trait `BillingFrequency` which stands for a number of blocks. When this amount of blocks pass the contract cost is calculated as following:

- Amount of `resources in use * price for those resources * time passed`
- Amount `NRU (network) used * price for network units * time passed`

Name contracts cost calculation is a static price \*

The pricing for contracts is read from `PricingPolicy` defined in `pallet-tfgrid` and the cost is calculated in USD. The amount due is then calculated in TFT based on the current price of TFT and the amount due in USD (see [Pallet TFT Price](../pallet-tft-price/readme.md)). This amount is then locked on the user's account, the lock identifier being the contract ID for which the contract cost is calculated.

Another configuration trait `DistributionFrequency` allows to set a frequency in block numbers where the contract rewards are distributed. This is done to not overload the system, say you put the `BillingFrequency` to 1 hour, you can put the `DistributionFrequency` to 24 hours.

Each contract where the rewards are distributed, are distributed to the following parties defined in `PricingPolicy` (see pallet-tfgrid).

- 5% To the validator staking pool (Rewards farmers that run TFChain 3.0 validator nodes in a later stage)
- 10% to the Threefold foundation (Funds allocated to promote and grow the ThreeFold Grid)
- 35% Burned (A mechanism used to maintain scarcity in the TFT economy)
- 50% Solution providers & sales channel (Managed by the DAO)

When a contract is canceled before the `DistributionFrequency` or `BillingFrequency` the elapsed time is calculated and the user get's billed for that amount and the rewards are distributed.

### Discounts

A user will get discounts on Contracts when he holds X amount of tokens in his wallet. Discounts are defined here: https://library.threefold.me/info/threefold/#/cloud/threefold__pricing?id=discount-levels.

### Grace period for contracts

There is a configuration trait `GracePeriod` that enabled grace period for contracts that ran out of funds.

Grace period is triggered if the amount due for a billing cycle is larger than the user's balance.
A grace period is removed on a contract if the next billing cycles notices that the user reloaded the balance on his account.
If this happens, the contract is set back to created state. If a user ignores a graced-out contract, the contract is deleted after the time defined by `GracePeriod` configuration trait.

## Footnote

Sending the workloads encrypted to the chain makes sure that nobody except the destination Node can read the deployment's information as this can contain sensitive data. This way we also don't need to convert all the Zero OS primitive types to a Rust implementation and we can keep it relatively simple.
