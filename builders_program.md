# Improvements to TF Chain

Modules:

- [pallet-smart-contract](https://github.com/threefoldtech/tfchain/tree/development/pallets/pallet-smart-contract): smart contract 4 IT module.
- [pallet-tfgrid](https://github.com/threefoldtech/tfchain/tree/development/pallets/pallet-tfgrid): module where are the grid objects are saved (twins, farms, nodes, pricing, ..)
- [pallet-tft-bridge](https://github.com/threefoldtech/tfchain_tft_bridge/tree/main/pallet-tft-bridge): module to bridge TFT from Stellar to Substrate TFchain
- [others that are less crucial..](https://github.com/threefoldtech/tfchain/tree/development/pallets)

## Smart Contract Module: Currently implemented

### NodeContracts

A NodeContract is an aggreement between a user and a node to deploy a certain workload. This workload is defined by the `deployment_hash` field. This is a hashed representation of a workload definition (virtual machine, storage, ..). Once the contract is created, the user can send the workload definition the node. Once the node aggrees to deploy the workload, the amount of used resources are set on in storage. The resources define how much the workload is using on a capacity level (storage, cpu, mem, ..). Every contract that is created is inserted into it's own billing cycle. The frequency of the billing cycle is defined with `BillingFrequency` trait on the pallet. Whenever a cycle is triggered, the amount of used resources is fetched for this contract and the amount due for the workload is calculated against the prices that we (Threefold) have specified. 

### NameContracts

A NameContract is an agreement to use a certain dns name to be used on the Threefold gateways. A NameContract uses the same billing mechanism as a NodeContract. 

Still work in progress to hook up all the necessary components in order to actually get a deployment exposed on the webgateway using this name contract.

### RentContracts

A RentContract is an aggreement between a user a node as well, except that it rent's the full available resources on a node instead of only a part of it (NodeContract). By creating a RentContract a user get's a discount on the capacity reserved on a certain node. A RentContract uses the same billing mechanism as a NodeContract. 

When the user creates a RentContract on a node, all subsequent NodeContracts (deployment contracts) are free of billing. This is because the user is already paying for the full capacity on the node and there is not charged for actually using the resources on a node. The only exception here is network resources used and public ips used on a node contract, these are still billed every billing cycle.

## Smart Contract Module: What needs to be done

### Capacity planning
 
In order to highly improve the usability of our decentralised internet we could do some capacity planning for the user. This means we, based on the the workload the user want's to deploy, we suggest the most optimal location and node. This process can't be done on the chain itself because a when a contract is created the amount of resources the user wants to have is not specified, also the location is omitted. 

A seperate service or client could be created for this specific use case. So that a user can ask the service for a suggestion of node(s) for a specific deployment. 

### Improvements on the code

We are not sure how the currently implemented code will behave when we generate high traffic on the grid. For now it seems to be working as intended but we want to prepare for the future where maybe 10.000 users are deploying workloads on our internet. We implemented the concept of a billing cycle but we don't really know the technical limitation of the `OnFinalize` which is heavily relies on. 

More technical details can be submitted.

## TFGrid Module: Currently implemented:

This one is mostly done, see [spec](https://github.com/threefoldtech/tfchain/blob/development/pallets/pallet-tfgrid/spec.md).

## TFT Bridge Module: Currently implemented:

See [spec](https://github.com/threefoldtech/tfchain_tft_bridge/blob/main/specs/bridge.md) for what is already implemented.

## TFT Bridge Module: What needs to be done:

The bridge is not stress tested at all, a lot of things can go wrong. We also need to rethink how failed deposits and withdraws are handled. 

## New features:

### Scaling

Since blockchains rarely scale we will be deploying a or multiple TFchains per set of Zero-OS nodes. We project to have a maximimum number of 10.000 nodes per TFchain. This "cluster" of nodes will be independendant for other other clusters. To actually make these chains communicate with eachother we are looking to build an overlay layer on top of these clusters, we are currently researching if it could be done with Cosmos. A Cosmos chain could provide the overlay layer for deploying workloads over mulitple TFChains. We are also looking into the possibility to host our own parachain that could connect all these TFChains. 

