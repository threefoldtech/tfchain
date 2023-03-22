# TF Chain client

## Run the examples

```
cargo run
```

## Usage

```rust
// Create a keypair from phrase or secret
let mnemonic = "<words>";
let p = client::get_from_seed(mnemonic, None);

// Specify the network (devnet, testnet or mainnet)
let network = "mainnet";

// Construct the client and provide a websocket endpoint and port
let cl = client::TfchainClient::new(String::from("ws://127.0.0.1:9944"), p, network).await?;

// Get a twin by ID (example)
let twin = cl.get_twin_by_id(1, None).await?;
println!("got twin: {:?}", twin);
```

## Development

When the runtime has changed you generate the base client like:

```
subxt metadata -f bytes --url http://localhost:9933 > artifacts/network.scale
```
