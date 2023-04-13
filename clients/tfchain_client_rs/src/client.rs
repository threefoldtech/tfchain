use crate::runtimes::{mainnet, types};
use std::str::FromStr;
use subxt::utils::AccountId32;
use subxt::{
    ext::sp_core::{crypto::SecretStringError, ed25519, sr25519, Pair},
    tx::{PairSigner, Signer},
    Error, OnlineClient, PolkadotConfig,
};
pub use types::{BlockNumber, Contract, Hash, SystemAccountInfo, TfgridFarm, TfgridNode, Twin};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Runtime {
    Local,
    Devnet,
    Testnet,
    Mainnet,
}

impl FromStr for Runtime {
    type Err = &'static str;

    fn from_str(v: &str) -> Result<Self, Self::Err> {
        match v {
            "local" => Ok(Self::Local),
            "devnet" => Ok(Self::Devnet),
            "mainnet" => Ok(Self::Mainnet),
            "testnet" => Ok(Self::Testnet),
            _ => Err("unknown runtime"),
        }
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
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

#[derive(Clone)]
pub enum KeyPair {
    Sr25519(sr25519::Pair),
    Ed25519(ed25519::Pair),
}

impl KeyPair {
    // create a key pair from a seed prefixed with `0x`. or a BIP-39 phrase
    pub fn from_phrase<S: AsRef<str>>(
        k: KeyType,
        phrase: S,
        password: Option<&str>,
    ) -> Result<Self, SecretStringError> {
        let phrase = phrase.as_ref();

        let pair = match k {
            KeyType::Sr25519 => {
                let pair: sr25519::Pair = Pair::from_string(phrase, password)?;
                Self::Sr25519(pair)
            }
            KeyType::Ed25519 => {
                let pair: ed25519::Pair = Pair::from_string(phrase, password)?;
                Self::Ed25519(pair)
            }
        };

        Ok(pair)
    }

    pub fn signer(&self) -> KeySigner {
        match self {
            Self::Ed25519(pair) => KeySigner::Ed25519(PairSigner::new(pair.clone())),
            Self::Sr25519(pair) => KeySigner::Sr25519(PairSigner::new(pair.clone())),
        }
    }
}

pub enum KeySigner {
    Sr25519(PairSigner<PolkadotConfig, sr25519::Pair>),
    Ed25519(PairSigner<PolkadotConfig, ed25519::Pair>),
}

impl Signer<PolkadotConfig> for KeySigner {
    fn account_id(&self) -> &<PolkadotConfig as subxt::Config>::AccountId {
        match self {
            Self::Sr25519(signer) => signer.account_id(),
            Self::Ed25519(signer) => signer.account_id(),
        }
    }

    fn address(&self) -> <PolkadotConfig as subxt::Config>::Address {
        match self {
            Self::Sr25519(signer) => signer.address(),
            Self::Ed25519(signer) => signer.address(),
        }
    }

    fn sign(&self, signer_payload: &[u8]) -> <PolkadotConfig as subxt::Config>::Signature {
        match self {
            Self::Sr25519(signer) => signer.sign(signer_payload),
            Self::Ed25519(signer) => signer.sign(signer_payload),
        }
    }
}

impl From<sr25519::Pair> for KeyPair {
    fn from(value: sr25519::Pair) -> Self {
        Self::Sr25519(value)
    }
}

impl From<ed25519::Pair> for KeyPair {
    fn from(value: ed25519::Pair) -> Self {
        Self::Ed25519(value)
    }
}

#[derive(Clone)]
pub struct Client {
    pub runtime: Runtime,
    pub api: OnlineClient<PolkadotConfig>,
}

macro_rules! call {
    ($self:ident, $name:ident, $($arg:expr),+) => (
        match $self.runtime {
            Runtime::Local => local::$name($self, $($arg),+).await,
            Runtime::Devnet => devnet::$name($self, $($arg),+).await,
            Runtime::Testnet => testnet::$name($self, $($arg),+).await,
            Runtime::Mainnet => mainnet::$name($self, $($arg),+).await,
        }
    )
}

impl Client {
    pub async fn new<U: AsRef<str>>(url: U, runtime: Runtime) -> Result<Client, Error> {
        let api = OnlineClient::<PolkadotConfig>::from_url(url).await?;

        Ok(Client { api, runtime })
    }

    // Creates a twin and checks for success, twin ID is returned on success
    pub async fn create_twin(
        &self,
        kp: &KeyPair,
        relay: Option<String>,
        pk: Option<String>,
    ) -> Result<u32, Error> {
        call!(self, create_twin, kp, relay, pk)
    }

    // Updates a twin and checks for success, blockhash is returned on success
    pub async fn update_twin(
        &self,
        kp: &KeyPair,
        relay: Option<String>,
        pk: Option<&[u8]>,
    ) -> Result<Hash, Error> {
        call!(self, update_twin, kp, relay, pk)
    }

    // Signs terms and condition and checks for success, blockhash is returned on success
    pub async fn sign_terms_and_conditions(
        &self,
        kp: &KeyPair,
        document_link: String,
        document_hash: String,
    ) -> Result<Hash, Error> {
        call!(
            self,
            sign_terms_and_conditions,
            kp,
            document_link,
            document_hash
        )
    }

    pub async fn get_twin_by_id(&self, id: u32) -> Result<Option<Twin>, Error> {
        call!(self, get_twin_by_id, id)
    }

    pub async fn get_twin_id_by_account(&self, account: AccountId32) -> Result<Option<u32>, Error> {
        call!(self, get_twin_id_by_account, account)
    }

    pub async fn get_farm_by_id(&self, id: u32) -> Result<Option<TfgridFarm>, Error> {
        call!(self, get_farm_by_id, id)
    }

    pub async fn get_node_by_id(&self, id: u32) -> Result<Option<TfgridNode>, Error> {
        call!(self, get_node_by_id, id)
    }

    pub async fn get_balance(
        &self,
        account: &AccountId32,
    ) -> Result<Option<SystemAccountInfo>, Error> {
        call!(self, get_balance, account)
    }

    pub async fn get_block_hash(
        &self,
        block_number: Option<BlockNumber>,
    ) -> Result<Option<Hash>, Error> {
        call!(self, get_block_hash, block_number)
    }

    pub async fn get_contract_by_id(&self, id: u64) -> Result<Option<Contract>, Error> {
        call!(self, get_contract_by_id, id)
    }
}
