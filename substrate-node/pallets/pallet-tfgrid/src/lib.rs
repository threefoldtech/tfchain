#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

use codec::Encode;
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch, ensure, traits::EnsureOrigin,
    traits::Get, weights::Pays,
};
use frame_system::{self as system, ensure_signed, RawOrigin};
use hex::FromHex;
use pallet_timestamp as timestamp;
use sp_runtime::{traits::SaturatedConversion, DispatchError};
use sp_std::prelude::*;
use tfchain_support::{
    resources,
    traits::ChangeNode,
    types::{
        Farm, FarmCertification, FarmingPolicyLimit, Interface, Location, Node, NodeCertification,
        PublicConfig, PublicIP, Resources,
    },
};

#[cfg(test)]
mod tests;

mod benchmarking;
#[cfg(test)]
mod mock;

pub mod weights;

pub mod types;

pub mod farm_migration;
pub mod node_migration;

pub use weights::WeightInfo;

pub trait Config: system::Config + timestamp::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    /// Origin for restricted extrinsics
    /// Can be the root or another origin configured in the runtime
    type RestrictedOrigin: EnsureOrigin<Self::Origin>;
    /// Weight information for extrinsics in this pallet.
    type WeightInfo: WeightInfo;

    type NodeChanged: ChangeNode;
}

// Version constant that referenced the struct version
pub const TFGRID_ENTITY_VERSION: u32 = 1;
pub const TFGRID_FARM_VERSION: u32 = 3;
pub const TFGRID_TWIN_VERSION: u32 = 1;
pub const TFGRID_NODE_VERSION: u32 = 4;
pub const TFGRID_PRICING_POLICY_VERSION: u32 = 2;
pub const TFGRID_CERTIFICATION_CODE_VERSION: u32 = 1;
pub const TFGRID_FARMING_POLICY_VERSION: u32 = 2;

decl_storage! {
    trait Store for Module<T: Config> as TfgridModule {
        pub Farms get(fn farms): map hasher(blake2_128_concat) u32 => Farm;
        pub FarmIdByName get(fn farms_by_name_id): map hasher(blake2_128_concat) Vec<u8> => u32;
        pub FarmPayoutV2AddressByFarmID get(fn farm_payout_address_by_farm_id): map hasher(blake2_128_concat) u32 => Vec<u8>;
        pub DedicatedFarms get(fn dedicated_farms): Vec<u32>;

        pub Nodes get(fn nodes): map hasher(blake2_128_concat) u32 => Node;
        pub NodeIdByTwinID get(fn node_by_twin_id): map hasher(blake2_128_concat) u32 => u32;

        pub Entities get(fn entities): map hasher(blake2_128_concat) u32 => types::Entity<T::AccountId>;
        pub EntityIdByAccountID get(fn entities_by_pubkey_id): map hasher(blake2_128_concat) T::AccountId => u32;
        pub EntityIdByName get(fn entities_by_name_id): map hasher(blake2_128_concat) Vec<u8> => u32;

        pub Twins get(fn twins): map hasher(blake2_128_concat) u32 => types::Twin<T::AccountId>;
        pub TwinIdByAccountID get(fn twin_ids_by_pubkey): map hasher(blake2_128_concat) T::AccountId => u32;

        pub PricingPolicies get(fn pricing_policies): map hasher(blake2_128_concat) u32 => types::PricingPolicy<T::AccountId>;
        pub PricingPolicyIdByName get(fn pricing_policies_by_name_id): map hasher(blake2_128_concat) Vec<u8> => u32;

        pub FarmingPoliciesMap get(fn farming_policies_map): map hasher(blake2_128_concat) u32 => types::FarmingPolicy<T::BlockNumber>;

        pub UsersTermsAndConditions get(fn users_terms_and_condition): map hasher(blake2_128_concat) T::AccountId => Vec<types::TermsAndConditions<T::AccountId>>;

        // Account that are allowed to mark nodes as certified
        pub AllowedNodeCertifiers get(fn allowed_node_certifiers): Vec<T::AccountId>;

        // Connection price
        pub ConnectionPrice: u32;

        // ID maps
        FarmID: u32;
        NodeID: u32;
        EntityID: u32;
        TwinID: u32;
        PricingPolicyID: u32;
        FarmingPolicyID: u32;

        /// The current version of the pallet.
        PalletVersion: types::StorageVersion = types::StorageVersion::V3Struct;
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
        config(discount_for_dedication_nodes): u8;

        config(farming_policy_diy_cu): u32;
        config(farming_policy_diy_nu): u32;
        config(farming_policy_diy_su): u32;
        config(farming_policy_diy_ipu): u32;

        config(farming_policy_certified_cu): u32;
        config(farming_policy_certified_nu): u32;
        config(farming_policy_certified_su): u32;
        config(farming_policy_certified_ipu): u32;

        config(connection_price): u32;

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
                sales_account,
                _config.discount_for_dedication_nodes
            );

            // let _ = <Module<T>>::create_farming_policy(
            //     RawOrigin::Root.into(),
            //     "threefold_default_diy_farming_policy".as_bytes().to_vec(),
            //     _config.farming_policy_diy_su,
            //     _config.farming_policy_diy_cu,
            //     _config.farming_policy_diy_nu,
            //     _config.farming_policy_diy_ipu,
            //     NodeCertification::Diy,
            // );

            // let _ = <Module<T>>::create_farming_policy(
            //     RawOrigin::Root.into(),
            //     "threefold_default_certified_farming_policy".as_bytes().to_vec(),
            //     _config.farming_policy_certified_su,
            //     _config.farming_policy_certified_cu,
            //     _config.farming_policy_certified_nu,
            //     _config.farming_policy_certified_ipu,
            //     NodeCertification::Certified,
            // );

            let _ = <Module<T>>::set_connection_price(
                RawOrigin::Root.into(),
                _config.connection_price
            );
        });
    }

}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Config>::AccountId,
        BlockNumber = <T as frame_system::Config>::BlockNumber,
    {
        FarmStored(Farm),
        FarmUpdated(Farm),
        FarmDeleted(u32),

        NodeStored(Node),
        NodeUpdated(Node),
        NodeDeleted(u32),
        NodeUptimeReported(u32, u64, u64),
        NodePublicConfigStored(u32, PublicConfig),

        EntityStored(types::Entity<AccountId>),
        EntityUpdated(types::Entity<AccountId>),
        EntityDeleted(u32),

        TwinStored(types::Twin<AccountId>),
        TwinUpdated(types::Twin<AccountId>),

        TwinEntityStored(u32, u32, Vec<u8>),
        TwinEntityRemoved(u32, u32),
        TwinDeleted(u32),

        PricingPolicyStored(types::PricingPolicy<AccountId>),
        // CertificationCodeStored(types::CertificationCodes),
        FarmingPolicyStored(types::FarmingPolicy<BlockNumber>),
        FarmPayoutV2AddressRegistered(u32, Vec<u8>),
        FarmMarkedAsDedicated(u32),
        ConnectionPriceSet(u32),
        NodeCertificationSet(u32, NodeCertification),
        NodeCertifierAdded(AccountId),
        NodeCertifierRemoved(AccountId),
        FarmingPolicyUpdated(types::FarmingPolicy<BlockNumber>),
        FarmingPolicySet(u32, Option<FarmingPolicyLimit>),
        FarmCertificationSet(u32, FarmCertification),
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
        InvalidFarmName,

        AlreadyCertifier,
        NotCertifier,
        NotAllowedToCertifyNode,

        FarmingPolicyNotExists,
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        fn on_runtime_upgrade() -> frame_support::weights::Weight {
            frame_support::debug::info!("Resetting farming policy ID map...");
            FarmingPolicyID::put(0);
            frame_support::debug::info!("Resetting farming policy ID map done!");
            100_000_000
        }

        #[weight = 100_000_000 + T::DbWeight::get().writes(1)]
        pub fn set_storage_version(origin, version: types::StorageVersion) -> dispatch::DispatchResult {
            T::RestrictedOrigin::ensure_origin(origin)?;

            PalletVersion::set(version);

            Ok(())
        }

        #[weight = <T as Config>::WeightInfo::create_farm()]
        pub fn create_farm(origin, name: Vec<u8>, public_ips: Vec<PublicIP>) -> dispatch::DispatchResult {
            let address = ensure_signed(origin)?;

            Self::validate_farm_name(name.clone())?;

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
            let mut pub_ips: Vec<PublicIP> = Vec::new();
            for ip in public_ips {
                match pub_ips.iter().position(|pub_ip| pub_ip.ip == ip.ip) {
                    Some(_) => return Err(Error::<T>::IpExists.into()),
                    None => {
                        pub_ips.push(PublicIP{
                            ip: ip.ip,
                            gateway: ip.gateway,
                            contract_id: 0
                        });
                    }
                };
            };

            let new_farm = Farm {
                version: TFGRID_FARM_VERSION,
                id,
                twin_id,
                name,
                pricing_policy_id: 1,
                certification: FarmCertification::NotCertified,
                public_ips: pub_ips,
                dedicated_farm: false,
                farming_policy_limits: None,
            };

            Farms::insert(id, &new_farm);
            FarmIdByName::insert(new_farm.name.clone(), id);
            FarmID::put(id);

            Self::deposit_event(RawEvent::FarmStored(new_farm));

            Ok(())
        }

        #[weight = 100_000_000 + T::DbWeight::get().writes(3) + T::DbWeight::get().reads(2)]
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

        #[weight = 100_000_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(2)]
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

        #[weight = 100_000_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(1)]
        pub fn set_farm_certification(origin, farm_id: u32, certification: FarmCertification) -> dispatch::DispatchResult {
            T::RestrictedOrigin::ensure_origin(origin)?;

            ensure!(Farms::contains_key(farm_id), Error::<T>::FarmNotExists);
            let mut stored_farm = Farms::get(farm_id);

            stored_farm.certification = certification;

            Farms::insert(farm_id, &stored_farm);

            Self::deposit_event(RawEvent::FarmCertificationSet(farm_id, certification));

            Ok(())
        }

        #[weight = 100_000_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(2)]
        pub fn add_farm_ip(origin, id: u32, ip: Vec<u8>, gateway: Vec<u8>) -> dispatch::DispatchResult {
            let address = ensure_signed(origin)?;

            ensure!(Farms::contains_key(id), Error::<T>::FarmNotExists);
            let mut stored_farm = Farms::get(id);

            let twin = Twins::<T>::get(stored_farm.twin_id);
            ensure!(twin.account_id == address, Error::<T>::CannotUpdateFarmWrongTwin);

            let new_ip = PublicIP {
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

        #[weight = 100_000_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(2)]
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

        #[weight = 100_000_000 + T::DbWeight::get().writes(2) + T::DbWeight::get().reads(2)]
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

        #[weight = <T as Config>::WeightInfo::create_node()]
        pub fn create_node(origin,
            farm_id: u32,
            resources: Resources,
            location: Location,
            country: Vec<u8>,
            city: Vec<u8>,
            interfaces: Vec<Interface>,
            secure_boot: bool,
            virtualized: bool,
            serial_number: Vec<u8>,
        ) -> dispatch::DispatchResult {
            let account_id = ensure_signed(origin)?;

            ensure!(Farms::contains_key(farm_id), Error::<T>::FarmNotExists);
            ensure!(TwinIdByAccountID::<T>::contains_key(&account_id), Error::<T>::TwinNotExists);
            let twin_id = TwinIdByAccountID::<T>::get(&account_id);

            ensure!(!NodeIdByTwinID::contains_key(twin_id), Error::<T>::NodeWithTwinIdExists);

            let mut id = NodeID::get();
            id = id+1;

            let created = <timestamp::Module<T>>::get().saturated_into::<u64>() / 1000;

            let mut new_node = Node {
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
                farming_policy_id: 0,
                interfaces,
                certification: NodeCertification::default(),
                secure_boot,
                virtualized,
                serial_number,
                connection_price: ConnectionPrice::get()
            };

            let farming_policy = Self::get_farming_policy(&new_node)?;
            new_node.farming_policy_id = farming_policy.id;

            Nodes::insert(id, &new_node);
            NodeID::put(id);
            NodeIdByTwinID::insert(twin_id, new_node.id);

            T::NodeChanged::node_changed(None, &new_node);

            Self::deposit_event(RawEvent::NodeStored(new_node));

            Ok(())
        }

        #[weight = <T as Config>::WeightInfo::update_node()]
        pub fn update_node(origin,
            node_id: u32,
            farm_id: u32,
            resources: Resources,
            location: Location,
            country: Vec<u8>,
            city: Vec<u8>,
            interfaces: Vec<Interface>,
            secure_boot: bool,
            virtualized: bool,
            serial_number: Vec<u8>,
        ) -> dispatch::DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;

            ensure!(Nodes::contains_key(&node_id), Error::<T>::NodeNotExists);
            ensure!(TwinIdByAccountID::<T>::contains_key(&account_id), Error::<T>::TwinNotExists);

            let twin_id = TwinIdByAccountID::<T>::get(&account_id);
            let node = Nodes::get(&node_id);
            ensure!(node.twin_id == twin_id, Error::<T>::NodeUpdateNotAuthorized);

            ensure!(Farms::contains_key(farm_id), Error::<T>::FarmNotExists);

            let old_node = Nodes::get(node_id);
            let mut stored_node = Nodes::get(node_id);

            stored_node.farm_id = farm_id;
            stored_node.resources = resources;
            stored_node.location = location;
            stored_node.country = country;
            stored_node.city = city;
            stored_node.interfaces = interfaces;
            stored_node.secure_boot = secure_boot;
            stored_node.virtualized = virtualized;
            stored_node.serial_number = serial_number;

            // override node in storage
            Nodes::insert(stored_node.id, &stored_node);

            T::NodeChanged::node_changed(Some(&old_node), &stored_node);

            Self::deposit_event(RawEvent::NodeUpdated(stored_node));

            Ok(Pays::No.into())
        }

        #[weight = 100_000_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(1)]
        pub fn set_node_certification(origin, node_id: u32, node_certification: NodeCertification) -> dispatch::DispatchResult {
            let account_id = ensure_signed(origin)?;

            let certifiers = AllowedNodeCertifiers::<T>::get();
            ensure!(certifiers.contains(&account_id), Error::<T>::NotAllowedToCertifyNode);

            ensure!(Nodes::contains_key(&node_id), Error::<T>::NodeNotExists);
            let mut stored_node = Nodes::get(node_id);

            stored_node.certification = node_certification;

            // Refetch farming policy and save it on the node
            let farming_policy = Self::get_farming_policy(&stored_node)?;
            stored_node.farming_policy_id = farming_policy.id;

            // override node in storage
            Nodes::insert(stored_node.id, &stored_node);

            Self::deposit_event(RawEvent::NodeCertificationSet(node_id, node_certification));

            Ok(())
        }

        #[weight = <T as Config>::WeightInfo::report_uptime()]
        pub fn report_uptime(origin, uptime: u64) -> dispatch::DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;

            ensure!(TwinIdByAccountID::<T>::contains_key(&account_id), Error::<T>::TwinNotExists);
            let twin_id = TwinIdByAccountID::<T>::get(account_id);

            ensure!(NodeIdByTwinID::contains_key(twin_id), Error::<T>::TwinNotExists);
            let node_id = NodeIdByTwinID::get(twin_id);

            ensure!(Nodes::contains_key(node_id), Error::<T>::NodeNotExists);

            let now = <timestamp::Module<T>>::get().saturated_into::<u64>() / 1000;

            Self::deposit_event(RawEvent::NodeUptimeReported(node_id, now, uptime));

            Ok(Pays::No.into())
        }

        #[weight = 100_000_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(3)]
        pub fn add_node_public_config(origin, farm_id: u32, node_id: u32, public_config: PublicConfig) -> dispatch::DispatchResult {
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

            Ok(())
        }

        #[weight = 100_000_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(2)]
        pub fn delete_node(origin, id: u32) -> dispatch::DispatchResult {
            let account_id = ensure_signed(origin)?;

            ensure!(Nodes::contains_key(id), Error::<T>::NodeNotExists);

            let stored_node = Nodes::get(id);
            let twin_id = TwinIdByAccountID::<T>::get(&account_id);
            ensure!(stored_node.twin_id == twin_id, Error::<T>::NodeUpdateNotAuthorized);

            // Call node deleted
            T::NodeChanged::node_deleted(&stored_node);

            Nodes::remove(id);

            Self::deposit_event(RawEvent::NodeDeleted(id));

            // Call node deleted
            T::NodeChanged::node_deleted(&stored_node);

            Ok(())
        }

        #[weight = 100_000_000 + T::DbWeight::get().writes(4) + T::DbWeight::get().reads(3)]
        pub fn create_entity(origin, target: T::AccountId, name: Vec<u8>, country: Vec<u8>, city: Vec<u8>, signature: Vec<u8>) -> dispatch::DispatchResult {
            let _ = ensure_signed(origin)?;

            ensure!(!EntityIdByName::contains_key(&name), Error::<T>::EntityWithNameExists);
            ensure!(!EntityIdByAccountID::<T>::contains_key(&target), Error::<T>::EntityWithPubkeyExists);
            ensure!(signature.len() == 128, Error::<T>::SignatureLenghtIsIncorrect);
            let decoded_signature_as_byteslice = <[u8; 64]>::from_hex(signature.clone()).expect("Decoding failed");
            let mut message = Vec::new();
            message.extend_from_slice(&name);
            message.extend_from_slice(&country);
            message.extend_from_slice(&city);

            ensure!(Self::verify_signature(decoded_signature_as_byteslice, &target, &message), Error::<T>::EntitySignatureDoesNotMatch);

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

        #[weight = 100_000_000 + T::DbWeight::get().writes(3) + T::DbWeight::get().reads(3)]
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
        #[weight = 100_000_000 + T::DbWeight::get().writes(3) + T::DbWeight::get().reads(2)]
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

        #[weight = <T as Config>::WeightInfo::create_twin()]
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

        #[weight = 100_000_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(3)]
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
        #[weight = 100_000_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(2)]
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

            let mut message = Vec::new();
            message.extend_from_slice(&entity_id.to_be_bytes());
            message.extend_from_slice(&twin_id.to_be_bytes());

            ensure!(Self::verify_signature(decoded_signature_as_byteslice, &stored_entity.account_id, &message), Error::<T>::EntitySignatureDoesNotMatch);

            // Store proof
            twin.entities.push(entity_proof);

            // Update twin
            Twins::<T>::insert(&twin_id, &twin);

            Self::deposit_event(RawEvent::TwinEntityStored(twin_id, entity_id, signature));

            Ok(())
        }

        #[weight = 100_000_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(1)]
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

        #[weight = 100_000_000 + T::DbWeight::get().writes(2) + T::DbWeight::get().reads(1)]
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

        #[weight = 100_000_000 + T::DbWeight::get().writes(3) + T::DbWeight::get().reads(2)]
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
            certified_sales_account: T::AccountId,
            discount_for_dedication_nodes: u8,
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
                discount_for_dedication_nodes,
            };

            PricingPolicies::<T>::insert(&id, &new_policy);
            PricingPolicyIdByName::insert(&new_policy.name, &id);
            PricingPolicyID::put(id);

            Self::deposit_event(RawEvent::PricingPolicyStored(new_policy));
            Ok(())
        }

        #[weight = 100_000_000 + T::DbWeight::get().writes(4) + T::DbWeight::get().reads(2)]
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
            certified_sales_account: T::AccountId,
            discount_for_dedication_nodes: u8
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
            pricing_policy.discount_for_dedication_nodes = discount_for_dedication_nodes;

            PricingPolicies::<T>::insert(&id, &pricing_policy);
            PricingPolicyIdByName::insert(&pricing_policy.name, &id);
            PricingPolicyID::put(id);

            Self::deposit_event(RawEvent::PricingPolicyStored(pricing_policy));

            Ok(())
        }

        #[weight = 100_000_000 + T::DbWeight::get().writes(2) + T::DbWeight::get().reads(3)]
        pub fn create_farming_policy(
            origin,
            name: Vec<u8>,
            su: u32,
            cu: u32,
            nu: u32,
            ipv4: u32,
            minimal_uptime: u16,
            policy_end: T::BlockNumber,
            immutable: bool,
            default: bool,
            node_certification: NodeCertification,
            farm_certification: FarmCertification
        ) -> dispatch::DispatchResult {
            T::RestrictedOrigin::ensure_origin(origin)?;

            let mut id = FarmingPolicyID::get();
            id = id+1;

            let now_block = system::Pallet::<T>::block_number();

            let new_policy = types::FarmingPolicy {
                version: TFGRID_FARMING_POLICY_VERSION,
                id,
                name,
                su,
                cu,
                nu,
                ipv4,
                minimal_uptime,
                policy_created: now_block,
                policy_end,
                immutable,
                default,
                node_certification,
                farm_certification,
            };

            FarmingPoliciesMap::<T>::insert(id, &new_policy);
            FarmingPolicyID::put(id);

            Self::deposit_event(RawEvent::FarmingPolicyStored(new_policy));

            Ok(())
        }

        #[weight = 100_000_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(2)]
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

        #[weight = 100_000_000 + T::DbWeight::get().writes(2) + T::DbWeight::get().reads(5)]
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

            // Call node deleted
            T::NodeChanged::node_deleted(&node);

            Nodes::remove(node_id);
            NodeIdByTwinID::remove(node.twin_id);

            // Call node deleted
            T::NodeChanged::node_deleted(&node);

            Self::deposit_event(RawEvent::NodeDeleted(node_id));

            Ok(())
        }

        #[weight = 100_000_000 + T::DbWeight::get().writes(3) + T::DbWeight::get().reads(2)]
        pub fn set_farm_dedicated(origin, farm_id: u32, dedicated: bool) -> dispatch::DispatchResult {
            T::RestrictedOrigin::ensure_origin(origin)?;

            ensure!(Farms::contains_key(farm_id), Error::<T>::FarmNotExists);

            let mut farm = Farms::get(farm_id);
            farm.dedicated_farm = dedicated;
            Farms::insert(farm_id, &farm);

            Self::deposit_event(RawEvent::FarmUpdated(farm));

            Ok(())
        }

        #[weight = 100_000_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(1)]
        pub fn force_reset_farm_ip(origin, farm_id: u32, ip: Vec<u8>) -> dispatch::DispatchResult {
            T::RestrictedOrigin::ensure_origin(origin)?;

            ensure!(Farms::contains_key(farm_id), Error::<T>::FarmNotExists);
            let mut stored_farm = Farms::get(farm_id);

            match stored_farm.public_ips.iter_mut().find(|pubip| pubip.ip == ip) {
                Some(ip) => {
                    ip.contract_id = 0;
                },
                None => return Err(Error::<T>::IpNotExists.into()),
            };

            Farms::insert(stored_farm.id, &stored_farm);

            Self::deposit_event(RawEvent::FarmUpdated(stored_farm));

            Ok(())
        }

        #[weight = 100_000_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(1)]
        pub fn set_connection_price(origin, price: u32) -> dispatch::DispatchResult {
            T::RestrictedOrigin::ensure_origin(origin)?;

            ConnectionPrice::set(price);

            Self::deposit_event(RawEvent::ConnectionPriceSet(price));

            Ok(())
        }

        #[weight = 100_000_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(1)]
        pub fn add_node_certifier(origin, who: T::AccountId) -> dispatch::DispatchResult {
            T::RestrictedOrigin::ensure_origin(origin)?;

            let mut certifiers = AllowedNodeCertifiers::<T>::get();
            let location = certifiers.binary_search(&who).err().ok_or(Error::<T>::AlreadyCertifier)?;
            certifiers.insert(location, who.clone());
            AllowedNodeCertifiers::<T>::put(certifiers);

            Self::deposit_event(RawEvent::NodeCertifierAdded(who));

            Ok(())
        }

        #[weight = 100_000_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(1)]
        pub fn remove_node_certifier(origin, who: T::AccountId) -> dispatch::DispatchResult {
            T::RestrictedOrigin::ensure_origin(origin)?;

            let mut certifiers = AllowedNodeCertifiers::<T>::get();
            let location = certifiers.binary_search(&who).ok().ok_or(Error::<T>::NotCertifier)?;
            certifiers.remove(location);
            AllowedNodeCertifiers::<T>::put(&certifiers);

            Self::deposit_event(RawEvent::NodeCertifierRemoved(who));

            Ok(())
        }

        #[weight = 100_000_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(1)]
        pub fn update_farming_policy(
            origin,
            id: u32,
            name: Vec<u8>,
            su: u32,
            cu: u32,
            nu: u32,
            ipv4: u32,
            minimal_uptime: u16,
            policy_end: T::BlockNumber,
            default: bool,
            node_certification: NodeCertification,
            farm_certification: FarmCertification
        ) -> dispatch::DispatchResult {
            T::RestrictedOrigin::ensure_origin(origin)?;

            ensure!(FarmingPoliciesMap::<T>::contains_key(id), Error::<T>::FarmingPolicyNotExists);

            let mut farming_policy = FarmingPoliciesMap::<T>::get(id);

            farming_policy.name = name;
            farming_policy.su = su;
            farming_policy.cu = cu;
            farming_policy.nu = nu;
            farming_policy.ipv4 = ipv4;
            farming_policy.minimal_uptime = minimal_uptime;
            farming_policy.policy_end = policy_end;
            farming_policy.default = default;
            farming_policy.node_certification = node_certification;
            farming_policy.farm_certification = farm_certification;

            FarmingPoliciesMap::<T>::insert(id, &farming_policy);

            Self::deposit_event(RawEvent::FarmingPolicyUpdated(farming_policy));

            Ok(())
        }

        #[weight = 100_000_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(1)]
        pub fn attach_policy_to_farm(
            origin,
            farm_id: u32,
            limits: Option<FarmingPolicyLimit>
        ) -> dispatch::DispatchResult {
            T::RestrictedOrigin::ensure_origin(origin)?;

            ensure!(Farms::contains_key(farm_id), Error::<T>::FarmNotExists);

            let mut farm = Farms::get(farm_id);
            farm.farming_policy_limits = limits.clone();
            Farms::insert(farm_id, farm);

            Self::deposit_event(RawEvent::FarmingPolicySet(farm_id, limits));

            Ok(())
        }
    }
}

impl<T: Config> Module<T> {
    pub fn verify_signature(signature: [u8; 64], target: &T::AccountId, payload: &Vec<u8>) -> bool {
        if Self::verify_ed_signature(signature, target, payload) {
            return true;
        } else if Self::verify_sr_signature(signature, target, payload) {
            return true;
        }

        false
    }

    fn verify_ed_signature(signature: [u8; 64], target: &T::AccountId, payload: &Vec<u8>) -> bool {
        let entity_pubkey_ed25519 = Self::convert_account_to_ed25519(target);
        // Decode signature into a ed25519 signature
        let ed25519_signature = sp_core::ed25519::Signature::from_raw(signature);

        sp_io::crypto::ed25519_verify(&ed25519_signature, &payload, &entity_pubkey_ed25519)
    }

    fn verify_sr_signature(signature: [u8; 64], target: &T::AccountId, payload: &Vec<u8>) -> bool {
        let entity_pubkey_sr25519 = Self::convert_account_to_sr25519(target);
        // Decode signature into a sr25519 signature
        let sr25519_signature = sp_core::sr25519::Signature::from_raw(signature);

        sp_io::crypto::sr25519_verify(&sr25519_signature, &payload, &entity_pubkey_sr25519)
    }

    fn convert_account_to_ed25519(account: &T::AccountId) -> sp_core::ed25519::Public {
        // Decode entity's public key
        let account_vec = &account.encode();
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&account_vec);
        sp_core::ed25519::Public::from_raw(bytes)
    }

    fn convert_account_to_sr25519(account: &T::AccountId) -> sp_core::sr25519::Public {
        // Decode entity's public key
        let account_vec = &account.encode();
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&account_vec);
        sp_core::sr25519::Public::from_raw(bytes)
    }

    fn validate_farm_name(name: Vec<u8>) -> dispatch::DispatchResult {
        ensure!(
            name.len() > 0 && name.len() <= 50,
            Error::<T>::InvalidFarmName
        );
        for character in &name {
            match character {
                // 45 = -
                c if *c == 45 => (),
                // 95 = _
                c if *c == 95 => (),
                // 45 -> 57 = 0,1,2 ..
                c if *c >= 48 && *c <= 57 => (),
                // 65 -> 90 = A, B, C, ..
                c if *c >= 65 && *c <= 90 => (),
                // 97 -> 122 = a, b, c, ..
                c if *c >= 97 && *c <= 122 => (),
                _ => return Err(DispatchError::from(Error::<T>::InvalidFarmName)),
            }
        }

        return Ok(());
    }

    fn get_farming_policy(
        node: &Node,
    ) -> Result<types::FarmingPolicy<T::BlockNumber>, DispatchError> {
        let mut farm = Farms::get(node.farm_id);

        // If there is a farming policy defined on the
        // farm policy limits, use that one
        match farm.farming_policy_limits {
            Some(mut limits) => {
                ensure!(
                    FarmingPoliciesMap::<T>::contains_key(limits.farming_policy_id),
                    Error::<T>::FarmingPolicyNotExists
                );
                match limits.end {
                    Some(end_timestamp) => {
                        let now = <timestamp::Module<T>>::get().saturated_into::<u64>() / 1000;
                        if now > end_timestamp {
                            return Self::get_default_farming_policy();
                        }
                    }
                    None => (),
                };

                match limits.cu {
                    Some(cu_limit) => {
                        let cu = resources::get_cu(node.resources);
                        if cu > cu_limit {
                            return Self::get_default_farming_policy();
                        }
                        limits.cu = Some(cu_limit - cu);
                    }
                    None => (),
                };

                match limits.su {
                    Some(su_limit) => {
                        let su = resources::get_su(node.resources);
                        if su > su_limit {
                            return Self::get_default_farming_policy();
                        }
                        limits.su = Some(su_limit - su);
                    }
                    None => (),
                };

                match limits.node_count {
                    Some(node_count) => {
                        if node_count == 0 {
                            return Self::get_default_farming_policy();
                        }
                        limits.node_count = Some(node_count - 1);
                    }
                    None => (),
                };

                // Save limits when decrement is done
                farm.farming_policy_limits = Some(limits.clone());
                Farms::insert(node.farm_id, farm);

                return Ok(FarmingPoliciesMap::<T>::get(limits.farming_policy_id));
            }
            None => (),
        };

        let mut policies: Vec<types::FarmingPolicy<T::BlockNumber>> =
            FarmingPoliciesMap::<T>::iter().map(|p| p.1).collect();

        policies.sort();
        policies.reverse();

        let possible_policy = policies
            .into_iter()
            .filter(|policy| {
                policy.node_certification <= node.certification
                    && policy.farm_certification <= farm.certification
            })
            .take(1)
            .next();

        match possible_policy {
            Some(policy) => Ok(policy),
            None => return Err(DispatchError::from(Error::<T>::FarmingPolicyNotExists)),
        }
    }

    fn get_default_farming_policy() -> Result<types::FarmingPolicy<T::BlockNumber>, DispatchError> {
        let mut policies: Vec<types::FarmingPolicy<T::BlockNumber>> =
            FarmingPoliciesMap::<T>::iter().map(|p| p.1).collect();

        policies.sort();
        policies.reverse();

        let possible_policy = policies
            .into_iter()
            .filter(|policy| {
                policy.default
            })
            .take(1)
            .next();

        match possible_policy {
            Some(policy) => Ok(policy),
            None => return Err(DispatchError::from(Error::<T>::FarmingPolicyNotExists)),
        }
    }
}

impl<T: Config> tfchain_support::traits::Tfgrid<T::AccountId> for Module<T> {
    fn get_farm(farm_id: u32) -> Farm {
        Farms::get(farm_id)
    }

    fn is_farm_owner(farm_id: u32, who: T::AccountId) -> bool {
        let farm = Farms::get(farm_id);
        let twin = Twins::<T>::get(farm.twin_id);
        twin.account_id == who
    }

    fn is_twin_owner(twin_id: u32, who: T::AccountId) -> bool {
        let twin = Twins::<T>::get(twin_id);
        twin.account_id == who
    }
}
