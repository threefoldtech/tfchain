# Development

We will describe everything related to getting started with development on tfchain here.

## Setup work environment

See [setup](./setup.md)

## Get the code

```sh
git clone git@github.com:threefoldtech/tfchain.git
```

## Installing and running tfchain in dev mode

```sh
cd tfchain/substrate-node
cargo build

./target/debug/tfchain --dev --ws-external --pruning archive
```

This will run the node in default development mode.

## Running multi node local tfchain network

If you want to run tfchain in a multi node network (more than one node), see [local](./local_multinode.md)

## Code changes?

Wipe data and recompile.

### Data Cleanup:

To wipe data run:

```sh
./target/debug/tfchain purge-chain --dev
```

## Writing tests for pallets

Every pallet should have all functionality tested, you can write unit tests and integration tests for a pallet:

- unit test: check https://docs.substrate.io/reference/how-to-guides/testing/
- integration test: [see](../../substrate-node/tests/readme.md)
