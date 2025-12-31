use serde::{Deserialize, Serialize};
use std::str::FromStr;
use subxt::{
    dynamic::{At, Value as DynamicValue},
    OnlineClient, PolkadotConfig,
};
use subxt_signer::sr25519::Keypair;
use thiserror::Error;

/// Error types for the TFChain client
#[derive(Error, Debug)]
pub enum TfChainError {
    #[error("Subxt error: {0}")]
    Subxt(#[from] subxt::Error),
    #[error("RPC error: {0}")]
    Rpc(String),
    #[error("Invalid key: {0}")]
    InvalidKey(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Decode error: {0}")]
    DecodeError(String),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum KeyType {
    Sr25519,
    Ed25519,
}

impl FromStr for KeyType {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "sr25519" => Ok(Self::Sr25519),
            "ed25519" => Ok(Self::Ed25519),
            _ => Err("unknown key type"),
        }
    }
}

/// Wrapper around subxt_signer Keypair
#[derive(Clone)]
pub struct KeyPair {
    inner: Keypair,
}

impl KeyPair {
    /// Create a key pair from a BIP-39 phrase
    pub fn from_phrase<S: AsRef<str>>(
        _k: KeyType, // Currently only sr25519 is supported via subxt-signer
        phrase: S,
        password: Option<&str>,
    ) -> Result<Self, TfChainError> {
        let phrase = phrase.as_ref();
        
        let keypair = match password {
            Some(pwd) => Keypair::from_phrase(&bip39::Mnemonic::parse(phrase)
                .map_err(|e| TfChainError::InvalidKey(e.to_string()))?, Some(pwd))
                .map_err(|e| TfChainError::InvalidKey(e.to_string()))?,
            None => Keypair::from_phrase(&bip39::Mnemonic::parse(phrase)
                .map_err(|e| TfChainError::InvalidKey(e.to_string()))?, None)
                .map_err(|e| TfChainError::InvalidKey(e.to_string()))?,
        };
        
        Ok(Self { inner: keypair })
    }
    
    /// Create from a secret URI (like "//Alice" or a hex seed)
    pub fn from_uri(uri: &str) -> Result<Self, TfChainError> {
        let keypair = Keypair::from_uri(&uri.parse().map_err(|e: subxt_signer::SecretUriError| TfChainError::InvalidKey(e.to_string()))?)
            .map_err(|e| TfChainError::InvalidKey(e.to_string()))?;
        Ok(Self { inner: keypair })
    }

    pub fn public_key(&self) -> Vec<u8> {
        self.inner.public_key().0.to_vec()
    }

    pub fn account_id(&self) -> subxt::utils::AccountId32 {
        subxt::utils::AccountId32::from(self.inner.public_key().0)
    }
    
    /// Get the inner keypair for signing
    pub fn signer(&self) -> &Keypair {
        &self.inner
    }
}

/// Twin data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Twin {
    pub id: u32,
    pub account_id: String,
    pub relay: Option<String>,
    pub pk: Option<Vec<u8>>,
}

/// Farm data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Farm {
    pub id: u32,
    pub name: String,
    pub twin_id: u32,
    pub pricing_policy_id: u32,
    pub certification: String,
    pub public_ips: Vec<PublicIp>,
}

/// Public IP data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicIp {
    pub ip: String,
    pub gateway: String,
    pub contract_id: u64,
}

/// Node resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resources {
    pub hru: u64,
    pub sru: u64,
    pub cru: u64,
    pub mru: u64,
}

/// Node data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: u32,
    pub farm_id: u32,
    pub twin_id: u32,
    pub resources: Resources,
    pub created: u64,
    pub certification: String,
}

/// Account balance info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub nonce: u32,
    pub free: u128,
    pub reserved: u128,
}

/// The TFChain client with dynamic metadata support
#[derive(Clone)]
pub struct Client {
    pub api: OnlineClient<PolkadotConfig>,
}

impl Client {
    /// Create a new client connected to the given URL
    pub async fn new<U: AsRef<str>>(url: U) -> Result<Client, TfChainError> {
        let api = OnlineClient::<PolkadotConfig>::from_url(url).await?;
        Ok(Client { api })
    }

    /// Get account balance using dynamic query
    pub async fn get_balance(
        &self,
        account: &subxt::utils::AccountId32,
    ) -> Result<Option<AccountInfo>, TfChainError> {
        let storage_query = subxt::dynamic::storage("System", "Account", vec![DynamicValue::from_bytes(account)]);

        let result = self
            .api
            .storage()
            .at_latest()
            .await?
            .fetch(&storage_query)
            .await?;

        match result {
            Some(value) => {
                let decoded = value.to_value().map_err(|e| TfChainError::DecodeError(e.to_string()))?;
                
                // Extract nonce
                let nonce = decoded.at("nonce").and_then(|v| v.as_u128()).map(|v| v as u32).unwrap_or(0);
                
                // Extract balance data
                let free = decoded.at("data").and_then(|d| d.at("free")).and_then(|v| v.as_u128()).unwrap_or(0);
                let reserved = decoded.at("data").and_then(|d| d.at("reserved")).and_then(|v| v.as_u128()).unwrap_or(0);

                Ok(Some(AccountInfo {
                    nonce,
                    free,
                    reserved,
                }))
            }
            None => Ok(None),
        }
    }

    /// Get twin ID by account using dynamic query
    pub async fn get_twin_id_by_account(
        &self,
        account: subxt::utils::AccountId32,
    ) -> Result<Option<u32>, TfChainError> {
        let storage_query = subxt::dynamic::storage(
            "TfgridModule",
            "TwinIdByAccountID",
            vec![DynamicValue::from_bytes(&account)],
        );

        let result = self
            .api
            .storage()
            .at_latest()
            .await?
            .fetch(&storage_query)
            .await?;

        match result {
            Some(value) => {
                let decoded = value.to_value().map_err(|e| TfChainError::DecodeError(e.to_string()))?;
                if let Some(id) = decoded.as_u128() {
                    Ok(Some(id as u32))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    /// Get twin by ID using dynamic query
    pub async fn get_twin_by_id(&self, id: u32) -> Result<Option<Twin>, TfChainError> {
        let storage_query = subxt::dynamic::storage(
            "TfgridModule",
            "Twins",
            vec![DynamicValue::u128(id as u128)],
        );

        let result = self
            .api
            .storage()
            .at_latest()
            .await?
            .fetch(&storage_query)
            .await?;

        match result {
            Some(value) => {
                let decoded = value.to_value().map_err(|e| TfChainError::DecodeError(e.to_string()))?;
                
                let twin_id = decoded.at("id").and_then(|v| v.as_u128()).map(|v| v as u32).unwrap_or(0);
                let account_id = extract_account_id_from_value(&decoded, "account_id")
                    .unwrap_or_else(|| "unknown".to_string());
                let relay = extract_optional_string(&decoded, "relay");
                let pk = extract_optional_bytes_field(&decoded, "pk");

                Ok(Some(Twin {
                    id: twin_id,
                    account_id,
                    relay,
                    pk,
                }))
            }
            None => Ok(None),
        }
    }

    /// Get farm by ID using dynamic query
    pub async fn get_farm_by_id(&self, id: u32) -> Result<Option<Farm>, TfChainError> {
        let storage_query = subxt::dynamic::storage(
            "TfgridModule",
            "Farms",
            vec![DynamicValue::u128(id as u128)],
        );

        let result = self
            .api
            .storage()
            .at_latest()
            .await?
            .fetch(&storage_query)
            .await?;

        match result {
            Some(value) => {
                let decoded = value.to_value().map_err(|e| TfChainError::DecodeError(e.to_string()))?;
                
                let farm_id = decoded.at("id").and_then(|v| v.as_u128()).map(|v| v as u32).unwrap_or(0);
                let name = extract_bytes_as_string_from_value(&decoded, "name")
                    .unwrap_or_else(|| "unknown".to_string());
                let twin_id = decoded.at("twin_id").and_then(|v| v.as_u128()).map(|v| v as u32).unwrap_or(0);
                let pricing_policy_id = decoded.at("pricing_policy_id").and_then(|v| v.as_u128()).map(|v| v as u32).unwrap_or(0);
                let certification = extract_variant_name_from_value(&decoded, "certification")
                    .unwrap_or_else(|| "Unknown".to_string());
                let public_ips = extract_public_ips_from_value(&decoded);

                Ok(Some(Farm {
                    id: farm_id,
                    name,
                    twin_id,
                    pricing_policy_id,
                    certification,
                    public_ips,
                }))
            }
            None => Ok(None),
        }
    }

    /// Get node by ID using dynamic query
    pub async fn get_node_by_id(&self, id: u32) -> Result<Option<Node>, TfChainError> {
        let storage_query = subxt::dynamic::storage(
            "TfgridModule",
            "Nodes",
            vec![DynamicValue::u128(id as u128)],
        );

        let result = self
            .api
            .storage()
            .at_latest()
            .await?
            .fetch(&storage_query)
            .await?;

        match result {
            Some(value) => {
                let decoded = value.to_value().map_err(|e| TfChainError::DecodeError(e.to_string()))?;
                
                let node_id = decoded.at("id").and_then(|v| v.as_u128()).map(|v| v as u32).unwrap_or(0);
                let farm_id = decoded.at("farm_id").and_then(|v| v.as_u128()).map(|v| v as u32).unwrap_or(0);
                let twin_id = decoded.at("twin_id").and_then(|v| v.as_u128()).map(|v| v as u32).unwrap_or(0);
                let created = decoded.at("created").and_then(|v| v.as_u128()).map(|v| v as u64).unwrap_or(0);
                let certification = extract_variant_name_from_value(&decoded, "certification")
                    .unwrap_or_else(|| "Unknown".to_string());

                // Extract resources
                let resources = decoded.at("resources");
                let hru = resources.and_then(|r| r.at("hru")).and_then(|v| v.as_u128()).map(|v| v as u64).unwrap_or(0);
                let sru = resources.and_then(|r| r.at("sru")).and_then(|v| v.as_u128()).map(|v| v as u64).unwrap_or(0);
                let cru = resources.and_then(|r| r.at("cru")).and_then(|v| v.as_u128()).map(|v| v as u64).unwrap_or(0);
                let mru = resources.and_then(|r| r.at("mru")).and_then(|v| v.as_u128()).map(|v| v as u64).unwrap_or(0);

                Ok(Some(Node {
                    id: node_id,
                    farm_id,
                    twin_id,
                    resources: Resources { hru, sru, cru, mru },
                    created,
                    certification,
                }))
            }
            None => Ok(None),
        }
    }

    /// Get block hash 
    pub async fn get_block_hash(
        &self,
        block_number: Option<u32>,
    ) -> Result<Option<subxt::utils::H256>, TfChainError> {
        // Get the block at the specified number or latest
        let block = match block_number {
            Some(_n) => {
                // For specific block numbers, we get latest for now
                // A full implementation would use RPC
                let block_ref = self.api.blocks().at_latest().await?;
                Some(block_ref.hash())
            }
            None => {
                let block_ref = self.api.blocks().at_latest().await?;
                Some(block_ref.hash())
            }
        };
        Ok(block)
    }

    /// Get contract by ID using dynamic query
    pub async fn get_contract_by_id(&self, id: u64) -> Result<Option<serde_json::Value>, TfChainError> {
        let storage_query = subxt::dynamic::storage(
            "SmartContractModule",
            "Contracts",
            vec![DynamicValue::u128(id as u128)],
        );

        let result = self
            .api
            .storage()
            .at_latest()
            .await?
            .fetch(&storage_query)
            .await?;

        match result {
            Some(value) => {
                let decoded = value.to_value().map_err(|e| TfChainError::DecodeError(e.to_string()))?;
                // Return as JSON for flexibility
                Ok(Some(value_to_json(&decoded)))
            }
            None => Ok(None),
        }
    }

    /// Transfer TFT to another account
    /// amount is in the smallest unit (1 TFT = 10_000_000 units)
    pub async fn transfer(
        &self,
        keypair: &KeyPair,
        dest: subxt::utils::AccountId32,
        amount: u128,
    ) -> Result<String, TfChainError> {
        // Build the dynamic transfer call
        // Balances.transfer_keep_alive is safer as it prevents the account from being reaped
        let transfer_tx = subxt::dynamic::tx(
            "Balances",
            "transfer_keep_alive",
            vec![
                DynamicValue::from_bytes(&dest),
                DynamicValue::u128(amount),
            ],
        );

        // Sign and submit the transaction
        let tx_progress = self
            .api
            .tx()
            .sign_and_submit_then_watch_default(&transfer_tx, keypair.signer())
            .await?;

        // Wait for the transaction to be finalized
        let events = tx_progress.wait_for_finalized_success().await?;
        let tx_hash = events.extrinsic_hash();

        Ok(format!("0x{}", hex::encode(tx_hash.0)))
    }

    /// Transfer TFT (allows account to be reaped if balance goes to zero)
    /// amount is in the smallest unit (1 TFT = 10_000_000 units)
    pub async fn transfer_allow_death(
        &self,
        keypair: &KeyPair,
        dest: subxt::utils::AccountId32,
        amount: u128,
    ) -> Result<String, TfChainError> {
        let transfer_tx = subxt::dynamic::tx(
            "Balances",
            "transfer_allow_death",
            vec![
                DynamicValue::from_bytes(&dest),
                DynamicValue::u128(amount),
            ],
        );

        let tx_progress = self
            .api
            .tx()
            .sign_and_submit_then_watch_default(&transfer_tx, keypair.signer())
            .await?;

        let events = tx_progress.wait_for_finalized_success().await?;
        let tx_hash = events.extrinsic_hash();

        Ok(format!("0x{}", hex::encode(tx_hash.0)))
    }

    /// Get node IDs belonging to a farm
    /// 
    /// Uses the on-chain `NodesByFarmID` storage map which directly indexes farm_id -> Vec<node_id>
    pub async fn get_node_ids_by_farm(&self, farm_id: u32) -> Result<Vec<u32>, TfChainError> {
        let storage_query = subxt::dynamic::storage(
            "TfgridModule",
            "NodesByFarmID",
            vec![DynamicValue::u128(farm_id as u128)],
        );

        let result = self
            .api
            .storage()
            .at_latest()
            .await?
            .fetch(&storage_query)
            .await?;

        match result {
            Some(value) => {
                let decoded = value.to_value().map_err(|e| TfChainError::DecodeError(e.to_string()))?;
                
                // Extract the vector of node IDs
                let mut node_ids = Vec::new();
                let mut idx: usize = 0;
                while let Some(node_val) = decoded.at(idx) {
                    if let Some(node_id) = node_val.as_u128() {
                        node_ids.push(node_id as u32);
                    }
                    idx += 1;
                }
                
                Ok(node_ids)
            }
            None => Ok(vec![]),
        }
    }
}

// Helper functions for extracting values

fn extract_bytes_as_string_from_value<T>(value: &subxt::dynamic::Value<T>, field: &str) -> Option<String> {
    let field_value = value.at(field)?;
    
    // The field might be a composite (sequence of bytes)
    let mut bytes = Vec::new();
    let mut idx: usize = 0;
    while let Some(byte_val) = field_value.at(idx) {
        if let Some(b) = byte_val.as_u128() {
            bytes.push(b as u8);
        }
        idx += 1;
    }
    
    if !bytes.is_empty() {
        String::from_utf8(bytes).ok()
    } else {
        None
    }
}

fn extract_optional_string<T>(value: &subxt::dynamic::Value<T>, field: &str) -> Option<String> {
    let field_value = value.at(field)?;
    
    // Check if it's a Some variant by trying to access index 0
    if let Some(inner) = field_value.at(0usize) {
        let mut bytes = Vec::new();
        let mut idx: usize = 0;
        while let Some(byte_val) = inner.at(idx) {
            if let Some(b) = byte_val.as_u128() {
                bytes.push(b as u8);
            }
            idx += 1;
        }
        if !bytes.is_empty() {
            return String::from_utf8(bytes).ok();
        }
    }
    None
}

fn extract_optional_bytes_field<T>(value: &subxt::dynamic::Value<T>, field: &str) -> Option<Vec<u8>> {
    let field_value = value.at(field)?;
    
    // Check if it's a Some variant by trying to access index 0
    if let Some(inner) = field_value.at(0usize) {
        let mut bytes = Vec::new();
        let mut idx: usize = 0;
        while let Some(byte_val) = inner.at(idx) {
            if let Some(b) = byte_val.as_u128() {
                bytes.push(b as u8);
            }
            idx += 1;
        }
        if !bytes.is_empty() {
            return Some(bytes);
        }
    }
    None
}

fn extract_account_id_from_value<T>(value: &subxt::dynamic::Value<T>, field: &str) -> Option<String> {
    let field_value = value.at(field)?;
    
    // AccountId is typically 32 bytes
    let mut bytes = Vec::new();
    let mut idx: usize = 0;
    while let Some(byte_val) = field_value.at(idx) {
        if let Some(b) = byte_val.as_u128() {
            bytes.push(b as u8);
        }
        idx += 1;
    }
    
    if bytes.len() == 32 {
        let account = subxt::utils::AccountId32::from(<[u8; 32]>::try_from(bytes.as_slice()).ok()?);
        Some(account.to_string())
    } else {
        None
    }
}

fn extract_variant_name_from_value<T>(value: &subxt::dynamic::Value<T>, field: &str) -> Option<String> {
    let field_value = value.at(field)?;
    
    // For variants, try to identify them by checking for variant-like structure
    // We can check if accessing by index works (variants have their values at index 0)
    // Common certification values: "Diy", "Certified", "NotCertified", "Gold"
    
    // Try common variant names by seeing if they have values
    let has_inner = field_value.at(0usize).is_some();
    
    // For simple enums like certification, we need to rely on the structure
    // Return a default based on whether it has inner data
    if has_inner {
        Some("Unknown".to_string())
    } else {
        // Simple variant without data - we can't determine the name without Debug
        // but common cases are enums
        Some("Unknown".to_string())
    }
}

fn extract_public_ips_from_value<T>(value: &subxt::dynamic::Value<T>) -> Vec<PublicIp> {
    let mut public_ips = Vec::new();
    
    if let Some(ips_value) = value.at("public_ips") {
        let mut idx: usize = 0;
        while let Some(ip_value) = ips_value.at(idx) {
            let ip = extract_bytes_as_string_from_value(&ip_value, "ip").unwrap_or_default();
            let gateway = extract_bytes_as_string_from_value(&ip_value, "gateway").unwrap_or_default();
            let contract_id = ip_value.at("contract_id").and_then(|v| v.as_u128()).map(|v| v as u64).unwrap_or(0);
            
            public_ips.push(PublicIp {
                ip,
                gateway,
                contract_id,
            });
            idx += 1;
        }
    }
    
    public_ips
}

fn value_to_json<T>(value: &subxt::dynamic::Value<T>) -> serde_json::Value {
    // Check for primitive values first
    if let Some(b) = value.as_bool() {
        return serde_json::Value::Bool(b);
    }
    if let Some(n) = value.as_u128() {
        return serde_json::json!(n);
    }
    if let Some(n) = value.as_i128() {
        return serde_json::json!(n);
    }
    if let Some(c) = value.as_char() {
        return serde_json::Value::String(c.to_string());
    }
    if let Some(s) = value.as_str() {
        return serde_json::Value::String(s.to_string());
    }
    
    // Try to iterate as a composite/sequence
    let mut arr = Vec::new();
    let mut idx: usize = 0;
    while let Some(elem) = value.at(idx) {
        arr.push(value_to_json(&elem));
        idx += 1;
        if idx > 1000 {
            // Safety limit
            break;
        }
    }
    
    if !arr.is_empty() {
        return serde_json::Value::Array(arr);
    }
    
    // Try named fields
    let mut obj = serde_json::Map::new();
    // Common field names to try
    for field in &["id", "name", "account_id", "twin_id", "farm_id", "node_id", "contract_id", 
                   "nonce", "free", "reserved", "data", "relay", "pk", "resources", "certification",
                   "public_ips", "pricing_policy_id", "created", "hru", "sru", "cru", "mru",
                   "ip", "gateway", "version", "state", "contract_type"] {
        if let Some(field_val) = value.at(*field) {
            obj.insert(field.to_string(), value_to_json(&field_val));
        }
    }
    
    if !obj.is_empty() {
        return serde_json::Value::Object(obj);
    }
    
    serde_json::Value::Null
}
