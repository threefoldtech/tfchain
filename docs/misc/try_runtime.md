# Trying out a runtime migration on a running network

List of available endpoints for our networks:

- Devnet: 10.10.0.40
- QAnet: 10.10.0.42
- Testnet: 10.10.0.55
- Mainnet: 10.10.0.56

(If you are not in Lochristi office you probably need VPN to reach to ips)

## Select a network you wish to try the runtime upgrade upon

Now select the release you want to test with `try-runtime` and follow instructions [here](https://paritytech.github.io/try-runtime-cli/try_runtime/).
Example of commands sequence for running migrations of a given runtime on top of Devnet live state:

```sh
# Install try-runtime latest version (recommended for local development)
cargo install --git https://github.com/paritytech/try-runtime-cli --locked
```
```sh
# Compile substrate node 
cd substrate-node
cargo build --features try-runtime --release
```
```sh
# Run the runtime migrations on top of Devnet live state
try-runtime --runtime ./target/release/wbuild/tfchain-runtime/tfchain_runtime.compact.compressed.wasm on-runtime-upgrade live --uri ws://10.10.0.40:9944
```
