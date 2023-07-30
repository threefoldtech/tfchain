# TFchain &middot; ![Build & Tests](https://github.com/threefoldtech/tfchain/actions/workflows/010_build_and_test.yaml/badge.svg)

<p align="center">
  <img height="25%" width="25%" src="./substrate-node/.maintain/header.png">
</p>
The ultimate solution for a more fair, sustainable, and accessible Internet for everyone.


## Overview
TFchain is the official implementation of the ThreeFold blockchain, which is the ledger technology behind the ThreeFold Grid.

TFchain serves as a digitally distributed, decentralized, public ledger that tracks the storage and computing resources of [ZOS](https://github.com/threefoldtech/zos) nodes running on The ThreeFold Grid, manages the smart contracts that enable users to rent and use these resources and provides the incentive layer which ensures that the nodes are rewarded for their contributions to the network.

The ThreeFold Grid is a decentralized network of nodes that provide cloud resources without intermediaries. The nodes are operated by independent farmers who are incentivized by the ThreeFold Token (TFT). TFT is a utility token that is used to exchange resources on the network and to build and run digital workloads, also is used for paying transaction fees, staking, governance, and smart contract execution.

TFchain is based on [Substrate](https://substrate.io), a framework for building blockchains in Rust. it has several custom pallets that implement its features.

TFchain also supports cross-chain transfers with other blockchains, such as Bitcoin, Binance Smart Chain and Stellar.

## Code structure
The project consists of several components, each with a specific purpose and functionality.

* [activation-service](./activation-service): Contains all the GitHub workflows for this repository, such as testing, linting, and creating a release.
* [bridge](./bridge): Contains a program that runs in the background scanning a Stellar vault address for deposits and TFchain for withdrawals and executes upon them. This allows cross-chain transfers between Stellar and TFchain.
* [clients](./clients): Contains various clients that provide all the functionality for interacting with TFchain.
  * [tfchain-client-js](./clients/tfchain-client-js): JS TFchain client
  * [tfchain-client-rs](./clients/tfchain-client-rs): Rust TFchain client (Unfinished)
  * [tfchain-client-go](./clients/tfchain-client-go): Go TFchain client
* [docs](./docs): Contains all the development and production related documentation for TFchain, such as how to run a node, how to create an account, and how to upgrade the runtime.
  * [architecture](./docs/architecture): Architecture Decision Records
  * [development](./docs/development): Development docs, such as how to run a node, how to create an account.
  * [misc](./docs/misc): Other docs
  * [production](./docs/production): Production related docs, such as release and runtime upgrade flows.
* [research](./research): Contains some research around proof of stake, which is the consensus mechanism used by TFchain. Proof of stake is a way of securing the network by requiring validators to stake their tokens and participate in the consensus process.
  * [pos](./research/pos)
* [tools](./tools): Contains tools for developing and testing TFchain.
  * [fork-off-substrate](./tools/fork-off-substrate): A tool that can fork a live network to be used in development for testing out runtime migrations or upgrades. Runtime migrations or upgrades are changes in the runtime logic that require updating all nodes on the network.
* [scripts](./tools/fork-off-substrate/scripts): Contains JavaScript scripts for various purposes, such as subscribing to events.
* [substrate-node](./substrate-node): Contains the actual node code that runs the blockchain. It is based on a reference implementation of Substrate called `substrate-node-templat`, which is kept up to date with the latest Substrate release. The node code defines how to initialize and run a Substrate node with the TFchain runtime and pallets.
  * [chainspecs](./substrate-node/chainspecs): Contains initial TFchain specifications for all live networks, such as testnet and mainnet. Chain specifications are JSON files that define the genesis state and configuration of each network. They must be used in order to spin up a node from scratch.
    * [dev](./substrate-node/chainspecs/dev): Dev network chain specification
    * [main](./substrate-node/chainspecs/main): Main network chain specification
    * [qanet](./substrate-node/chainspecs/qanet): Quality assurance (QA) network chain specification
    * [test](./substrate-node/chainspecs/test): test network chain specification
  * [charts](./substrate-node/charts): Contains Kubernetes charts for deploying and managing TFchain nodes on cloud platforms. Kubernetes is a system for automating deployment, scaling, and management of containerized applications.
    * [substrate-node](./substrate-node/charts/substrate-node)
  * [node](./substrate-node/node): Contains the application that allows users to participate in a blockchain network. 
  * [support](./substrate-node/support): Contains a shared types and configuration crate that is used across the runtime, pallets, and node. It defines common types, traits and constants that are used by different components of TFchain, such as `Farm`. It also declares the Zos `Node` type, which is a special type of node that provides zero-os services on Tfgrid.
  * [runtime](./substrate-node/runtime): Contains the runtime crate that specifies what pallets are used by TFchain and their configuration. A crate is a compilation unit in Rust that can produce an executable or a library. The runtime crate is used by the node crate to run the blockchain with the desired logic and features.
  * [tests](./substrate-node/tests): Contains integration tests crate for TFchain. Integration tests are tests that run the entire system or some of its parts together to check if they work as expected. See the [docs](./substrate-node/tests/readme.md) on how to develop and run these tests.
  * [target](./substrate-node/target): Contains the compiled binary files of the Substrate node
    * [cxxbridge](./substrate-node/target/cxxbridge)
    * [debug](./substrate-node/target/debug) 
    * [doc](./substrate-node/target/doc): Compiled rust documentation for all crates. 
  * [pallets](./substrate-node/pallets): Contains the actual runtime implementation of TFchain. Runtime is the core logic of the blockchain that defines how it works and what it can do. Runtime is composed of modules called pallets, which provide specific functionality and can be plugged into a runtime. TFchain has several custom pallets that implement its features, such as Tfgrid, TFT Price, Dao, Smart Contract modules and bridge. Check modules documentations for what each module is about.
  * [Cargo.toml](./substrate-node/Cargo.toml): Contains crate dependencies for the Substrate node project. These dependencies are shared across all sub-crates and specify what external libraries are used by TFchain. If you need to add dependencies for a specific runtime pallet use `Cargo.toml` file for that pallet instead.
* [scripts](./scripts): Contains JavaScript scripts for various purposes, such as subscribing to events.

## Runtime Architecture
The TFchain Runtime is built using FRAME and consists of pallets from substrate and `pallet/`

### From substrate:
- [System](https://paritytech.github.io/substrate/master/pallet_grandpa/index.html): provide core functionality that all other pallets depend on.
- [Utility](https://paritytech.github.io/substrate/master/pallet_utility/index.html): Allows users to use derivative accounts, and batch calls.
- [Balances](https://paritytech.github.io/substrate/master/pallet_balances/index.html): Tracks TFT token balances.
- [Timestamp](https://paritytech.github.io/substrate/master/pallet_timestamp/index.html): On-Chain notion of time.
- [Transaction Payment](https://paritytech.github.io/substrate/master/pallet_transaction_payment/index.html): Provides the basic logic to compute pre-dispatch transaction fees.
- [Scheduler](https://paritytech.github.io/substrate/master/pallet_scheduler/index.html): Exposes capabilities for scheduling dispatches to occur at a specified block number or at a specified period. 
- [Session](https://paritytech.github.io/substrate/master/pallet_scheduler/index.html): Allows validators to manage their session keys, provides a function for changing the session length, and handles session rotation.
- [Aura](https://paritytech.github.io/substrate/master/pallet_aura/index.html): Extends Aura consensus by managing offline reporting.
- [Grandpa](https://paritytech.github.io/substrate/master/pallet_grandpa/index.html): Manage the GRANDPA finality authority set.
- [Authorship](https://paritytech.github.io/substrate/master/pallet_authorship/index.html): Tracks the current author of the block and recent uncles.
- [Collective](https://paritytech.github.io/substrate/master/pallet_collective/index.html): Allows Members of a set of account IDs to make their collective feelings known through dispatched calls from one of two origins.
- [Membership](https://paritytech.github.io/substrate/master/pallet_membership/index.html): Allows control of membership of a set of Account IDs.

### The following pallets are stored in pallets/. They are designed for TFchain's specific requirements:

- [pallet-dao](./substrate-node/pallets/pallet-dao): Enables the TFchain network to be governed by its stakeholders. See [dao](./docs/misc/minimal_DAO.md) for more info.
- [pallet-kvstore](./substrate-node/pallets/pallet-kvstore): Allow TFchain network participants to store and retrieve key-value pairs of data.
- [pallet-runtime-upgrade](./substrate-node/pallets/pallet-runtime-upgrade): Wrapper for frame_system set_code extrinsic to execute it with a configurable origin. 
- [pallet-smart-contract](./substrate-node/pallets/pallet-smart-contract): Enables the creation, billing and deletion of smart contracts.
- [pallet-tfgrid](./substrate-node/pallets/pallet-tfgrid): Registry for Nodes / Farms / Twins
- [pallet-tft-bridge](./substrate-node/pallets/pallet-tft-bridge): Support bridging requirement between TFchain TFT and Stellar TFT
- [pallet-tft-price](./substrate-node/pallets/pallet-tft-price): TFT price oracle. See [price](./docs/misc/price.md) for more info. 
- [substrate-validator-set](./substrate-node/pallets/substrate-validator-set): For adding / removing authorities. fork of substrate validator set pallet.
- [pallet-validator](./substrate-node/pallets/pallet-validator): (Kinda Deprecated).
- [pallet-burning](./substrate-node/pallets/pallet-burning): Allow anyone to burn his TFT (Kinda Deprecated).

## Bridge

See [bridge](./bridge/README.md) for more information on the bridge between TFchain TFT and Stellar TFT.

## Scripts

See [scripts](./scripts/README.md) for more information on how to use the scripts.
## Deployed instances

- Development network:

  - Polkadot UI: [https://polkadot.js.org/apps/?rpc=wss%3A%2F%2F/tfchain.dev.grid.tf#/explorer](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2F/tfchain.dev.grid.tf#/explorer)
  - Websocket url: wss://tfchain.dev.grid.tf

- Qa testing network:

  - Polkadot UI: [https://polkadot.js.org/apps/?rpc=wss%3A%2F%2F/tfchain.qa.grid.tf#/explorer](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2F/tfchain.qa.grid.tf#/explorer)
  - Websocket url: wss://tfchain.qa.grid.tf

- Test network

  - Polkadot UI: [https://polkadot.js.org/apps/?rpc=wss%3A%2F%2F/tfchain.test.grid.tf#/explorer](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2F/tfchain.test.grid.tf#/explorer)
  - Websocket url: wss://tfchain.test.grid.tf

- Production network

  - Polkadot UI: [https://polkadot.js.org/apps/?rpc=wss%3A%2F%2F/tfchain.grid.tf#/explorer](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2F/tfchain.grid.tf#/explorer)
  - Websocket url: wss://tfchain.grid.tf

## Commit messages

In this repository [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/) are used.

## Releases

See [releases](./docs/production/releases.md) for more information on how to create or validate a release.
