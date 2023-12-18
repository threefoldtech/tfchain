# Trying out a runtime migration on a running network

List of available endpoints for our networks:

- Devnet: 10.10.0.44
- QAnet: 10.10.0.43
- Testnet: 10.10.0.100
- Mainnet: 10.10.0.154

(If you are not in lochristi office you probably need vpn to reach to ips)

## Select a network you wish to try the runtime upgrade upon

Now go the release you want to test with try-runtime and compile as following:

```sh
cargo run --release --features=try-runtime try-runtime --runtime ./target/release/wbuild/tfchain-runtime/tfchain_runtime.compact.wasm --chain chainspecs/NETWORK/chainSpecRaw.json on-runtime-upgrade live --uri NETWORK_URL
```
