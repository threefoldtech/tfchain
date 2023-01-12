# Development

We will describe everything related to getting started with development on tfchain here.

## Setup work environment

See [setup](./setup.md)

## Get the code

```
git clone git@github.com:threefoldtech/tfchain.git
```

## Installing and running tfchain in dev mode

```
cd tfchain/substrate-node
cargo build

./target/debug/tfchain --dev --ws-external --pruning archive
```

## Code changes?

Wipe data and recompile.

### Data Cleanup:

To wipe data run:

```
./target/debug/tfchain purge-chain --dev
```
