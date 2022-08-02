use crate::pallet::DeploymentHash;
use crate::Config;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{traits::ConstU32, BoundedVec, RuntimeDebugNoBound};
use scale_info::TypeInfo;
use sp_std::prelude::*;
use substrate_fixed::types::U64F64;
use tfchain_support::types::{Resources};
use crate::pallet::ContractPublicIP;

pub type BlockNumber = u64;

/// Utility type for managing upgrades/migrations.
#[derive(Encode, Decode, Clone, Debug, PartialEq, TypeInfo, MaxEncodedLen)]
pub enum StorageVersion {
    V1,
    V2,
    V3,
    V4,
}

impl Default for StorageVersion {
    fn default() -> StorageVersion {
        StorageVersion::V3
    }
}

#[derive(Clone, Eq, PartialEq, RuntimeDebugNoBound, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct Contract<T: Config> {
    pub version: u32,
    pub state: ContractState,
    pub contract_id: u64,
    pub twin_id: u32,
    pub contract_type: ContractData<T>,
}

impl<T: Config> Contract<T> {
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

#[derive(Clone, Eq, PartialEq, RuntimeDebugNoBound, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct NodeContract<T: Config> {
    pub node_id: u32,
    // Hash of the deployment, set by the user
    // Max 32 bytes
    pub deployment_hash: DeploymentHash,
    pub public_ips: u32,
    pub public_ips_list: BoundedVec<
        ContractPublicIP<T>,
        ConstU32<5>,
    >,
}

#[derive(Clone, Eq, PartialEq, RuntimeDebugNoBound, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct NameContract<T: Config> {
    pub name: T::NameContractName,
}

#[derive(
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Encode,
    Decode,
    Default,
    RuntimeDebugNoBound,
    TypeInfo,
    MaxEncodedLen,
)]
pub struct RentContract {
    pub node_id: u32,
}

#[derive(Clone, Eq, PartialEq, RuntimeDebugNoBound, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub enum ContractData<T: Config> {
    NodeContract(NodeContract<T>),
    NameContract(NameContract<T>),
    RentContract(RentContract),
}

impl<T: Config> Default for ContractData<T> {
    fn default() -> ContractData<T> {
        ContractData::RentContract(RentContract::default())
    }
}

#[derive(
    PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo, MaxEncodedLen,
)]
pub struct ContractBillingInformation {
    pub previous_nu_reported: u64,
    pub last_updated: u64,
    pub amount_unbilled: u64,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Debug, TypeInfo, MaxEncodedLen)]
pub enum ContractState {
    Created,
    Deleted(Cause),
    GracePeriod(BlockNumber),
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Debug, TypeInfo, MaxEncodedLen)]
pub enum Cause {
    CanceledByUser,
    OutOfFunds,
}

impl Default for ContractState {
    fn default() -> ContractState {
        ContractState::Created
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Debug, TypeInfo, MaxEncodedLen)]
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

#[derive(
    PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo, MaxEncodedLen,
)]
pub struct Consumption {
    pub contract_id: u64,
    pub timestamp: u64,
    pub cru: u64,
    pub sru: u64,
    pub hru: u64,
    pub mru: u64,
    pub nru: u64,
}

#[derive(
    PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo, MaxEncodedLen,
)]
pub struct NruConsumption {
    pub contract_id: u64,
    pub timestamp: u64,
    pub window: u64,
    pub nru: u64,
}

#[derive(
    PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo, MaxEncodedLen,
)]
pub struct ContractBill {
    pub contract_id: u64,
    pub timestamp: u64,
    pub discount_level: DiscountLevel,
    pub amount_billed: u128,
}

#[derive(
    PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo, MaxEncodedLen,
)]
pub struct ContractResources {
    pub contract_id: u64,
    pub used: Resources,
}

#[derive(
    PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo, MaxEncodedLen,
)]
pub struct ContractLock<BalanceOf> {
    pub amount_locked: BalanceOf,
    pub lock_updated: u64,
    pub cycles: u16,
}
