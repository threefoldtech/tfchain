# Development

You can find everything related to getting started with development on TFchain here.

## Learn The Basics

check [substrate learn topics](https://docs.substrate.io/learn/) which cover the fundamental concepts and terminology to start building your blockchain using the Substrate framework.

After you digest the information in these introductory sections, you'll be ready to start tinkering with TFchain development.

## Setup Development Environment

see [how to setup your development environment](./setup_development_environment.md) document.

## Get the TFchain Node Code

```sh
git clone https://github.com/threefoldtech/tfchain.git
```

## Compile TFchain Binary

```sh
cd tfchain/substrate-node
cargo build
```

## Run A Dev Node
A dev node is a single-node network that runs on your local machine. It is useful for testing and debugging purposes. To run a dev node, you need to do the following:

```sh
cd tfchain/substrate-node
cargo build

./target/debug/tfchain --dev --ws-external --pruning archive
```
This will run the node in default development mode. This flag sets `--chain=dev`, `--force-authoring`, `--rpc-cors=all`, `--alice`, and `--tmp` flags, unless explicitly overridden.

## Run Multiple Local Nodes

If you want to run tfchain in a multi node network (more than one node), see [local](./local_multinode.md)

## Basic Operations

You can use the Polkadot JS Apps ui to connect to your dev node. You can access the web interface at https://polkadot.js.org/apps/ and change the settings to use a local node with the address `ws://127.0.0.1:9944`.

This will allow you to interact with your dev node and perform basic operations.
- Use the Polkadot JS Apps to interact with your nodes. You can access the web interface at https://polkadot.js.org/apps/ and change the settings to use a local node or a remote node with the appropriate address.
- Use the `Accounts` tab to manage your accounts and balances. You can create new accounts, import existing accounts, transfer tokens, and view your transaction history.
- Use the `Explorer` tab to view the network status and activity. You can see the latest blocks, events, validators, and peers.
- Use the `Chain State` tab to query the state of the network. You can select a module and a storage item and see its value at any given block.
- Use the `Extrinsics` tab to submit extrinsics to the network. You can select an account, a module, and a function and provide any required parameters.

## create an account

An account is a pair of public and private keys that represents your identity and allows you to perform transactions on the tfchain network.

see [how to an create account](./create_devnet_account.md).

## Code Changes ?

Wipe data and recompile.

### Data Cleanup:

```sh
./target/debug/tfchain purge-chain 
```

Note: You don't need to wipe data when you run a node with `--dev` flag and no explicit `--base-path`. this because `--tmp` option is implied which create a temporary directory to store the configuration on node start that will be deleted at the end of the process.
## Fork-Off-Substrate Tool

To experiment with different features and parameters of TFchain, See [here](./fork-off-substrate.md) how you can use `fork-off-substrate` tool to create a local fork of the TFchain network.

## How to develop a pallet

A pallet is a modular component that defines some logic and functionality for the tfchain network. You can develop your own custom pallets using macros, add them to the runtime, and test their functionality.

To learn about pallet development, you can start by checking these resources:

- Get start by reading The substrate [Build application logic tutorials](https://docs.substrate.io/tutorials/build-application-logic/). these tutorials focus on how you can customize the runtime using pallets, creating a custom pallet using macros, adding it to the runtime, and testing its functionality.

- Check the [the Substrate collectibles workshop](https://docs.substrate.io/tutorials/collectibles-workshop/runtime-and-pallets/). This is an interactive, hands-on, and self-paced workshop that introduces the basic steps for building a blockchain-based application using Substrate. 

- Explore other existing pallets on the [Substrate GitHub repository](https://github.com/paritytech/substrate/tree/master/frame) and learn from their code. You can see how different pallets implement different features and patterns, such as storage, events, errors, hooks, traits, weights, origins, calls, etc.


## Writing Tests For Pallets

Every pallet should have all functionality tested, you can write unit tests and integration tests for a pallet:

- unit test: check https://docs.substrate.io/reference/how-to-guides/testing/
- integration test: [see](../../substrate-node/tests/readme.md)

## Upgrading Substrate Version

see [how to upgrade substrate version](./upgrade_substrate.md).
