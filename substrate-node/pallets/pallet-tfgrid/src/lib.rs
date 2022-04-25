#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod types;

// Version constant that referenced the struct version
pub const TFGRID_ENTITY_VERSION: u32 = 1;
pub const TFGRID_FARM_VERSION: u32 = 2;
pub const TFGRID_TWIN_VERSION: u32 = 1;
pub const TFGRID_NODE_VERSION: u32 = 3;
pub const TFGRID_PRICING_POLICY_VERSION: u32 = 2;
pub const TFGRID_CERTIFICATION_CODE_VERSION: u32 = 1;
pub const TFGRID_FARMING_POLICY_VERSION: u32 = 1;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    pub type TwinIndex = u32;

    pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub type TwinInfoOf<T> = super::types::Twin<AccountIdOf<T>>;

	#[pallet::storage]
	#[pallet::getter(fn twins)]
	pub(super) type Twins<T: Config> = StorageMap
	<	_, 
		Blake2_128Concat, 
		TwinIndex, 
		TwinInfoOf<T>,
		OptionQuery,
	>;

    #[pallet::storage]
    #[pallet::getter(fn twin_ids_by_pubkey)]
    pub(super) type TwinIdByAccountID<T: Config> = StorageMap
    <
        _,
        Blake2_128Concat,
        T::AccountId,
        TwinIndex,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn users_terms_and_condition)]
    pub(super) type UsersTermsAndConditions<T: Config> = StorageMap
    <
        _,
        Blake2_128Concat,
        T::AccountId,
        Vec<super::types::TermsAndConditions<T::AccountId>>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn twin_id)]
    pub(super) type TwinID<T: Config> = StorageValue<_, TwinIndex, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		TwinStored {
            account: T::AccountId,
			ip: Vec<u8>,
		},
        TwinUpdated {
            id: u32,
            account: T::AccountId,
			ip: Vec<u8>,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
        NoneValue,
        StorageOverflow,

        CannotCreateNode,
        NodeNotExists,
        NodeWithTwinIdExists,
        CannotDeleteNode,
        NodeDeleteNotAuthorized,
        NodeUpdateNotAuthorized,

        FarmExists,
        FarmNotExists,
        CannotCreateFarmWrongTwin,
        CannotUpdateFarmWrongTwin,
        CannotDeleteFarm,
        CannotDeleteFarmWithPublicIPs,
        CannotDeleteFarmWithNodesAssigned,
        CannotDeleteFarmWrongTwin,
        IpExists,
        IpNotExists,

        EntityWithNameExists,
        EntityWithPubkeyExists,
        EntityNotExists,
        EntitySignatureDoesNotMatch,
        EntityWithSignatureAlreadyExists,
        CannotUpdateEntity,
        CannotDeleteEntity,
        SignatureLenghtIsIncorrect,

        TwinExists,
        TwinNotExists,
        TwinWithPubkeyExists,
        CannotCreateTwin,
        UnauthorizedToUpdateTwin,

        PricingPolicyExists,
        PricingPolicyNotExists,
        PricingPolicyWithDifferentIdExists,
        CertificationCodeExists,
        FarmingPolicyAlreadyExists,
        FarmPayoutAdressAlreadyRegistered,
        FarmerDoesNotHaveEnoughFunds,
        UserDidNotSignTermsAndConditions,
        FarmerDidNotSignTermsAndConditions,
        FarmerNotAuthorized,
        InvalidFarmName,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {

        // #[weight = <T as Config>::WeightInfo::create_twin()]
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn create_twin(origin: OriginFor<T>, ip: Vec<u8>) -> DispatchResult {
            let account_id = ensure_signed(origin)?;

            ensure!(UsersTermsAndConditions::<T>::contains_key(account_id.clone()), Error::<T>::UserDidNotSignTermsAndConditions);

            ensure!(!<TwinIdByAccountID<T>>::contains_key(&account_id), Error::<T>::TwinWithPubkeyExists);

            let mut twin_id = <TwinID<T>>::get();
            twin_id = twin_id+1;

            let twin = super::types::Twin::<T::AccountId> {
                version: super::TFGRID_TWIN_VERSION,
                id: twin_id,
                account_id: account_id.clone(),
                entities: Vec::new(),
                ip: ip.clone(),
            };

            <Twins<T>>::insert(&twin_id, &twin);
            <TwinID<T>>::put(twin_id);

            // add the twin id to this users map of twin ids
            <TwinIdByAccountID<T>>::insert(&account_id.clone(), twin_id);

            Self::deposit_event(Event::TwinStored { account: account_id, ip: twin.ip });

            Ok(())
        }

        // #[weight = 100_000_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(3)]
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn update_twin(origin: OriginFor<T>, ip: Vec<u8>) -> DispatchResult {
            let account_id = ensure_signed(origin)?;

            let twin_id = <TwinIdByAccountID<T>>::get(account_id.clone()).ok_or(Error::<T>::TwinNotExists)?;
            let mut twin = <Twins<T>>::get(&twin_id).ok_or(Error::<T>::TwinNotExists)?;

            // // Make sure only the owner of this twin can update his twin
            ensure!(twin.account_id == account_id, Error::<T>::UnauthorizedToUpdateTwin);

            // Update the twin
            twin.ip = ip;
            <Twins<T>>::insert(&twin_id, &twin);

            Self::deposit_event(Event::TwinUpdated { id: twin_id, account: account_id, ip: twin.ip });

            Ok(())
        }

        // // Method for twins only
        // // #[weight = 100_000_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(2)]
        // #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        // pub fn add_twin_entity(origin: OriginFor<T>, twin_id: u32, entity_id: u32, signature: Vec<u8>) -> DispatchResult {
        //     let account_id = ensure_signed(origin)?;

        //     ensure!(<Twins<T>>::contains_key(&twin_id), Error::<T>::TwinNotExists);

        //     ensure!(Entities::<T>::contains_key(&entity_id), Error::<T>::EntityNotExists);
        //     let stored_entity = Entities::<T>::get(entity_id);

        //     let mut twin = <Twins<T>>::get(&twin_id);
        //     // Make sure only the owner of this twin can call this method
        //     ensure!(twin.account_id == account_id, Error::<T>::UnauthorizedToUpdateTwin);

        //     let entity_proof = types::EntityProof{
        //         entity_id,
        //         signature: signature.clone()
        //     };

        //     ensure!(!twin.entities.contains(&entity_proof), Error::<T>::EntityWithSignatureAlreadyExists);

        //     let decoded_signature_as_byteslice = <[u8; 64]>::from_hex(signature.clone()).expect("Decoding failed");

        //     let mut message = Vec::new();
        //     message.extend_from_slice(&entity_id.to_be_bytes());
        //     message.extend_from_slice(&twin_id.to_be_bytes());

        //     ensure!(Self::verify_signature(decoded_signature_as_byteslice, &stored_entity.account_id, &message), Error::<T>::EntitySignatureDoesNotMatch);

        //     // Store proof
        //     twin.entities.push(entity_proof);

        //     // Update twin
        //     <Twins<T>>::insert(&twin_id, &twin);

        //     Self::deposit_event(RawEvent::TwinEntityStored(twin_id, entity_id, signature));

        //     Ok(())
        // }

        // // #[weight = 100_000_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(1)]
        // #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        // pub fn delete_twin_entity(origin: OriginFor<T>, twin_id: u32, entity_id: u32) -> DispatchResult {
        //     let account_id = ensure_signed(origin)?;

        //     ensure!(<Twins<T>>::contains_key(&twin_id), Error::<T>::TwinNotExists);

        //     let mut twin = <Twins<T>>::get(&twin_id);
        //     // Make sure only the owner of this twin can call this method
        //     ensure!(twin.account_id == account_id, Error::<T>::UnauthorizedToUpdateTwin);

        //     ensure!(twin.entities.iter().any(|v| v.entity_id == entity_id), Error::<T>::EntityNotExists);

        //     let index = twin.entities.iter().position(|x| x.entity_id == entity_id).unwrap();
        //     twin.entities.remove(index);

        //     // Update twin
        //     <Twins<T>>::insert(&twin_id, &twin);

        //     Self::deposit_event(RawEvent::TwinEntityRemoved(twin_id, entity_id));

        //     Ok(())
        // }

        // // #[weight = 100_000_000 + T::DbWeight::get().writes(2) + T::DbWeight::get().reads(1)]
        // #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        // pub fn delete_twin(origin: OriginFor<T>, twin_id: u32) -> DispatchResult {
        //     let account_id = ensure_signed(origin)?;

        //     ensure!(<Twins<T>>::contains_key(&twin_id), Error::<T>::TwinNotExists);

        //     let twin = <Twins<T>>::get(&twin_id);
        //     // Make sure only the owner of this twin can call this method
        //     ensure!(twin.account_id == account_id, Error::<T>::UnauthorizedToUpdateTwin);

        //     <Twins<T>>::remove(&twin_id);

        //     // remove twin id from this users map of twin ids
        //     <TwinIdByAccountID<T>>::remove(&account_id.clone());

        //     Self::deposit_event(RawEvent::TwinDeleted(twin_id));

        //     Ok(())
        // }

	}
}