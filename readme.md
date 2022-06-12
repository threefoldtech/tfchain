# TFchain

## Installation

### Node

Ensure you have the following installed first (they can be installed using apt-get install): 
- librocksdb-dev
- libclang-dev
- clang lldb lld
- build-essential 

You will also need  rust and nightly installed. 
To install Rust:
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
To install nightly:
```
rustup install nightly-2021-06-09
```

Now you can build.

```
cd substrate-node

rustup target add wasm32-unknown-unknown --toolchain nightly-2021-06-09

cargo +nightly-2021-06-09 build --release
```

This will build the node binary in release mode, once built you can execute it by doing following:

```
./target/release/tfchain --ws-external --rpc-methods Unsafe --dev
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
rm -rf /home/<username>/.local/share/tfchain/chains/
```
