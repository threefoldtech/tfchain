# Tfchain Substrate Node Development

## Run

Use Rust's native `cargo` command to build and run

```sh
cargo run -- --dev --tmp --offchain-worker Never
```

## Attach a UI

In your web browser, navigate to [https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9945#/explorer](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9945#/explorer) .

## Release Build

The `cargo run` command will perform an initial build. Use the following command to build the node
without launching it:

```sh
cargo build --release
```

## Embedded Docs

Once the project has been built, the following command can be used to explore all parameters and subcommands:

```sh
./target/release/tfchain -h
```

## Update the chain spec

The [chainspecs](./chainspecs) need to be updated if you want to start a new chain.

## Single-Node Development Chain

The provided `cargo run` command will launch a temporary node and its state will be discarded after
you terminate the process. After the project has been built, there are other ways to launch the
node.

Build it in debug mode first:

```sh
cargo build 
```

This command will start the single-node development chain with persistent state:

```sh
./target/debug/tfchain --dev
```

Purge the development chain's state:

```sh
./target/debug/tfchain purge-chain --dev
```

Start the development chain with detailed logging:

```sh
RUST_LOG=debug RUST_BACKTRACE=1 ./target/debug/tfchain -lruntime=debug --dev
```
