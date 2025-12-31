# TFChain Rust Client

A Rust client for interacting with TFChain using dynamic metadata (no compile-time codegen required).

## Features

### KeyPair

| Method | Description |
|--------|-------------|
| `from_phrase(key_type, mnemonic, password)` | Create keypair from BIP-39 mnemonic |
| `from_uri(uri)` | Create from secret URI (e.g., "//Alice") |
| `public_key()` | Get public key bytes |
| `account_id()` | Get AccountId32 |
| `signer()` | Get inner keypair for signing |

### Client - Queries (Read)

| Method | Description | On-Chain Storage |
|--------|-------------|------------------|
| `get_balance(account)` | Get account balance (free, reserved, nonce) | `System.Account` |
| `get_twin_id_by_account(account)` | Get twin ID for an account | `TfgridModule.TwinIdByAccountID` |
| `get_twin_by_id(id)` | Get twin details | `TfgridModule.Twins` |
| `get_farm_by_id(id)` | Get farm details | `TfgridModule.Farms` |
| `get_node_by_id(id)` | Get node details | `TfgridModule.Nodes` |
| `get_node_ids_by_farm(farm_id)` | Get list of node IDs for a farm | `TfgridModule.NodesByFarmID` |
| `get_contract_by_id(id)` | Get contract as JSON | `SmartContractModule.Contracts` |
| `get_block_hash(block_number)` | Get block hash | RPC |

### Client - Transactions (Write)

| Method | Description |
|--------|-------------|
| `transfer(keypair, dest, amount)` | Transfer TFT (keep_alive) |
| `transfer_allow_death(keypair, dest, amount)` | Transfer TFT (allow reap) |

### Data Structures

- `Twin` - id, account_id, relay, pk
- `Farm` - id, name, twin_id, pricing_policy_id, certification, public_ips
- `Node` - id, farm_id, twin_id, resources, created, certification
- `Resources` - hru, sru, cru, mru
- `PublicIp` - ip, gateway, contract_id
- `AccountInfo` - nonce, free, reserved

## Usage

```rust
use tfchain_client::{Client, KeyPair, KeyType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create keypair from mnemonic
    let mnemonic = "your twelve word mnemonic phrase here";
    let keypair = KeyPair::from_phrase(KeyType::Sr25519, mnemonic, None)?;
    let account_id = keypair.account_id();

    // Connect to TFChain
    let client = Client::new("wss://tfchain.grid.tf:443").await?;

    // Query balance
    if let Some(info) = client.get_balance(&account_id).await? {
        let tft = info.free as f64 / 10_000_000.0;
        println!("Balance: {} TFT", tft);
    }

    // Get twin ID
    if let Some(twin_id) = client.get_twin_id_by_account(account_id).await? {
        println!("Twin ID: {}", twin_id);
        
        // Get twin details
        if let Some(twin) = client.get_twin_by_id(twin_id).await? {
            println!("Twin: {:?}", twin);
        }
    }

    // Get farm by ID
    if let Some(farm) = client.get_farm_by_id(1).await? {
        println!("Farm: {} (twin: {})", farm.name, farm.twin_id);
        
        // Get nodes in farm
        let node_ids = client.get_node_ids_by_farm(farm.id).await?;
        println!("Nodes: {:?}", node_ids);
    }

    // Get node by ID
    if let Some(node) = client.get_node_by_id(1).await? {
        println!("Node {} - {} cores, {} GB RAM", 
            node.id, 
            node.resources.cru,
            node.resources.mru / 1_000_000_000
        );
    }

    // Transfer TFT (1 TFT = 10_000_000 units)
    // let dest = "5GrwvaEF...".parse()?;
    // let tx_hash = client.transfer(&keypair, dest, 10_000_000).await?;

    Ok(())
}
```

## Network Endpoints

| Network | WebSocket URL |
|---------|---------------|
| Mainnet | `wss://tfchain.grid.tf:443` |
| Testnet | `wss://tfchain.test.grid.tf:443` |
| Devnet | `wss://tfchain.dev.grid.tf:443` |

## Run Examples

```bash
cargo run --example mainnet_test
```

## Note on Queries

This client only supports queries that have direct on-chain storage indexes. For queries like "get all farms by twin_id" or "get all nodes by twin_id", use the [Grid Proxy/Explorer API](https://gridproxy.grid.tf) which maintains these indexes:

- `GET /farms?twin_id=X` - Get farms by twin
- `GET /nodes?farm_ids=X` - Get nodes by farm
- `GET /nodes?owned_by=X` - Get nodes by twin
