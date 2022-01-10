#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

use codec::Encode;
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame
use frame_support::{
    debug, decl_error, decl_event, decl_module, decl_storage, dispatch, ensure,
    traits::Get,
    traits::{Currency, ExistenceRequirement::KeepAlive, EnsureOrigin},
};
use frame_system::{self as system, ensure_signed, RawOrigin};
use hex::FromHex;
use pallet_timestamp as timestamp;
use sp_runtime::traits::SaturatedConversion;
use sp_std::prelude::*;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod mock;

pub mod types;

pub type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance;

pub trait Config: system::Config + timestamp::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type Currency: Currency<Self::AccountId>;
	/// Origin for restricted extrinsics
	/// Can be the root or another origin configured in the runtime
	type RestrictedOrigin: EnsureOrigin<Self::Origin>;
}

// Version constant that referenced the struct version
pub const TFGRID_ENTITY_VERSION: u32 = 1;
pub const TFGRID_FARM_VERSION: u32 = 1;
pub const TFGRID_TWIN_VERSION: u32 = 1;
pub const TFGRID_NODE_VERSION: u32 = 3;
pub const TFGRID_PRICING_POLICY_VERSION: u32 = 1;
pub const TFGRID_CERTIFICATION_CODE_VERSION: u32 = 1;
pub const TFGRID_FARMING_POLICY_VERSION: u32 = 1;

decl_storage! {
    trait Store for Module<T: Config> as TfgridModule {
        pub Farms get(fn farms): map hasher(blake2_128_concat) u32 => types::Farm;
        pub FarmIdByName get(fn farms_by_name_id): map hasher(blake2_128_concat) Vec<u8> => u32;
        pub FarmPayoutV2AddressByFarmID get(fn farm_payout_address_by_farm_id): map hasher(blake2_128_concat) u32 => Vec<u8>;

        pub Nodes get(fn nodes): map hasher(blake2_128_concat) u32 => types::Node;
        pub NodeIdByTwinID get(fn node_by_twin_id): map hasher(blake2_128_concat) u32 => u32;

        pub Entities get(fn entities): map hasher(blake2_128_concat) u32 => types::Entity<T::AccountId>;
        pub EntityIdByAccountID get(fn entities_by_pubkey_id): map hasher(blake2_128_concat) T::AccountId => u32;
        pub EntityIdByName get(fn entities_by_name_id): map hasher(blake2_128_concat) Vec<u8> => u32;

        pub Twins get(fn twins): map hasher(blake2_128_concat) u32 => types::Twin<T::AccountId>;
        pub TwinIdByAccountID get(fn twin_ids_by_pubkey): map hasher(blake2_128_concat) T::AccountId => u32;

        pub PricingPolicies get(fn pricing_policies): map hasher(blake2_128_concat) u32 => types::PricingPolicy<T::AccountId>;
        pub PricingPolicyIdByName get(fn pricing_policies_by_name_id): map hasher(blake2_128_concat) Vec<u8> => u32;

        pub CertificationCodes get(fn certification_codes): map hasher(blake2_128_concat) u32 => types::CertificationCodes;
        pub CertificationCodeIdByName get(fn certification_codes_by_name_id): map hasher(blake2_128_concat) Vec<u8> => u32;

        pub FarmingPolicies get(fn farming_policies): Vec<types::FarmingPolicy>;
        pub FarmingPolicyIDsByCertificationType get (fn farming_policies_by_certification_type): map hasher(blake2_128_concat) types::CertificationType => Vec<u32>;

        pub UsersTermsAndConditions get(fn users_terms_and_condition): map hasher(blake2_128_concat) T::AccountId => Vec<types::TermsAndConditions<T::AccountId>>;
        pub FarmersTermsAndConditions get(fn farmers_terms_and_condition): map hasher(blake2_128_concat) T::AccountId => Vec<types::TermsAndConditions<T::AccountId>>;

        // ID maps
        FarmID: u32;
        NodeID: u32;
        EntityID: u32;
        TwinID: u32;
        PricingPolicyID: u32;
        CertificationCodeID: u32;
        FarmingPolicyID: u32;

        /// The current version of the pallet.
        PalletVersion: types::StorageVersion = types::StorageVersion::V1Struct;
    }

    add_extra_genesis {
        config(su_price_value): u32;
        config(su_price_unit): u32;
        config(nu_price_value): u32;
        config(nu_price_unit): u32;
        config(ipu_price_value): u32;
        config(ipu_price_unit): u32;
        config(cu_price_value): u32;
        config(cu_price_unit): u32;
        config(unique_name_price_value): u32;
        config(domain_name_price_value): u32;
        config(foundation_account): T::AccountId;
        config(sales_account): T::AccountId;

        config(farming_policy_diy_cu): u32;
        config(farming_policy_diy_nu): u32;
        config(farming_policy_diy_su): u32;
        config(farming_policy_diy_ipu): u32;

        config(farming_policy_certified_cu): u32;
        config(farming_policy_certified_nu): u32;
        config(farming_policy_certified_su): u32;
        config(farming_policy_certified_ipu): u32;

        build(|_config| {
            let foundation_account = _config.foundation_account.clone();
            let sales_account = _config.sales_account.clone();

            let su_price = types::Policy{
                value: _config.su_price_value,
                unit: types::Unit::from_u32(_config.su_price_unit),
            };

            let cu_price = types::Policy{
                value: _config.cu_price_value,
                unit: types::Unit::from_u32(_config.cu_price_unit),
            };

            let nu_price = types::Policy{
                value: _config.nu_price_value,
                unit: types::Unit::from_u32(_config.nu_price_unit),
            };

            let ipu_price = types::Policy{
                value: _config.ipu_price_value,
                unit: types::Unit::from_u32(_config.ipu_price_unit),
            };

            let unique_name_price = types::Policy{
                value: _config.unique_name_price_value,
                unit: types::Unit::default(),
            };

            let domain_name_price = types::Policy{
                value: _config.domain_name_price_value,
                unit: types::Unit::default(),
            };

            let _ = <Module<T>>::create_pricing_policy(
                RawOrigin::Root.into(),
                "threefold_default_pricing_policy".as_bytes().to_vec(),
                su_price,
                cu_price,
                nu_price,
                ipu_price,
                unique_name_price,
                domain_name_price,
                foundation_account,
                sales_account
            );

            let _ = <Module<T>>::create_farming_policy(
                RawOrigin::Root.into(),
                "threefold_default_diy_farming_policy".as_bytes().to_vec(),
                _config.farming_policy_diy_su,
                _config.farming_policy_diy_cu,
                _config.farming_policy_diy_nu,
                _config.farming_policy_diy_ipu,
                types::CertificationType::Diy,
            );

            let _ = <Module<T>>::create_farming_policy(
                RawOrigin::Root.into(),
                "threefold_default_certified_farming_policy".as_bytes().to_vec(),
                _config.farming_policy_certified_su,
                _config.farming_policy_certified_cu,
                _config.farming_policy_certified_nu,
                _config.farming_policy_certified_ipu,
                types::CertificationType::Certified,
            );

        });
    }

}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Config>::AccountId,
    {
        FarmStored(types::Farm),
        FarmUpdated(types::Farm),
        FarmDeleted(u32),

        NodeStored(types::Node),
        NodeUpdated(types::Node),
        NodeDeleted(u32),
        NodeUptimeReported(u32, u64, u64),
        NodePublicConfigStored(u32, types::PublicConfig),

        EntityStored(types::Entity<AccountId>),
        EntityUpdated(types::Entity<AccountId>),
        EntityDeleted(u32),

        TwinStored(types::Twin<AccountId>),
        TwinUpdated(types::Twin<AccountId>),

        TwinEntityStored(u32, u32, Vec<u8>),
        TwinEntityRemoved(u32, u32),
        TwinDeleted(u32),

        PricingPolicyStored(types::PricingPolicy<AccountId>),
        CertificationCodeStored(types::CertificationCodes),
        FarmingPolicyStored(types::FarmingPolicy),
        FarmPayoutV2AddressRegistered(u32, Vec<u8>),
    }
);

decl_error! {
    pub enum Error for Module<T: Config> {
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
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        #[weight = 10 + T::DbWeight::get().writes(1)]
        pub fn user_accept_tc(origin, document_link: Vec<u8>, document_hash: Vec<u8>) -> dispatch::DispatchResult {
            let account_id = ensure_signed(origin)?;
            
            let timestamp = <timestamp::Module<T>>::get().saturated_into::<u64>() / 1000;

            let t_and_c = types::TermsAndConditions {
                account_id: account_id.clone(),
                timestamp,
                document_link,
                document_hash
            };

            let mut users_terms_and_condition = UsersTermsAndConditions::<T>::get(account_id.clone());
            users_terms_and_condition.push(t_and_c);
            UsersTermsAndConditions::<T>::insert(account_id, users_terms_and_condition);

            Ok(())
        }

        #[weight = 10 + T::DbWeight::get().writes(1)]
        pub fn farmer_accept_tc(origin, document_link: Vec<u8>, document_hash: Vec<u8>) -> dispatch::DispatchResult {
            let account_id = ensure_signed(origin)?;
            
            let timestamp = <timestamp::Module<T>>::get().saturated_into::<u64>() / 1000;

            let t_and_c = types::TermsAndConditions {
                account_id: account_id.clone(),
                timestamp,
                document_link,
                document_hash
            };

            let mut users_terms_and_condition = FarmersTermsAndConditions::<T>::get(account_id.clone());
            users_terms_and_condition.push(t_and_c);
            FarmersTermsAndConditions::<T>::insert(account_id, users_terms_and_condition);

            Ok(())
        }

        #[weight = 10 + T::DbWeight::get().writes(1)]
        pub fn create_farm(origin, name: Vec<u8>, public_ips: Vec<types::PublicIP>) -> dispatch::DispatchResult {
            let address = ensure_signed(origin)?;

            ensure!(FarmersTermsAndConditions::<T>::contains_key(address.clone()), Error::<T>::FarmerDidNotSignTermsAndConditions);

            ensure!(!FarmIdByName::contains_key(name.clone()), Error::<T>::FarmExists);
            ensure!(TwinIdByAccountID::<T>::contains_key(&address), Error::<T>::TwinNotExists);
            let twin_id = TwinIdByAccountID::<T>::get(&address);
            let twin = Twins::<T>::get(twin_id);
            ensure!(twin.account_id == address, Error::<T>::CannotCreateFarmWrongTwin);

            let mut id = FarmID::get();
            id = id+1;

            // reset all public ip contract id's
            // just a safeguard
            // filter out doubles
            let mut pub_ips: Vec<types::PublicIP> = Vec::new();
            for ip in public_ips {
                match pub_ips.iter().position(|pub_ip| pub_ip.ip == ip.ip) {
                    Some(_) => return Err(Error::<T>::IpExists.into()),
                    None => {
                        pub_ips.push(types::PublicIP{
                            ip: ip.ip,
                            gateway: ip.gateway,
                            contract_id: 0
                        });
                    }
                };
            };

            let new_farm = types::Farm {
                version: TFGRID_FARM_VERSION,
                id,
                twin_id,
                name,
                pricing_policy_id: 1,
                certification_type: types::CertificationType::Diy,
                public_ips: pub_ips,
            };

            Farms::insert(id, &new_farm);
            FarmIdByName::insert(new_farm.name.clone(), id);
            FarmID::put(id);

            Self::deposit_event(RawEvent::FarmStored(new_farm));

            Ok(())
        }

        #[weight = 10 + T::DbWeight::get().writes(1)]
        pub fn update_farm(origin, id: u32, name: Vec<u8>, pricing_policy_id: u32) -> dispatch::DispatchResult {
            let address = ensure_signed(origin)?;

            ensure!(TwinIdByAccountID::<T>::contains_key(&address), Error::<T>::TwinNotExists);
            let twin_id = TwinIdByAccountID::<T>::get(&address);

            ensure!(Farms::contains_key(id), Error::<T>::FarmNotExists);
            let farm = Farms::get(id);

            ensure!(farm.twin_id == twin_id, Error::<T>::CannotUpdateFarmWrongTwin);

            let mut stored_farm = Farms::get(id);
            // Remove stored farm by name and insert new one
            FarmIdByName::remove(stored_farm.name);

            stored_farm.name = name.clone();
            stored_farm.pricing_policy_id = pricing_policy_id;

            Farms::insert(id, &stored_farm);
            FarmIdByName::insert(name, stored_farm.id);

            Self::deposit_event(RawEvent::FarmUpdated(stored_farm));

            Ok(())
        }

        #[weight = 10 + T::DbWeight::get().writes(1)]
        pub fn add_stellar_payout_v2address(origin, farm_id: u32, stellar_address: Vec<u8>) -> dispatch::DispatchResult {
            let address = ensure_signed(origin)?;

            ensure!(TwinIdByAccountID::<T>::contains_key(&address), Error::<T>::TwinNotExists);
            let twin_id = TwinIdByAccountID::<T>::get(&address);

            ensure!(Farms::contains_key(farm_id), Error::<T>::FarmNotExists);
            let farm = Farms::get(farm_id);

            ensure!(farm.twin_id == twin_id, Error::<T>::CannotUpdateFarmWrongTwin);

            FarmPayoutV2AddressByFarmID::insert(&farm_id, &stellar_address);

            Self::deposit_event(RawEvent::FarmPayoutV2AddressRegistered(farm_id, stellar_address));

            Ok(())
        }

        #[weight = 10 + T::DbWeight::get().writes(1)]
        pub fn set_farm_certification(origin, farm_id: u32, certification_type: types::CertificationType) -> dispatch::DispatchResult {
            T::RestrictedOrigin::ensure_origin(origin)?;

            ensure!(Farms::contains_key(farm_id), Error::<T>::FarmNotExists);
            let mut stored_farm = Farms::get(farm_id);

            stored_farm.certification_type = certification_type;

            Farms::insert(farm_id, &stored_farm);

            Ok(())
        }

        #[weight = 10 + T::DbWeight::get().writes(1)]
        pub fn add_farm_ip(origin, id: u32, ip: Vec<u8>, gateway: Vec<u8>) -> dispatch::DispatchResult {
            let address = ensure_signed(origin)?;

            ensure!(Farms::contains_key(id), Error::<T>::FarmNotExists);
            let mut stored_farm = Farms::get(id);

            let twin = Twins::<T>::get(stored_farm.twin_id);
            ensure!(twin.account_id == address, Error::<T>::CannotUpdateFarmWrongTwin);

            let new_ip = types::PublicIP {
                ip,
                gateway,
                contract_id: 0
            };

            match stored_farm.public_ips.iter().position(|public_ip| public_ip.ip == new_ip.ip) {
                Some(_) => return Err(Error::<T>::IpExists.into()),
                None => {
                    stored_farm.public_ips.push(new_ip);
                    Farms::insert(stored_farm.id, &stored_farm);
                    Self::deposit_event(RawEvent::FarmUpdated(stored_farm));
                    return Ok(())
                }
            };
        }

        #[weight = 10 + T::DbWeight::get().writes(1)]
        pub fn remove_farm_ip(origin, id: u32, ip: Vec<u8>) -> dispatch::DispatchResult {
            let address = ensure_signed(origin)?;

            ensure!(Farms::contains_key(id), Error::<T>::FarmNotExists);
            let mut stored_farm = Farms::get(id);

            let twin = Twins::<T>::get(stored_farm.twin_id);
            ensure!(twin.account_id == address, Error::<T>::CannotUpdateFarmWrongTwin);

            match stored_farm.public_ips.iter().position(|pubip| pubip.ip == ip && pubip.contract_id == 0) {
                Some(index) => {
                    stored_farm.public_ips.remove(index);
                    Farms::insert(stored_farm.id, &stored_farm);
                    Self::deposit_event(RawEvent::FarmUpdated(stored_farm));
                    Ok(())
                },
                None => Err(Error::<T>::IpNotExists.into()),
            }
        }

        #[weight = 10 + T::DbWeight::get().writes(1)]
        pub fn delete_farm(origin, id: u32) -> dispatch::DispatchResult {
            let address = ensure_signed(origin)?;

            ensure!(Farms::contains_key(id), Error::<T>::FarmNotExists);
            let stored_farm = Farms::get(id);
            // make sure farm doesn't have public ips assigned
            ensure!(stored_farm.public_ips.len() == 0, Error::<T>::CannotDeleteFarmWithPublicIPs);
            // make sure farm doesn't have nodes assigned
            for (_, node) in Nodes::iter(){
                if node.farm_id == id {
                    return Err(Error::<T>::CannotDeleteFarmWithNodesAssigned.into())
                }
            }
            let twin = Twins::<T>::get(stored_farm.twin_id);
            ensure!(twin.account_id == address, Error::<T>::CannotDeleteFarmWrongTwin);

            // delete farm
            Farms::remove(id);

            // Remove stored farm by name and insert new one
            FarmIdByName::remove(stored_farm.name);

            Self::deposit_event(RawEvent::FarmDeleted(id));

            Ok(())
        }

        #[weight = 10 + T::DbWeight::get().writes(1)]
        pub fn create_node(origin, farm_id: u32, resources: types::Resources, location: types::Location, country: Vec<u8>, city: Vec<u8>, interfaces: Vec<types::Interface>) -> dispatch::DispatchResult {
            let account_id = ensure_signed(origin)?;

            ensure!(Farms::contains_key(farm_id), Error::<T>::FarmNotExists);
            let farm = Farms::get(farm_id);
            ensure!(TwinIdByAccountID::<T>::contains_key(&account_id), Error::<T>::TwinNotExists);
            let twin_id = TwinIdByAccountID::<T>::get(&account_id);

            ensure!(!NodeIdByTwinID::contains_key(twin_id), Error::<T>::NodeWithTwinIdExists);

            let mut id = NodeID::get();
            id = id+1;

            // Attach a farming policy to a node
            // We first filter on Policies by certification type of the farm
            // If there are policies set by us, attach the last on in the list
            // This list is updated with new policies when we change the farming rules, so we want new nodes
            // to always use the latest farming policy (last one in the list)
            let farming_policies = FarmingPolicyIDsByCertificationType::get(farm.certification_type);
            let mut farming_policy_id = 0;
            if farming_policies.len() > 0 {
                farming_policy_id = farming_policies[farming_policies.len() -1];
            }

            let created = <timestamp::Module<T>>::get().saturated_into::<u64>() / 1000;

            let new_node = types::Node {
                version: TFGRID_NODE_VERSION,
                id,
                farm_id,
                twin_id,
                resources,
                location,
                country,
                city,
                public_config: None,
                created,
                farming_policy_id,
                interfaces,
                certification_type: types::CertificationType::default(),
            };

            Nodes::insert(id, &new_node);
            NodeID::put(id);
            NodeIdByTwinID::insert(twin_id, new_node.id);

            Self::deposit_event(RawEvent::NodeStored(new_node));

            Ok(())
        }

        #[weight = 10 + T::DbWeight::get().writes(1)]
        pub fn update_node(origin, node_id: u32, farm_id: u32, resources: types::Resources, location: types::Location, country: Vec<u8>, city: Vec<u8>, interfaces: Vec<types::Interface>) -> dispatch::DispatchResult {
            let account_id = ensure_signed(origin)?;

            ensure!(Nodes::contains_key(&node_id), Error::<T>::NodeNotExists);
            ensure!(TwinIdByAccountID::<T>::contains_key(&account_id), Error::<T>::TwinNotExists);

            let twin_id = TwinIdByAccountID::<T>::get(&account_id);
            let node = Nodes::get(&node_id);
            ensure!(node.twin_id == twin_id, Error::<T>::NodeUpdateNotAuthorized);

            ensure!(Farms::contains_key(farm_id), Error::<T>::FarmNotExists);

            let mut stored_node = Nodes::get(node_id);

            stored_node.farm_id = farm_id;
            stored_node.resources = resources;
            stored_node.location = location;
            stored_node.country = country;
            stored_node.city = city;
            stored_node.interfaces = interfaces;

            // override node in storage
            Nodes::insert(stored_node.id, &stored_node);

            Self::deposit_event(RawEvent::NodeUpdated(stored_node));

            // refund node wallet if needed
            Self::fund_node_wallet(node_id);

            Ok(())
        }

        #[weight = 10 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(4)]
        pub fn delete_node_farm(origin, node_id: u32) -> dispatch::DispatchResult {
            let account_id = ensure_signed(origin)?;

            ensure!(TwinIdByAccountID::<T>::contains_key(&account_id), Error::<T>::TwinNotExists);
            ensure!(Nodes::contains_key(&node_id), Error::<T>::NodeNotExists);

            // check if the farmer twin is authorized
            let farm_twin_id = TwinIdByAccountID::<T>::get(&account_id);
            // check if the ndode belong to said farm
            let node = Nodes::get(&node_id);
            let farm = Farms::get(node.farm_id);
            let farm_twin = Twins::<T>::get(farm.twin_id);
            ensure!(farm_twin_id == farm_twin.id, Error::<T>::FarmerNotAuthorized);


            Nodes::remove(node_id);

            Ok(())
        }

        #[weight = 10 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(1)]
        pub fn set_node_certification(origin, node_id: u32, certification_type: types::CertificationType) -> dispatch::DispatchResult {
            T::RestrictedOrigin::ensure_origin(origin)?;

            ensure!(Nodes::contains_key(&node_id), Error::<T>::NodeNotExists);
            let mut stored_node = Nodes::get(node_id);

            stored_node.certification_type = certification_type;

            // override node in storage
            Nodes::insert(stored_node.id, &stored_node);

            Self::deposit_event(RawEvent::NodeUpdated(stored_node));

            Ok(())
        }

        #[weight = 10 + T::DbWeight::get().writes(1)]
        pub fn report_uptime(origin, uptime: u64) -> dispatch::DispatchResult {
            let account_id = ensure_signed(origin)?;

            ensure!(TwinIdByAccountID::<T>::contains_key(&account_id), Error::<T>::TwinNotExists);
            let twin_id = TwinIdByAccountID::<T>::get(account_id);

            ensure!(NodeIdByTwinID::contains_key(twin_id), Error::<T>::TwinNotExists);
            let node_id = NodeIdByTwinID::get(twin_id);

            ensure!(Nodes::contains_key(node_id), Error::<T>::NodeNotExists);

            let now = <timestamp::Module<T>>::get().saturated_into::<u64>() / 1000;

            Self::deposit_event(RawEvent::NodeUptimeReported(node_id, now, uptime));

            // refund node wallet if needed
            Self::fund_node_wallet(node_id);

            Ok(())
        }

        #[weight = 10 + T::DbWeight::get().writes(1)]
        pub fn add_node_public_config(origin, farm_id: u32, node_id: u32, public_config: types::PublicConfig) -> dispatch::DispatchResult {
            let account_id = ensure_signed(origin)?;

            // check if this twin can update the farm with id passed
            ensure!(Farms::contains_key(farm_id), Error::<T>::FarmNotExists);
            let farm = Farms::get(farm_id);
            let farm_twin = Twins::<T>::get(farm.twin_id);
            ensure!(farm_twin.account_id == account_id, Error::<T>::CannotUpdateFarmWrongTwin);

            // check if the node belong to the farm
            ensure!(Nodes::contains_key(node_id), Error::<T>::NodeNotExists);
            let mut node = Nodes::get(node_id);
            ensure!(node.farm_id == farm_id, Error::<T>::NodeUpdateNotAuthorized);

            // update the public config and save
            node.public_config = Some(public_config.clone());
            Nodes::insert(node_id, node);

            Self::deposit_event(RawEvent::NodePublicConfigStored(node_id, public_config));

            // refund node wallet if needed
            Self::fund_node_wallet(node_id);

            Ok(())
        }

        #[weight = 10 + T::DbWeight::get().writes(1)]
        pub fn delete_node(origin, id: u32) -> dispatch::DispatchResult {
            let account_id = ensure_signed(origin)?;

            ensure!(Nodes::contains_key(id), Error::<T>::NodeNotExists);

            let stored_node = Nodes::get(id);
            let twin_id = TwinIdByAccountID::<T>::get(&account_id);
            ensure!(stored_node.twin_id == twin_id, Error::<T>::NodeUpdateNotAuthorized);

            Nodes::remove(id);

            Self::deposit_event(RawEvent::NodeDeleted(id));

            Ok(())
        }

        #[weight = 10 + T::DbWeight::get().writes(1)]
        pub fn create_entity(origin, target: T::AccountId, name: Vec<u8>, country: Vec<u8>, city: Vec<u8>, signature: Vec<u8>) -> dispatch::DispatchResult {
            let _ = ensure_signed(origin)?;

            ensure!(!EntityIdByName::contains_key(&name), Error::<T>::EntityWithNameExists);
            ensure!(!EntityIdByAccountID::<T>::contains_key(&target), Error::<T>::EntityWithPubkeyExists);

            let entity_pubkey_ed25519 = Self::convert_account_to_ed25519(target.clone());

            ensure!(signature.len() == 128, Error::<T>::SignatureLenghtIsIncorrect);
            let decoded_signature_as_byteslice = <[u8; 64]>::from_hex(signature.clone()).expect("Decoding failed");

            // Decode signature into a ed25519 signature
            let ed25519_signature = sp_core::ed25519::Signature::from_raw(decoded_signature_as_byteslice);

            let mut message = Vec::new();
            message.extend_from_slice(&name);
            message.extend_from_slice(&country);
            message.extend_from_slice(&city);

            ensure!(sp_io::crypto::ed25519_verify(&ed25519_signature, &message, &entity_pubkey_ed25519), Error::<T>::EntitySignatureDoesNotMatch);

            let mut id = EntityID::get();
            id = id+1;

            let entity = types::Entity::<T::AccountId> {
                version: TFGRID_ENTITY_VERSION,
                id,
                name: name.clone(),
                country,
                city,
                account_id: target.clone(),
            };

            Entities::<T>::insert(&id, &entity);
            EntityIdByName::insert(&name, id);
            EntityIdByAccountID::<T>::insert(&target, id);
            EntityID::put(id);

            Self::deposit_event(RawEvent::EntityStored(entity));

            Ok(())
        }

        #[weight = 10 + T::DbWeight::get().writes(1)]
        pub fn update_entity(origin, name: Vec<u8>, country: Vec<u8>, city: Vec<u8>) -> dispatch::DispatchResult {
            let account_id = ensure_signed(origin)?;

            ensure!(!EntityIdByName::contains_key(&name), Error::<T>::EntityWithNameExists);

            ensure!(EntityIdByAccountID::<T>::contains_key(&account_id), Error::<T>::EntityNotExists);
            let stored_entity_id = EntityIdByAccountID::<T>::get(&account_id);

            ensure!(Entities::<T>::contains_key(&stored_entity_id), Error::<T>::EntityNotExists);
            let mut stored_entity = Entities::<T>::get(stored_entity_id);

            ensure!(stored_entity.account_id == account_id, Error::<T>::CannotUpdateEntity);

            // remove entity by name id
            EntityIdByName::remove(&stored_entity.name);

            stored_entity.name = name.clone();
            stored_entity.country = country;
            stored_entity.city = city;

            // overwrite entity
            Entities::<T>::insert(&stored_entity_id, &stored_entity);

            // re-insert with new name
            EntityIdByName::insert(&name, stored_entity_id);

            Self::deposit_event(RawEvent::EntityUpdated(stored_entity));

            Ok(())
        }

        // TODO: delete all object that have an entity id reference?
        #[weight = 10 + T::DbWeight::get().writes(1)]
        pub fn delete_entity(origin) -> dispatch::DispatchResult {
            let account_id = ensure_signed(origin)?;

            ensure!(EntityIdByAccountID::<T>::contains_key(&account_id), Error::<T>::EntityNotExists);
            let stored_entity_id = EntityIdByAccountID::<T>::get(&account_id);

            ensure!(Entities::<T>::contains_key(&stored_entity_id), Error::<T>::EntityNotExists);
            let stored_entity = Entities::<T>::get(stored_entity_id);

            ensure!(stored_entity.account_id == account_id, Error::<T>::CannotDeleteEntity);

            // Remove entity from storage
            Entities::<T>::remove(&stored_entity_id);

            // remove entity by name id
            EntityIdByName::remove(&stored_entity.name);

            // remove entity by pubkey id
            EntityIdByAccountID::<T>::remove(&account_id);

            Self::deposit_event(RawEvent::EntityDeleted(stored_entity_id));

            Ok(())
        }

        #[weight = 10 + T::DbWeight::get().writes(1)]
        pub fn create_twin(origin, ip: Vec<u8>) -> dispatch::DispatchResult {
            let account_id = ensure_signed(origin)?;

            ensure!(UsersTermsAndConditions::<T>::contains_key(account_id.clone()), Error::<T>::UserDidNotSignTermsAndConditions);

            ensure!(!TwinIdByAccountID::<T>::contains_key(&account_id), Error::<T>::TwinWithPubkeyExists);

            let mut twin_id = TwinID::get();
            twin_id = twin_id+1;

            let twin = types::Twin::<T::AccountId> {
                version: TFGRID_TWIN_VERSION,
                id: twin_id,
                account_id: account_id.clone(),
                entities: Vec::new(),
                ip: ip.clone(),
            };

            Twins::<T>::insert(&twin_id, &twin);
            TwinID::put(twin_id);

            // add the twin id to this users map of twin ids
            TwinIdByAccountID::<T>::insert(&account_id.clone(), twin_id);

            Self::deposit_event(RawEvent::TwinStored(twin));

            Ok(())
        }

        #[weight = 10 + T::DbWeight::get().writes(1)]
        pub fn update_twin(origin, ip: Vec<u8>) -> dispatch::DispatchResult {
            let account_id = ensure_signed(origin)?;

            ensure!(TwinIdByAccountID::<T>::contains_key(account_id.clone()), Error::<T>::TwinNotExists);
            let twin_id = TwinIdByAccountID::<T>::get(account_id.clone());
            let mut twin = Twins::<T>::get(&twin_id);

            // Make sure only the owner of this twin can update his twin
            ensure!(twin.account_id == account_id, Error::<T>::UnauthorizedToUpdateTwin);

            twin.ip = ip.clone();

            Twins::<T>::insert(&twin_id, &twin);

            Self::deposit_event(RawEvent::TwinUpdated(twin));
            Ok(())
        }

        // Method for twins only
        #[weight = 10 + T::DbWeight::get().writes(1)]
        pub fn add_twin_entity(origin, twin_id: u32, entity_id: u32, signature: Vec<u8>) -> dispatch::DispatchResult {
            let account_id = ensure_signed(origin)?;

            ensure!(Twins::<T>::contains_key(&twin_id), Error::<T>::TwinNotExists);

            ensure!(Entities::<T>::contains_key(&entity_id), Error::<T>::EntityNotExists);
            let stored_entity = Entities::<T>::get(entity_id);

            let mut twin = Twins::<T>::get(&twin_id);
            // Make sure only the owner of this twin can call this method
            ensure!(twin.account_id == account_id, Error::<T>::UnauthorizedToUpdateTwin);

            let entity_proof = types::EntityProof{
                entity_id,
                signature: signature.clone()
            };

            ensure!(!twin.entities.contains(&entity_proof), Error::<T>::EntityWithSignatureAlreadyExists);

            let decoded_signature_as_byteslice = <[u8; 64]>::from_hex(signature.clone()).expect("Decoding failed");

            // Decode signature into a ed25519 signature
            let ed25519_signature = sp_core::ed25519::Signature::from_raw(decoded_signature_as_byteslice);

            let entity_pubkey_ed25519 = Self::convert_account_to_ed25519(stored_entity.account_id.clone());

            let mut message = Vec::new();

            message.extend_from_slice(&entity_id.to_be_bytes());
            message.extend_from_slice(&twin_id.to_be_bytes());

            ensure!(sp_io::crypto::ed25519_verify(&ed25519_signature, &message, &entity_pubkey_ed25519), Error::<T>::EntitySignatureDoesNotMatch);

            // Store proof
            twin.entities.push(entity_proof);

            // Update twin
            Twins::<T>::insert(&twin_id, &twin);

            Self::deposit_event(RawEvent::TwinEntityStored(twin_id, entity_id, signature));

            Ok(())
        }

        #[weight = 10 + T::DbWeight::get().writes(1)]
        pub fn delete_twin_entity(origin, twin_id: u32, entity_id: u32) -> dispatch::DispatchResult {
            let account_id = ensure_signed(origin)?;

            ensure!(Twins::<T>::contains_key(&twin_id), Error::<T>::TwinNotExists);

            let mut twin = Twins::<T>::get(&twin_id);
            // Make sure only the owner of this twin can call this method
            ensure!(twin.account_id == account_id, Error::<T>::UnauthorizedToUpdateTwin);

            ensure!(twin.entities.iter().any(|v| v.entity_id == entity_id), Error::<T>::EntityNotExists);

            let index = twin.entities.iter().position(|x| x.entity_id == entity_id).unwrap();
            twin.entities.remove(index);

            // Update twin
            Twins::<T>::insert(&twin_id, &twin);

            Self::deposit_event(RawEvent::TwinEntityRemoved(twin_id, entity_id));

            Ok(())
        }

        #[weight = 10 + T::DbWeight::get().writes(1)]
        pub fn delete_twin(origin, twin_id: u32) -> dispatch::DispatchResult {
            let account_id = ensure_signed(origin)?;

            ensure!(Twins::<T>::contains_key(&twin_id), Error::<T>::TwinNotExists);

            let twin = Twins::<T>::get(&twin_id);
            // Make sure only the owner of this twin can call this method
            ensure!(twin.account_id == account_id, Error::<T>::UnauthorizedToUpdateTwin);

            Twins::<T>::remove(&twin_id);

            // remove twin id from this users map of twin ids
            TwinIdByAccountID::<T>::remove(&account_id.clone());

            Self::deposit_event(RawEvent::TwinDeleted(twin_id));

            Ok(())
        }

        #[weight = 10 + T::DbWeight::get().writes(1)]
        pub fn create_pricing_policy(
            origin,
            name: Vec<u8>,
            su: types::Policy,
            cu: types::Policy,
            nu: types::Policy,
            ipu: types::Policy,
            unique_name: types::Policy,
            domain_name: types::Policy,
            foundation_account: T::AccountId,
            certified_sales_account: T::AccountId
        ) -> dispatch::DispatchResult {
            T::RestrictedOrigin::ensure_origin(origin)?;

            ensure!(!PricingPolicyIdByName::contains_key(&name), Error::<T>::PricingPolicyExists);

            let mut id = PricingPolicyID::get();
            id = id+1;


            let new_policy = types::PricingPolicy {
                version: TFGRID_PRICING_POLICY_VERSION,
                id,
                name,
                su,
                cu,
                nu,
                ipu,
                unique_name,
                domain_name,
                foundation_account,
                certified_sales_account,
            };

            PricingPolicies::<T>::insert(&id, &new_policy);
            PricingPolicyIdByName::insert(&new_policy.name, &id);
            PricingPolicyID::put(id);

            Self::deposit_event(RawEvent::PricingPolicyStored(new_policy));
            Ok(())
        }

        #[weight = 10 + T::DbWeight::get().writes(1)]
        pub fn update_pricing_policy(
            origin,
            id: u32,
            name: Vec<u8>,
            su: types::Policy,
            cu: types::Policy,
            nu: types::Policy,
            ipu: types::Policy,
            unique_name: types::Policy,
            domain_name: types::Policy,
            foundation_account: T::AccountId,
            certified_sales_account: T::AccountId
        ) -> dispatch::DispatchResult {
            T::RestrictedOrigin::ensure_origin(origin)?;

            // Ensure pricing policy with same id already exists
            ensure!(PricingPolicies::<T>::contains_key(&id), Error::<T>::PricingPolicyNotExists);

            // if name exists ensure that it belongs to the same policy id
            if PricingPolicyIdByName::contains_key(&name){
                let stored_id = PricingPolicyIdByName::get(&name);
                ensure!(stored_id==id, Error::<T>::PricingPolicyWithDifferentIdExists);
            }
            let mut pricing_policy = PricingPolicies::<T>::get(id);

            if name != pricing_policy.name {
                PricingPolicyIdByName::remove(&pricing_policy.name);
            }

            pricing_policy.name = name;
            pricing_policy.su = su;
            pricing_policy.cu = cu;
            pricing_policy.nu = nu;
            pricing_policy.ipu = ipu;
            pricing_policy.unique_name = unique_name;
            pricing_policy.domain_name = domain_name;
            pricing_policy.foundation_account = foundation_account;
            pricing_policy.certified_sales_account = certified_sales_account;

            PricingPolicies::<T>::insert(&id, &pricing_policy);
            PricingPolicyIdByName::insert(&pricing_policy.name, &id);
            PricingPolicyID::put(id);

            Self::deposit_event(RawEvent::PricingPolicyStored(pricing_policy));

            Ok(())
        }

        #[weight = 10 + T::DbWeight::get().writes(1)]
        pub fn create_certification_code(origin, name: Vec<u8>, description: Vec<u8>, certification_code_type: types::CertificationCodeType) -> dispatch::DispatchResult {
            T::RestrictedOrigin::ensure_origin(origin)?;

            ensure!(!CertificationCodeIdByName::contains_key(&name), Error::<T>::CertificationCodeExists);

            let mut id = CertificationCodeID::get();
            id = id+1;

            let certification_code = types::CertificationCodes{
                version: TFGRID_CERTIFICATION_CODE_VERSION,
                id,
                name: name.clone(),
                description,
                certification_code_type
            };

            CertificationCodes::insert(&id, &certification_code);
            CertificationCodeIdByName::insert(&name, &id);
            CertificationCodeID::put(id);

            Self::deposit_event(RawEvent::CertificationCodeStored(certification_code));

            Ok(())
        }

        #[weight = 10 + T::DbWeight::get().writes(1)]
        pub fn create_farming_policy(origin, name: Vec<u8>, su: u32, cu: u32, nu: u32, ipv4: u32, certification_type: types::CertificationType) -> dispatch::DispatchResult {
            T::RestrictedOrigin::ensure_origin(origin)?;

            let mut id = FarmingPolicyID::get();
            id = id+1;

            let mut farming_policies = FarmingPolicies::get();

            let now = <timestamp::Module<T>>::get().saturated_into::<u64>() / 1000;

            let new_policy = types::FarmingPolicy {
                version: TFGRID_FARMING_POLICY_VERSION,
                id,
                name,
                su,
                cu,
                nu,
                ipv4,
                timestamp: now,
                certification_type,
            };


            // We don't want to add duplicate farming_policies, so we check whether it exists, if so return error
            match farming_policies.binary_search(&new_policy) {
                Ok(_) => Err(Error::<T>::FarmingPolicyAlreadyExists.into()),
                Err(index) => {
                    // Object does not exists, save it
                    farming_policies.insert(index, new_policy.clone());
                    FarmingPolicies::put(farming_policies);
                    FarmingPolicyID::put(id);

                    // add in the map to quickly filter farming policy ids by certificationtype
                    let mut farming_policy_ids_by_certification_type = FarmingPolicyIDsByCertificationType::get(certification_type);
                    farming_policy_ids_by_certification_type.push(id);
                    FarmingPolicyIDsByCertificationType::insert(certification_type, farming_policy_ids_by_certification_type);

                    Self::deposit_event(RawEvent::FarmingPolicyStored(new_policy));

                    Ok(())
                }
            }
        }
    }
}

impl<T: Config> Module<T> {
    pub fn convert_account_to_ed25519(account: T::AccountId) -> sp_core::ed25519::Public {
        // Decode entity's public key
        let account_vec = &account.encode();
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&account_vec);
        let ed25519_pubkey = sp_core::ed25519::Public::from_raw(bytes);

        return ed25519_pubkey;
    }

    pub fn fund_node_wallet(node_id: u32) {
        if !Nodes::contains_key(&node_id) {
            return;
        }

        let node = Nodes::get(node_id);
        if !Farms::contains_key(node.farm_id) {
            return;
        }
        let farm = Farms::get(node.farm_id);

        let node_twin = Twins::<T>::get(node.twin_id);
        let farm_twin = Twins::<T>::get(farm.twin_id);

        let node_twin_balance: BalanceOf<T> = T::Currency::free_balance(&node_twin.account_id);
        let minimal_balance = BalanceOf::<T>::saturated_from(1000000 as u128);

        if node_twin_balance <= minimal_balance {
            let farmer_twin_balance: BalanceOf<T> =
                T::Currency::free_balance(&farm_twin.account_id);
            let balance_to_transfer = BalanceOf::<T>::saturated_from(10000000 as u128);

            if farmer_twin_balance <= balance_to_transfer {
                debug::info!("farmer does not have enough balance to transfer");
                return;
            }

            debug::info!(
                "Transfering: {:?} from farmer twin {:?} to node twin {:?}",
                &balance_to_transfer,
                &farm_twin.account_id,
                &node_twin.account_id
            );
            if let Err(_) = T::Currency::transfer(
                &farm_twin.account_id,
                &node_twin.account_id,
                balance_to_transfer,
                KeepAlive,
            ) {
                debug::error!("Can't make transfer from farmer twin to node twin");
            };
        }
    }
}
