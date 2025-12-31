use tfchain_client::{Client, KeyPair, KeyType};

const MAINNET_URL: &str = "wss://tfchain.grid.tf:443";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for better logging
    tracing_subscriber::fmt::init();

    println!("=== TFChain Mainnet Client Test (Dynamic Metadata) ===\n");

    // Test mnemonic 
    let mnemonic = "...";

    // Create keypair from mnemonic
    println!("Creating keypair from mnemonic...");
    let keypair = KeyPair::from_phrase(KeyType::Sr25519, mnemonic, None)?;
    
    // Get the account ID from the keypair
    let account_id = keypair.account_id();
    println!("Public key: 0x{}", hex::encode(keypair.public_key()));
    println!("Account ID: {}", account_id);

    // Connect to mainnet
    println!("\nConnecting to TFChain Mainnet at {}...", MAINNET_URL);
    let client = Client::new(MAINNET_URL).await?;
    println!("Connected successfully!\n");

    // Test 1: Get latest block hash (RPC call)
    println!("--- Test 1: Get Latest Block Hash ---");
    match client.get_block_hash(None).await {
        Ok(Some(hash)) => {
            println!("  Latest block hash: 0x{}", hex::encode(hash.0));
        }
        Ok(None) => {
            println!("  Could not get block hash");
        }
        Err(e) => {
            println!("  Error: {}", e);
        }
    }

    // Test 2: Get a specific block hash (block 1)
    println!("\n--- Test 2: Get Block Hash for Block 1 ---");
    match client.get_block_hash(Some(1)).await {
        Ok(Some(hash)) => {
            println!("  Block 1 hash: 0x{}", hex::encode(hash.0));
        }
        Ok(None) => {
            println!("  Could not get block hash for block 1");
        }
        Err(e) => {
            println!("  Error: {}", e);
        }
    }

    // Test 3: Get account balance (dynamic storage query)
    println!("\n--- Test 3: Get Account Balance ---");
    match client.get_balance(&account_id).await {
        Ok(Some(account_info)) => {
            // TFT has 7 decimal places
            let free_tft = account_info.free as f64 / 10_000_000.0;
            let reserved_tft = account_info.reserved as f64 / 10_000_000.0;
            println!("  Free balance: {} TFT ({} units)", free_tft, account_info.free);
            println!("  Reserved balance: {} TFT ({} units)", reserved_tft, account_info.reserved);
            println!("  Nonce: {}", account_info.nonce);
        }
        Ok(None) => {
            println!("  Account not found on chain (no balance)");
        }
        Err(e) => {
            println!("  Error: {}", e);
        }
    }

    // Test 4: Get twin ID by account (dynamic storage query)
    println!("\n--- Test 4: Get Twin ID by Account ---");
    match client.get_twin_id_by_account(account_id.clone()).await {
        Ok(Some(twin_id)) => {
            println!("  Twin ID: {}", twin_id);

            // Test 5: Get twin details
            println!("\n--- Test 5: Get Twin Details ---");
            match client.get_twin_by_id(twin_id).await {
                Ok(Some(twin)) => {
                    println!("  Twin ID: {}", twin.id);
                    println!("  Account: {}", twin.account_id);
                    if let Some(relay) = &twin.relay {
                        println!("  Relay: {}", relay);
                    } else {
                        println!("  Relay: None");
                    }
                    if let Some(pk) = &twin.pk {
                        println!("  Public Key: 0x{}", hex::encode(pk));
                    } else {
                        println!("  Public Key: None");
                    }
                }
                Ok(None) => {
                    println!("  Twin not found");
                }
                Err(e) => {
                    println!("  Error: {}", e);
                }
            }
        }
        Ok(None) => {
            println!("  No twin found for this account");
        }
        Err(e) => {
            println!("  Error: {}", e);
        }
    }

    // Test 6: Get a known farm (Farm ID 1)
    println!("\n--- Test 6: Get Farm by ID (Farm 1) ---");
    match client.get_farm_by_id(1).await {
        Ok(Some(farm)) => {
            println!("  Farm ID: {}", farm.id);
            println!("  Name: {}", farm.name);
            println!("  Twin ID: {}", farm.twin_id);
            println!("  Pricing Policy ID: {}", farm.pricing_policy_id);
            println!("  Certification: {}", farm.certification);
            println!("  Public IPs count: {}", farm.public_ips.len());
            // Show first 3 public IPs
            for (i, ip) in farm.public_ips.iter().take(3).enumerate() {
                println!("    IP {}: {} (gateway: {}, contract: {})", 
                    i + 1, ip.ip, ip.gateway, ip.contract_id);
            }
            if farm.public_ips.len() > 3 {
                println!("    ... and {} more", farm.public_ips.len() - 3);
            }
        }
        Ok(None) => {
            println!("  Farm 1 not found");
        }
        Err(e) => {
            println!("  Error: {}", e);
        }
    }

    // Test 7: Get a known node (Node ID 1)
    println!("\n--- Test 7: Get Node by ID (Node 1) ---");
    match client.get_node_by_id(1).await {
        Ok(Some(node)) => {
            println!("  Node ID: {}", node.id);
            println!("  Farm ID: {}", node.farm_id);
            println!("  Twin ID: {}", node.twin_id);
            println!("  Resources:");
            println!("    HRU: {} bytes ({:.2} TB)", node.resources.hru, node.resources.hru as f64 / 1e12);
            println!("    SRU: {} bytes ({:.2} TB)", node.resources.sru, node.resources.sru as f64 / 1e12);
            println!("    CRU: {} cores", node.resources.cru);
            println!("    MRU: {} bytes ({:.2} GB)", node.resources.mru, node.resources.mru as f64 / 1e9);
            println!("  Created: {}", node.created);
            println!("  Certification: {}", node.certification);
        }
        Ok(None) => {
            println!("  Node 1 not found");
        }
        Err(e) => {
            println!("  Error: {}", e);
        }
    }

    // Test 8: Get nodes by farm (for Farm 1)
    println!("\n--- Test 8: Get Nodes by Farm (Farm 1) ---");
    match client.get_node_ids_by_farm(1).await {
        Ok(node_ids) => {
            println!("  Farm 1 has {} nodes", node_ids.len());
            if !node_ids.is_empty() {
                println!("  Node IDs: {:?}", &node_ids[..std::cmp::min(10, node_ids.len())]);
                if node_ids.len() > 10 {
                    println!("  ... and {} more", node_ids.len() - 10);
                }
            }
        }
        Err(e) => {
            println!("  Error: {}", e);
        }
    }

    // Note: Transfer test is commented out to avoid accidental transfers
    // Uncomment to test transfers (will send real TFT!)
    /*
    println!("\n--- Test 11: Transfer TFT ---");
    let dest = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".parse::<subxt::utils::AccountId32>().unwrap();
    let amount = 1_0000000u128; // 1 TFT (7 decimal places)
    match client.transfer(&keypair, dest, amount).await {
        Ok(tx_hash) => {
            println!("  Transfer successful!");
            println!("  Transaction hash: {}", tx_hash);
        }
        Err(e) => {
            println!("  Transfer failed: {}", e);
        }
    }
    */

    println!("\n=== All tests completed! ===");

    Ok(())
}
