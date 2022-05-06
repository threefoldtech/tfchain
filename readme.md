# TFchain

## Installation

### Node

You will need a specific version of rust nightly in order to compile:

`rustup install nightly`

Wasm toolchain:

`rustup target add wasm32-unknown-unknown --toolchain nightly`

Now you can build.

```sh
cd substrate-node
# make sure you run nightly
rustup override set nightly
cargo build
```

This will build the node binary in debug mode, once built you can execute it by doing following:

`./target/debug/tfchain --dev --ws-external`

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
