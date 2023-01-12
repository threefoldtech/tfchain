# TFchain &middot; ![Build and Test](https://github.com/github/docs/actions/workflows/main.yml/badge.svg)

<p align="center">
  <img src="/substrate-node/.maintain/media/kilt.png">
</p>

Threefold blockchain serves as a registry for Nodes, Farm, Digital Twins and Deployment contracts.
It is the backbone of [ZOS](https://github.com/threefoldtech/zos) and other components.

## Docs

see [docs](./docs/readme.md) for more information on how to work with this component.

## Installation

#### Prerequisites:

Ensure you have the following installed first:

- librocksdb-dev
- libclang-dev
- clang lldb lld
- build-essential
- protoc

You will also need rust and nightly installed.

To install Rust:

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

To install nightly:

```
rustup install nightly-2022-05-11
```

### Node

Navigate to substrate node:

```
cd substrate-node
```

Add Wasm toolchain:

```
rustup target add wasm32-unknown-unknown --toolchain nightly-2022-05-11
```

Now you can build:

```
cargo +nightly-2022-05-11 build --release
```

You can also override the default toolchain

```
rustup override set nightly-2022-05-11
```

Now you can build as following:

```
cargo build
```

This will build the node binary in release mode, once built you can execute it by doing following:

```
./target/release/tfchain --ws-external --dev
```

> You need the `ws-external` flag in order to connect from a zos node to substrate in a local setup.

## Polkadot js

Polkadot js is a webbased substrate client you can use to connect to tfchain as well.

- Development network

  [https://polkadot.js.org/apps/?rpc=wss%3A%2F%2F/tfchain.dev.grid.tf#/explorer](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2F/tfchain.dev.grid.tf#/explorer)

- Test network

  [https://polkadot.js.org/apps/?rpc=wss%3A%2F%2F/tfchain.test.grid.tf#/explorer](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2F/tfchain.test.grid.tf#/explorer)

- Production network

  [https://polkadot.js.org/apps/?rpc=wss%3A%2F%2F/tfchain.grid.tf#/explorer](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2F/tfchain.grid.tf#/explorer)

### Upgrading runtime

See [process](./substrate-node/upgrade_process.md)

### Client

You can use the client to interact with the chain, [read more](./cli-tool/readme.md)

### Data Cleanup:

To wipe data run:

```
./target/release/tfchain purge-chain --dev
```
