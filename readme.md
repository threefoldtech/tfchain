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
make build-debug
```

This will build the node binary in release mode, once built you can execute it by doing following:

`./target/release/tfchain --dev --tmp --ws-external`

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

### Graphql (optional)

If you want to query the data inside the blockchain with graphql you can set this up locally.

```sh
cd graphql

yarn

yarn build

yarn db:up
yarn db:prepare
yarn db:migrate
yarn db:init

docker-compose up
```

Now browse to localhost:4000/graphql

Example query:

```graphql
query {
    entities(limit: 5) {
        name
    }
}
```
