use crate::pallet::{
    ContractPublicIP, DeploymentHash, MaxDeploymentDataLength, MaxNodeContractPublicIPs,
};
use crate::Config;
use codec::{Decode, Encode, MaxEncodedLen};
use core::marker::PhantomData;
use frame_support::{pallet_prelude::ConstU32, BoundedVec, RuntimeDebugNoBound};
use scale_info::TypeInfo;
use sp_std::prelude::*;
use substrate_fixed::types::U64F64;
use tfchain_support::{resources::Resources, types::ConsumableResources};

pub type BlockNumber = u64;

/// Utility type for managing upgrades/migrations.
#[derive(Encode, Decode, Clone, Debug, PartialEq, TypeInfo, MaxEncodedLen, PartialOrd)]
pub enum StorageVersion {
    V1,
    V2,
    V3,
    V4,
    V5,
    V6,
    V7,
}

impl Default for StorageVersion {
    fn default() -> StorageVersion {
        StorageVersion::V7
    }
}

#[derive(Clone, Eq, PartialEq, RuntimeDebugNoBound, Encode, Decode, TypeInfo)]
pub struct Group {
    pub id: u32,
    pub twin_id: u32,
    pub capacity_reservation_contract_ids: Vec<u64>,
}

#[derive(Clone, Eq, PartialEq, RuntimeDebugNoBound, Encode, Decode, TypeInfo)]
pub struct NodeGroupConfig {
    pub node_id: u32,
    pub group_id: u32,
}

#[derive(Clone, Eq, PartialEq, RuntimeDebugNoBound, Encode, Decode, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct Contract<T: Config> {
    pub version: u32,
    pub state: ContractState,
    pub contract_id: u64,
    pub twin_id: u32,
    pub contract_type: ContractData<T>,
    pub solution_provider_id: Option<u64>,
}

impl<T: Config> Contract<T> {
    pub fn is_state_delete(&self) -> bool {
        matches!(self.state, ContractState::Deleted(_))
    }

    pub fn get_node_id(&self) -> u32 {
        match self.contract_type.clone() {
            ContractData::CapacityReservationContract(c) => c.node_id,
            ContractData::DeploymentContract(_) => 0,
            ContractData::NameContract(_) => 0,
        }
    }
}

#[derive(Clone, Eq, PartialEq, RuntimeDebugNoBound, Encode, Decode, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct CapacityReservationContract {
    pub node_id: u32,
    pub resources: ConsumableResources,
    pub group_id: Option<u32>,
    pub public_ips: u32,
    pub deployment_contracts: Vec<u64>,
}

#[derive(Clone, Eq, PartialEq, RuntimeDebugNoBound, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct DeploymentContract<T: Config> {
    pub capacity_reservation_id: u64,
    // Hash of the deployment, set by the user
    // Max 32 bytes
    pub deployment_hash: DeploymentHash,
    pub deployment_data: BoundedVec<u8, MaxDeploymentDataLength<T>>,
    pub public_ips: u32,
    pub public_ips_list: BoundedVec<ContractPublicIP<T>, MaxNodeContractPublicIPs<T>>,
    pub resources: Resources,
}

#[derive(Clone, Eq, PartialEq, RuntimeDebugNoBound, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct NameContract<T: Config> {
    pub name: T::NameContractName,
}

// metadata length limited to 64 bytes (2 public keys)
pub const MAX_METADATA_LENGTH: u32 = 64;

#[derive(Clone, Eq, PartialEq, RuntimeDebugNoBound, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct ServiceContract<T: Config> {
    pub service_twin_id: u32,
    pub consumer_twin_id: u32,
    pub base_fee: u64,
    pub variable_fee: u64,
    pub metadata: BoundedVec<u8, ConstU32<MAX_METADATA_LENGTH>>,
    pub accepted_by_service: bool,
    pub accepted_by_consumer: bool,
    pub last_bill: u64,
    pub state: ServiceContractState,
    pub phantom: PhantomData<T>,
}

#[derive(
    PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo, MaxEncodedLen,
)]
pub struct ServiceContractBill {
    pub variable_amount: u64, // variable amount which is billed
    pub window: u64,          // amount of time (in seconds) covered since last bill
    pub metadata: BoundedVec<u8, ConstU32<50>>, // limited to 50 bytes for now
}

#[derive(Clone, Eq, PartialEq, RuntimeDebugNoBound, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum ServiceContractState {
    Created,
    AgreementReady,
    ApprovedByBoth,
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

#[derive(Clone, Eq, PartialEq, RuntimeDebugNoBound, Encode, Decode, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub enum ContractData<T: Config> {
    DeploymentContract(DeploymentContract<T>),
    NameContract(NameContract<T>),
    CapacityReservationContract(CapacityReservationContract),
}

// impl<T: Config> Default for ContractData<T> {
//     fn default() -> ContractData<T> {
//         ContractData::CapacityReservationContract(CapacityReservationContract::default())
//     }
// }

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

// DEPRECATED
#[derive(
    PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo, MaxEncodedLen,
)]
pub struct ContractLock<BalanceOf> {
    pub amount_locked: BalanceOf,
    pub lock_updated: u64,
    pub cycles: u16,
}

#[derive(
    PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo, MaxEncodedLen,
)]
pub struct CapacityReservationLock<BalanceOf> {
    pub amount_locked: BalanceOf,
    pub lock_updated: u64,
    pub cycles: u16,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo)]
pub struct SolutionProvider<AccountId> {
    pub solution_provider_id: u64,
    pub providers: Vec<Provider<AccountId>>,
    pub description: Vec<u8>,
    pub link: Vec<u8>,
    pub approved: bool,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo)]
pub struct Provider<AccountId> {
    pub who: AccountId,
    pub take: u8,
}
