# Smart Contract Module

A pallet for Threefold "smart contracts". A smart contract is an agreement between a user and a [ZOS](https://github.com/threefoldtech/zos) node that is enforced by the blockchain. The smart contract module is responsible for the creation, billing and deletion of smart contracts.

This module is tightly coupled with the [Tfgrid module](../pallet-tfgrid/readme.md) and the [Tft Price Module](../pallet-tft-price/readme.md).

## Overview

The smart contract module provides functions for:

- Creating / Updating / Canceling a smart contract
- Reporting resource usage for a smart contract
- Creating / Approving solution providers
- Creating / Updating / Canceling a Service Contract
- Setting an extra price for a dedicated ZOS node
- Billing all sorts of contracts based on a pricing policy defined in the [Tfgrid module](../pallet-tfgrid/readme.md) and a TFT price fetched in the [Tft Price Module](../pallet-tft-price/readme.md)..

The billing is triggered by an offchain worker that runs after a block is created. This offchain worker relies on the `aura` key in the keystore to sign the transaction triggers the billing. This means that only valid block creators can sign this transaction. Given this configuration this pallet will only work in an Aura / Grandpa based chains.

For a more in depth view of this module, see [spec](./spec.md).

## Terminology

- [ZOS](https://github.com/threefoldtech/zos): a Zero-OS node that is running on a physical machine.
- Smart Contract: an agreement between a user and a ZOS node that is enforced by the blockchain. This can be any of:
    - `NodeContract`: An agreement between a user and a ZOS node for the usage of resources on that node.
    - `NameContract`: An agreement between a user and a ZOS node for the usage of a dns name on that node using a gateway.
    - `RentContract`: An agreement between a user and a ZOS node for the usage of the entire node.
    - `ServiceContract`: An agreement between a user and an external service provider for the usage of a service.
- Solution provider: a provider of a solution, see [solution provider](./solution_provider.md)

## Interface

Dispatchable functions of this pallet.

- `create_node_contract` - Create a node contract
- `update_node_contract` - Update a node contract
- `create_name_contract` - Create a name contract
- `create_rent_contract` - Create a rent contract
- `cancel_contract`: Cancel a contract (any of the smart contract type)
- `add_nru_reports`: Reports network resource usage from ZOS to the chain
- `report_contract_resources`: Reports a `NodeContract` used resources (nru, cru, mru, sru, ipu) to the chain.
- `create_solution_provider`: Create a solution provider
- `approve_solution_provider`: Approve a solution provider, the origin for this call is a configurable origin.
- `bill_contract_for_block`: Triggers the billing of a contract on this block.
- `service_contract_create`: Create a service contract
- `service_contract_set_metadata`: Set metadata for a service contract
- `service_contract_set_fees`: Set fees for a service contract
- `service_contract_approve`: Approve a service contract
- `service_contract_reject`: Reject a service contract
- `service_contract_cancel`: Cancel a service contract
- `service_contract_bill`: Bill a service contract
- `change_billing_frequency`: Change the billing frequency of all contracts, the origin for this call is a configurable origin.
- `attach_solution_provider_id`: Attach a solution provider id to a contract
- `set_dedicated_node_extra_fee`: Set an extra fee for a dedicated node