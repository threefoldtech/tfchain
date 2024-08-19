use crate::{
    pallet::{MaxDeploymentDataLength, MaxNodeContractPublicIPs},
    Call, Config,
};
use core::{convert::TryInto, ops::Add};
use frame_support::{
    pallet_prelude::ConstU32,
    traits::{DefensiveSaturating, IsSubType},
    BoundedVec, RuntimeDebugNoBound,
};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{DispatchInfoOf, SignedExtension, Zero},
    transaction_validity::{TransactionValidity, TransactionValidityError, ValidTransaction},
    SaturatedConversion,
};
use sp_std::{fmt::Debug, marker::PhantomData, prelude::*};
use substrate_fixed::types::U64F64;
use tfchain_support::{resources::Resources, types::PublicIP};
pub type BlockNumber = u64;

/// Utility type for managing upgrades/migrations.
#[derive(Encode, Decode, Clone, Debug, PartialEq, PartialOrd, TypeInfo, MaxEncodedLen)]
pub enum StorageVersion {
    V1,
    V2,
    V3,
    V4,
    V5,
    V6,
    V7,
    V8,
    V9,
    V10,
    V11,
}

impl Default for StorageVersion {
    fn default() -> StorageVersion {
        StorageVersion::V10
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
    pub solution_provider_id: Option<u64>,
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

// HexHash is hex encoded hash
pub type HexHash = [u8; 32];

#[derive(Clone, Eq, PartialEq, RuntimeDebugNoBound, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct NodeContract<T: Config> {
    pub node_id: u32,
    // Hash of the deployment, set by the user
    // Max 32 bytes
    pub deployment_hash: HexHash,
    pub deployment_data: BoundedVec<u8, MaxDeploymentDataLength<T>>,
    pub public_ips: u32,
    pub public_ips_list: BoundedVec<PublicIP, MaxNodeContractPublicIPs<T>>,
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
    CanceledByCollective,
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
    pub extra_amount_locked: BalanceOf,
    pub lock_updated: u64,
    pub cycles: u16,
}
#[derive(
    PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo, MaxEncodedLen,
)]
pub struct ContractPaymentState<BalanceOf> {
    pub standard_reserve: BalanceOf,
    pub additional_reserve: BalanceOf,
    pub standard_overdraft: BalanceOf,
    pub additional_overdraft: BalanceOf,
    pub last_updated_seconds: u64,
    pub cycles: u16,
}

impl<BalanceOf: Add<Output = BalanceOf> + Copy + TryInto<u64>> ContractLock<BalanceOf> {
    pub fn total_amount_locked(&self) -> BalanceOf {
        self.amount_locked + self.extra_amount_locked
    }

    pub fn has_some_amount_locked(&self) -> bool {
        self.total_amount_locked().saturated_into::<u64>() > 0
    }

    pub fn has_extra_amount_locked(&self) -> bool {
        self.extra_amount_locked.saturated_into::<u64>() > 0
    }

    pub fn is_migrated(&self) -> bool {
        self.lock_updated == 0
    }
}

impl<BalanceOf> ContractPaymentState<BalanceOf>
where
    BalanceOf: DefensiveSaturating + Copy + Zero + PartialOrd,
{
    // accumulate the standard reserve
    pub fn reserve_standard_amount(&mut self, amount: BalanceOf) {
        self.standard_reserve.defensive_saturating_accrue(amount);
    }
    // accumulate the additional reserve
    pub fn reserve_additional_amount(&mut self, amount: BalanceOf) {
        self.additional_reserve.defensive_saturating_accrue(amount);
    }
    // accumulate the standard overdraft
    pub fn overdraft_standard_amount(&mut self, amount: BalanceOf) {
        self.standard_overdraft.defensive_saturating_accrue(amount);
    }
    // accumulate the additional overdraft
    pub fn overdraft_additional_amount(&mut self, amount: BalanceOf) {
        self.additional_overdraft
            .defensive_saturating_accrue(amount);
    }

    // Method to settle the standard overdraft
    pub fn settle_overdraft_standard_amount(&mut self) {
        self.standard_reserve
            .defensive_saturating_accrue(self.standard_overdraft);
        self.standard_overdraft = BalanceOf::zero();
    }

    // Method to settle the additional overdraft
    pub fn settle_overdraft_additional_amount(&mut self) {
        self.additional_reserve
            .defensive_saturating_accrue(self.additional_overdraft);
        self.additional_overdraft = BalanceOf::zero();
    }

    // Method to settle both standard and additional overdraft
    pub fn settle_overdraft(&mut self) {
        self.settle_overdraft_standard_amount();
        self.settle_overdraft_additional_amount();
    }

    // Method to return the sum of standard_overdraft and additional_overdraft
    pub fn get_overdraft(&self) -> BalanceOf {
        self.standard_overdraft
            .defensive_saturating_add(self.additional_overdraft)
    }

    // Method to return the sum of standard_reserve_amount and additional_reserve_amount
    pub fn get_reserve(&self) -> BalanceOf {
        self.standard_reserve
            .defensive_saturating_add(self.additional_reserve)
    }

    // Method to return weather the contract has any reserved balance at all.
    pub fn has_reserve(&self) -> bool {
        !self.standard_reserve.is_zero() || !self.additional_reserve.is_zero()
    }

    // Method to return weather the contract has overdraft or not
    pub fn has_overdraft(&self) -> bool {
        !self.standard_overdraft.is_zero() || !self.additional_overdraft.is_zero()
    }

    // Method to settle partial overdraft
    pub fn settle_partial_overdraft(&mut self, amount: BalanceOf) {
        let mut remaining_amount = amount;

        // Settle additional overdraft first
        if remaining_amount > BalanceOf::zero() {
            if remaining_amount >= self.additional_overdraft {
                remaining_amount.defensive_saturating_reduce(self.additional_overdraft);
                self.additional_reserve
                    .defensive_saturating_accrue(self.additional_overdraft);
                self.additional_overdraft = BalanceOf::zero();
            } else {
                self.additional_overdraft
                    .defensive_saturating_reduce(remaining_amount);
                self.additional_reserve
                    .defensive_saturating_accrue(remaining_amount);
                remaining_amount = BalanceOf::zero();
            }
        }

        // Settle standard overdraft with any remaining amount
        if remaining_amount > BalanceOf::zero() {
            if remaining_amount >= self.standard_overdraft {
                remaining_amount.defensive_saturating_reduce(self.standard_overdraft);
                self.standard_reserve
                    .defensive_saturating_accrue(self.standard_overdraft);
                self.standard_overdraft = BalanceOf::zero();
            } else {
                self.standard_overdraft
                    .defensive_saturating_reduce(remaining_amount);
                self.standard_reserve
                    .defensive_saturating_accrue(remaining_amount);
            }
        }
    }

    pub fn reset_standard_reserve(&mut self) {
        self.standard_reserve = BalanceOf::zero();
    }

    pub fn reset_additional_reserve(&mut self) {
        self.additional_reserve = BalanceOf::zero();
    }

    pub fn reset_cycles(&mut self) {
        self.cycles = 0;
    }
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

pub const MAX_METADATA_LENGTH: u32 = 64; // limited to 64 bytes (2 public keys)
pub const MAX_BILL_METADATA_LENGTH: u32 = 50; // limited to 50 bytes for now

#[derive(Clone, Eq, PartialEq, RuntimeDebugNoBound, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct ServiceContract {
    pub service_contract_id: u64,
    pub service_twin_id: u32,
    pub consumer_twin_id: u32,
    pub base_fee: u64,
    pub variable_fee: u64,
    pub metadata: BoundedVec<u8, ConstU32<MAX_METADATA_LENGTH>>,
    pub accepted_by_service: bool,
    pub accepted_by_consumer: bool,
    pub last_bill: u64,
    pub state: ServiceContractState,
}

#[derive(
    PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo, MaxEncodedLen,
)]
pub struct ServiceContractBill {
    pub variable_amount: u64, // variable amount which is billed
    pub window: u64,          // amount of time (in seconds) covered since last bill
    pub metadata: BoundedVec<u8, ConstU32<MAX_BILL_METADATA_LENGTH>>,
}

#[derive(Clone, Eq, PartialEq, RuntimeDebugNoBound, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum ServiceContractState {
    Created,
    AgreementReady,
    ApprovedByBoth,
}

// This code defines a custom SignedExtension `ContractIdProvides`
// SignedExtension is a trait that allows developers to add custom logic to the transaction validation process.
// It ensures that transactions of type bill_contract_for_block are unique per block based on the provided contract ID.
#[derive(Encode, Decode, Clone, Eq, PartialEq, scale_info::TypeInfo)]
pub struct ContractIdProvides<T: Config + Send + Sync + scale_info::TypeInfo>(PhantomData<T>)
where
    <T as frame_system::Config>::RuntimeCall: IsSubType<Call<T>>;

impl<T: Config + Send + Sync + scale_info::TypeInfo> SignedExtension for ContractIdProvides<T>
where
    <T as frame_system::Config>::RuntimeCall: IsSubType<Call<T>>,
{
    const IDENTIFIER: &'static str = "ContractIdProvides";
    type AccountId = T::AccountId;
    type Call = T::RuntimeCall;
    type AdditionalSigned = ();
    type Pre = ();

    fn additional_signed(&self) -> Result<(), TransactionValidityError> {
        Ok(())
    }

    //  Provides the contract ID in a way that prevents duplicate transactions with the same contract ID.
    fn validate(
        &self,
        _who: &Self::AccountId,
        call: &Self::Call,
        _info: &DispatchInfoOf<Self::Call>,
        _len: usize,
    ) -> TransactionValidity {
        if let Some(local_call) = call.is_sub_type() {
            if let Call::bill_contract_for_block { contract_id } = local_call {
                return ValidTransaction::with_tag_prefix(Self::IDENTIFIER)
                    .and_provides(contract_id.to_le_bytes().to_vec())
                    .build()
                    .into();
            }
        }
        Ok(ValidTransaction::default())
    }

    fn pre_dispatch(
        self,
        who: &Self::AccountId,
        call: &Self::Call,
        info: &DispatchInfoOf<Self::Call>,
        len: usize,
    ) -> Result<Self::Pre, TransactionValidityError> {
        self.validate(who, call, info, len).map(|_| ())
    }
}

impl<T: Config + Send + Sync + scale_info::TypeInfo> Debug for ContractIdProvides<T>
where
    <T as frame_system::Config>::RuntimeCall: IsSubType<Call<T>>,
{
    #[cfg(feature = "std")]
    fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        write!(f, "ContractIdProvides")
    }

    #[cfg(not(feature = "std"))]
    fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        Ok(())
    }
}

impl<T: Config + Send + Sync + scale_info::TypeInfo> Default for ContractIdProvides<T>
where
    <T as frame_system::Config>::RuntimeCall: IsSubType<Call<T>>,
{
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T: Config + Send + Sync + scale_info::TypeInfo> ContractIdProvides<T>
where
    <T as frame_system::Config>::RuntimeCall: IsSubType<Call<T>>,
{
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create a default ContractPaymentState
    fn default_contract_payment_state() -> ContractPaymentState<u64> {
        ContractPaymentState {
            last_updated_seconds: 0,
            standard_reserve: 0,
            additional_reserve: 0,
            standard_overdraft: 0,
            additional_overdraft: 0,
            cycles: 0,
        }
    }

    #[test]
    fn test_settle_overdraft() {
        let mut payment_state = default_contract_payment_state();
        payment_state.standard_overdraft = 50;
        payment_state.additional_overdraft = 30;

        payment_state.settle_overdraft();

        assert_eq!(payment_state.standard_overdraft, 0);
        assert_eq!(payment_state.additional_overdraft, 0);
    }

    #[test]
    fn test_reserve_standard_amount() {
        let mut payment_state = default_contract_payment_state();
        payment_state.reserve_standard_amount(100);
        assert_eq!(payment_state.standard_reserve, 100);

        payment_state.reserve_standard_amount(50);
        assert_eq!(payment_state.standard_reserve, 150);
    }

    #[test]
    fn test_reserve_additional_amount() {
        let mut payment_state = default_contract_payment_state();
        payment_state.reserve_additional_amount(200);
        assert_eq!(payment_state.additional_reserve, 200);

        payment_state.reserve_additional_amount(100);
        assert_eq!(payment_state.additional_reserve, 300);
    }

    #[test]
    fn test_overdraft_standard_amount() {
        let mut payment_state = default_contract_payment_state();
        payment_state.overdraft_standard_amount(150);
        assert_eq!(payment_state.standard_overdraft, 150);

        payment_state.overdraft_standard_amount(50);
        assert_eq!(payment_state.standard_overdraft, 200);
    }

    #[test]
    fn test_overdraft_additional_amount() {
        let mut payment_state = default_contract_payment_state();
        payment_state.overdraft_additional_amount(75);
        assert_eq!(payment_state.additional_overdraft, 75);

        payment_state.overdraft_additional_amount(75);
        assert_eq!(payment_state.additional_overdraft, 150);
    }

    #[test]
    fn test_settle_partial_overdraft() {
        let mut payment_state = default_contract_payment_state();
        payment_state.standard_overdraft = 100;
        payment_state.additional_overdraft = 50;

        payment_state.settle_partial_overdraft(30);

        // The remaining overdraft should be reduced by 30
        // Assuming 30 is preferably deducted from additional first
        assert_eq!(payment_state.standard_overdraft, 100);
        assert_eq!(payment_state.additional_overdraft, 20);

        payment_state.settle_partial_overdraft(30);
        assert_eq!(payment_state.standard_overdraft, 90);
        assert_eq!(payment_state.additional_overdraft, 0);

        payment_state.settle_partial_overdraft(100);
        assert_eq!(payment_state.standard_overdraft, 0);
        assert_eq!(payment_state.additional_overdraft, 0);

        payment_state.additional_overdraft = 50;
        payment_state.settle_partial_overdraft(40);
        assert_eq!(payment_state.additional_overdraft, 10);
    }

    #[test]
    fn test_get_overdraft() {
        let mut payment_state = default_contract_payment_state();
        payment_state.standard_overdraft = 200;
        payment_state.additional_overdraft = 100;

        let total_overdraft = payment_state.get_overdraft();

        assert_eq!(total_overdraft, 300);
    }

    #[test]
    fn test_get_reserve() {
        let mut payment_state = default_contract_payment_state();
        payment_state.standard_reserve = 120;
        payment_state.additional_reserve = 80;

        let total_reserved = payment_state.get_reserve();

        assert_eq!(total_reserved, 200);
    }

    #[test]
    fn test_has_reserve() {
        let mut payment_state = default_contract_payment_state();
        assert_eq!(payment_state.has_reserve(), false);

        payment_state.standard_reserve = 120;
        payment_state.additional_reserve = 80;
        assert_eq!(payment_state.has_reserve(), true);

        let mut payment_state = default_contract_payment_state();
        payment_state.standard_reserve = 120;
        assert_eq!(payment_state.has_reserve(), true);

        let mut payment_state = default_contract_payment_state();
        payment_state.additional_reserve = 80;
        assert_eq!(payment_state.has_reserve(), true);
    }

    #[test]
    fn test_has_overdarft() {
        let mut payment_state = default_contract_payment_state();
        assert_eq!(payment_state.has_overdraft(), false);

        payment_state.standard_overdraft = 120;
        payment_state.additional_overdraft = 80;
        assert_eq!(payment_state.has_overdraft(), true);

        let mut payment_state = default_contract_payment_state();
        payment_state.standard_overdraft = 120;
        assert_eq!(payment_state.has_overdraft(), true);

        let mut payment_state = default_contract_payment_state();
        payment_state.additional_overdraft = 80;
        assert_eq!(payment_state.has_overdraft(), true);
    }
}
