# Tfchain Substrate Node Development

## Run

Use Rust's native `cargo` command to build

```sh
cargo run --release -- --dev --tmp
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

Once the project has been built, the following command can be used to explore all parameters and
subcommands:

```sh
./target/release/tfchain -h
```

## Update the chain spec

When the [tfchain_pallets](https://github.com/threefoldtech/tfchain_pallets) have been updated, the chainspec needs to be updated if you want to start a new chain.

## Multi-Node Development Chain

The provided `cargo run` command will launch a temporary node and its state will be discarded after
you terminate the process. After the project has been built, there are other ways to launch the
node.

To run the NPoS network you need 2 validators running in order to finalize blocks. You can use Alice & Bob for ease of use.

### Start a Node with Alice

```sh
./target/release/tfchain \
  --base-path /tmp/alice \
  --chain local \
  --alice \
  --port 30333 \
  --ws-port 9945 \
  --rpc-port 9933 \
  --node-key 0000000000000000000000000000000000000000000000000000000000000001 \
  --validator
```

### Start a Node with Bob

```sh
./target/release/tfchain \
  --base-path /tmp/bob \
  --chain local \
  --bob \
  --port 30334 \
  --ws-port 9946 \
  --rpc-port 9934 \
  --validator \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp
```

There will show errors from the tft_price offchain_worker:

```log
2021-11-25 13:00:42  No local account available
2021-11-25 13:00:42  err: OffchainSignedTxError
```

These can be ignored.

Purge the development chain's state:

```sh
./target/release/tfchain purge-chain --dev
```

Start the development chain with detailed logging:

```sh
RUST_LOG=debug RUST_BACKTRACE=1 ./target/release/tfchain -lruntime=debug --dev
```
