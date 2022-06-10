use codec::{Decode, Encode};
use frame_support::traits::Vec;
use substrate_fixed::types::U64F64;
use tfchain_support::types::{PublicIP, Resources};

pub type BlockNumber = u64;

/// Utility type for managing upgrades/migrations.
#[derive(Encode, Decode, Clone, Debug, PartialEq)]
pub enum PalletStorageVersion {
    V1,
    V2,
    V3,
    V4,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct Contract {
    pub version: u32,
    pub state: ContractState,
    pub contract_id: u64,
    pub twin_id: u32,
    pub contract_type: ContractData,
    pub solution_provider_id: Option<u64>,
}

impl Contract {
    pub fn is_state_delete(&self) -> bool {
        matches!(self.state, ContractState::Deleted(_))
    }

    pub fn get_node_id(&self) -> u32 {
        match self.contract_type.clone() {
            ContractData::RentContract(c) => c.node_id,
            ContractData::NodeContract(c) => c.node_id,
            ContractData::NameContract(_) => 0,
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct NodeContract {
    pub node_id: u32,
    // deployment_data is the encrypted deployment body. This encrypted the deployment with the **USER** public key.
    // So only the user can read this data later on (or any other key that he keeps safe).
    // this data part is read only by the user and can actually hold any information to help him reconstruct his deployment or can be left empty.
    pub deployment_data: Vec<u8>,
    // Hash of the deployment, set by the user
    pub deployment_hash: Vec<u8>,
    pub public_ips: u32,
    pub public_ips_list: Vec<PublicIP>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct NameContract {
    pub name: Vec<u8>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct RentContract {
    pub node_id: u32,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Debug)]
pub enum ContractData {
    NodeContract(NodeContract),
    NameContract(NameContract),
    RentContract(RentContract),
}

impl Default for ContractData {
    fn default() -> ContractData {
        ContractData::NodeContract(NodeContract::default())
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct ContractBillingInformation {
    pub previous_nu_reported: u64,
    pub last_updated: u64,
    pub amount_unbilled: u64,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Debug)]
pub enum ContractState {
    Created,
    Deleted(Cause),
    GracePeriod(BlockNumber),
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Debug)]
pub enum Cause {
    CanceledByUser,
    OutOfFunds,
}

impl Default for ContractState {
    fn default() -> ContractState {
        ContractState::Created
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Debug)]
pub enum DiscountLevel {
    None,
    Default,
    Bronze,
    Silver,
    Gold,
}

impl Default for DiscountLevel {
    fn default() -> DiscountLevel {
        DiscountLevel::None
    }
}

impl DiscountLevel {
    pub fn price_multiplier(&self) -> U64F64 {
        match self {
            DiscountLevel::None => U64F64::from_num(1),
            DiscountLevel::Default => U64F64::from_num(0.8),
            DiscountLevel::Bronze => U64F64::from_num(0.7),
            DiscountLevel::Silver => U64F64::from_num(0.6),
            DiscountLevel::Gold => U64F64::from_num(0.4),
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct Consumption {
    pub contract_id: u64,
    pub timestamp: u64,
    pub cru: u64,
    pub sru: u64,
    pub hru: u64,
    pub mru: u64,
    pub nru: u64,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct NruConsumption {
    pub contract_id: u64,
    pub timestamp: u64,
    pub window: u64,
    pub nru: u64,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct ContractBill {
    pub contract_id: u64,
    pub timestamp: u64,
    pub discount_level: DiscountLevel,
    pub amount_billed: u128,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct ContractResources {
    pub contract_id: u64,
    pub used: Resources,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct ContractLock<BalanceOf> {
    pub amount_locked: BalanceOf,
    pub lock_updated: u64,
    pub cycles: u16,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct SolutionProvider<AccountId> {
    pub solution_provider_id: u64,
    pub providers: Vec<Provider<AccountId>>,
    pub description: Vec<u8>,
    pub link: Vec<u8>,
    pub approved: bool,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct Provider<AccountId> {
    pub who: AccountId,
    pub take: u8,
}
