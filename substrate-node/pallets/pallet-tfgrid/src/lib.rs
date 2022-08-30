#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame
use sp_std::prelude::*;

use codec::Encode;
use frame_support::dispatch::DispatchErrorWithPostInfo;
use frame_support::{ensure, traits::EnsureOrigin};
use frame_system::{self as system, ensure_signed};
use hex::FromHex;
use pallet_timestamp as timestamp;
use sp_runtime::SaturatedConversion;
use tfchain_support::{resources, types::Node};

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod weights;

pub mod types;

pub mod farm;
pub mod grid_migration;
pub mod interface;
pub mod nodes_migration;
pub mod pub_config;
pub mod pub_ip;
pub mod twin;

// Definition of the pallet logic, to be aggregated at runtime definition
// through `construct_runtime`.
#[frame_support::pallet]
pub mod pallet {
    use super::types;
    use super::weights::WeightInfo;
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_support::{traits::ConstU32, BoundedVec};
    use frame_system::pallet_prelude::*;
    use pallet_timestamp as timestamp;
    use sp_std::{convert::TryInto, fmt::Debug, vec::Vec};
    use tfchain_support::{
        traits::ChangeNode,
        types::{
            Farm, FarmCertification, FarmingPolicyLimit, Interface, Location, Node,
            NodeCertification, PublicConfig, PublicIP, Resources, IP,
        },
    };

    use codec::FullCodec;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    // Version constant that referenced the struct version
    pub const TFGRID_ENTITY_VERSION: u32 = 1;
    pub const TFGRID_FARM_VERSION: u32 = 4;
    pub const TFGRID_TWIN_VERSION: u32 = 1;
    pub const TFGRID_NODE_VERSION: u32 = 5;
    pub const TFGRID_PRICING_POLICY_VERSION: u32 = 2;
    pub const TFGRID_CERTIFICATION_CODE_VERSION: u32 = 1;
    pub const TFGRID_FARMING_POLICY_VERSION: u32 = 2;

    pub type FarmNameInput<T> = BoundedVec<u8, <T as Config>::MaxFarmNameLength>;
    pub type FarmNameOf<T> = <T as Config>::FarmName;
    pub type PublicIpOf<T> = PublicIP<<T as Config>::PublicIP, <T as Config>::GatewayIP>;
    pub type FarmInfoOf<T> = Farm<<T as Config>::FarmName, PublicIpOf<T>>;
    #[pallet::storage]
    #[pallet::getter(fn farms)]
    pub type Farms<T: Config> = StorageMap<_, Blake2_128Concat, u32, FarmInfoOf<T>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn nodes_by_farm_id)]
    pub type NodesByFarmID<T: Config> = StorageMap<_, Blake2_128Concat, u32, Vec<u32>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn farms_by_name_id)]
    pub type FarmIdByName<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn farm_payout_address_by_farm_id)]
    pub type FarmPayoutV2AddressByFarmID<T: Config> =
        StorageMap<_, Blake2_128Concat, u32, Vec<u8>, ValueQuery>;

    pub type Ip4ConfigOf<T> = IP<<T as Config>::IP4, <T as Config>::GW4>;
    pub type Ip6ConfigOf<T> = IP<<T as Config>::IP6, <T as Config>::GW6>;

    pub type PubConfigOf<T> =
        PublicConfig<Ip4ConfigOf<T>, Option<Ip6ConfigOf<T>>, Option<<T as Config>::Domain>>;

    pub type InterfaceIp<T> = <T as Config>::InterfaceIP;
    pub type InterfaceIpsOf<T> =
        BoundedVec<<T as Config>::InterfaceIP, <T as Config>::MaxInterfaceIpsLength>;
    pub type InterfaceOf<T> =
        Interface<<T as Config>::InterfaceName, <T as Config>::InterfaceMac, InterfaceIpsOf<T>>;

    #[pallet::storage]
    #[pallet::getter(fn nodes)]
    pub type Nodes<T> =
        StorageMap<_, Blake2_128Concat, u32, Node<PubConfigOf<T>, InterfaceOf<T>>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn node_by_twin_id)]
    pub type NodeIdByTwinID<T> = StorageMap<_, Blake2_128Concat, u32, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn entities)]
    pub type Entities<T: Config> =
        StorageMap<_, Blake2_128Concat, u32, types::Entity<T::AccountId>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn entities_by_pubkey_id)]
    pub type EntityIdByAccountID<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u32, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn entities_by_name_id)]
    pub type EntityIdByName<T> = StorageMap<_, Blake2_128Concat, Vec<u8>, u32, ValueQuery>;

    pub type TwinIndex = u32;
    type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    type TwinInfoOf<T> = types::Twin<<T as Config>::TwinIp, AccountIdOf<T>>;
    pub type TwinIpInput = BoundedVec<u8, ConstU32<39>>;
    pub type TwinIpOf<T> = <T as Config>::TwinIp;

    #[pallet::storage]
    #[pallet::getter(fn twins)]
    pub type Twins<T: Config> =
        StorageMap<_, Blake2_128Concat, TwinIndex, TwinInfoOf<T>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn twin_ids_by_pubkey)]
    pub type TwinIdByAccountID<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u32, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn pricing_policies)]
    pub type PricingPolicies<T: Config> =
        StorageMap<_, Blake2_128Concat, u32, types::PricingPolicy<T::AccountId>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn pricing_policies_by_name_id)]
    pub type PricingPolicyIdByName<T> = StorageMap<_, Blake2_128Concat, Vec<u8>, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn farming_policies_map)]
    pub type FarmingPoliciesMap<T: Config> =
        StorageMap<_, Blake2_128Concat, u32, types::FarmingPolicy<T::BlockNumber>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn users_terms_and_condition)]
    pub type UsersTermsAndConditions<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Vec<types::TermsAndConditions<T::AccountId>>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn allowed_node_certifiers)]
    pub type AllowedNodeCertifiers<T: Config> = StorageValue<_, Vec<T::AccountId>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn connection_price)]
    pub type ConnectionPrice<T> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn farm_id)]
    pub type FarmID<T> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn node_id)]
    pub type NodeID<T> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn entity_id)]
    pub type EntityID<T> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn twin_id)]
    pub type TwinID<T> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn pricing_policy_id)]
    pub type PricingPolicyID<T> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn farming_policy_id)]
    pub type FarmingPolicyID<T> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn pallet_version)]
    pub type PalletVersion<T> = StorageValue<_, types::StorageVersion, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn zos_version)]
    pub type ZosVersion<T> = StorageValue<_, Vec<u8>, ValueQuery>;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_timestamp::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Origin for restricted extrinsics
        /// Can be the root or another origin configured in the runtime
        type RestrictedOrigin: EnsureOrigin<Self::Origin>;
        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;

        type NodeChanged: ChangeNode<super::PubConfigOf<Self>, super::InterfaceOf<Self>>;

        /// The type of a name.
        type TwinIp: FullCodec
            + Debug
            + PartialEq
            + Clone
            + TypeInfo
            + TryFrom<Vec<u8>, Error = Error<Self>>
            + MaxEncodedLen;

        /// The type of a name.
        type FarmName: FullCodec
            + Debug
            + PartialEq
            + Clone
            + TypeInfo
            + TryFrom<Vec<u8>, Error = Error<Self>>
            + Into<Vec<u8>>
            + MaxEncodedLen;

        /// The type of a name.
        type PublicIP: FullCodec
            + Debug
            + PartialEq
            + Eq
            + Clone
            + TypeInfo
            + TryFrom<Vec<u8>, Error = Error<Self>>
            + MaxEncodedLen;

        /// The type of a name.
        type GatewayIP: FullCodec
            + Debug
            + PartialEq
            + Eq
            + Clone
            + TypeInfo
            + TryFrom<Vec<u8>, Error = Error<Self>>
            + MaxEncodedLen;

        /// The type of a name.
        type IP4: FullCodec
            + Debug
            + PartialEq
            + Eq
            + Clone
            + TypeInfo
            + TryFrom<Vec<u8>, Error = Error<Self>>
            + MaxEncodedLen;

        /// The type of a name.
        type GW4: FullCodec
            + Debug
            + PartialEq
            + Eq
            + Clone
            + TypeInfo
            + TryFrom<Vec<u8>, Error = Error<Self>>
            + MaxEncodedLen;

        /// The type of a name.
        type IP6: FullCodec
            + Debug
            + PartialEq
            + Eq
            + Clone
            + TypeInfo
            + TryFrom<Vec<u8>, Error = Error<Self>>
            + MaxEncodedLen;

        /// The type of a name.
        type GW6: FullCodec
            + Debug
            + PartialEq
            + Eq
            + Clone
            + TypeInfo
            + TryFrom<Vec<u8>, Error = Error<Self>>
            + MaxEncodedLen;

        /// The type of a name.
        type Domain: FullCodec
            + Debug
            + PartialEq
            + Eq
            + Clone
            + TypeInfo
            + TryFrom<Vec<u8>, Error = Error<Self>>
            + MaxEncodedLen;

        /// The type of an interface name.
        type InterfaceName: FullCodec
            + Debug
            + PartialEq
            + Eq
            + Clone
            + TypeInfo
            + TryFrom<Vec<u8>, Error = Error<Self>>
            + MaxEncodedLen;

        /// The type of an interface mac address.
        type InterfaceMac: FullCodec
            + Debug
            + PartialEq
            + Eq
            + Clone
            + TypeInfo
            + TryFrom<Vec<u8>, Error = Error<Self>>
            + MaxEncodedLen;

        /// The type of an interface IP.
        type InterfaceIP: FullCodec
            + Debug
            + PartialEq
            + Eq
            + Clone
            + TypeInfo
            + TryFrom<Vec<u8>, Error = Error<Self>>
            + MaxEncodedLen;

        #[pallet::constant]
        type MaxFarmNameLength: Get<u32>;

        #[pallet::constant]
        type MaxInterfaceIpsLength: Get<u32>;

        // #[pallet::constant]
        // type MaxFarmIPs: Get<u32>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        FarmStored(FarmInfoOf<T>),
        FarmUpdated(FarmInfoOf<T>),
        FarmDeleted(u32),

        NodeStored(Node<pallet::PubConfigOf<T>, pallet::InterfaceOf<T>>),
        NodeUpdated(Node<pallet::PubConfigOf<T>, pallet::InterfaceOf<T>>),
        NodeDeleted(u32),
        NodeUptimeReported(u32, u64, u64),
        NodePublicConfigStored(u32, Option<pallet::PubConfigOf<T>>),

        EntityStored(types::Entity<T::AccountId>),
        EntityUpdated(types::Entity<T::AccountId>),
        EntityDeleted(u32),

        TwinStored(types::Twin<T::TwinIp, T::AccountId>),
        TwinUpdated(types::Twin<T::TwinIp, T::AccountId>),

        TwinEntityStored(u32, u32, Vec<u8>),
        TwinEntityRemoved(u32, u32),
        TwinDeleted(u32),

        PricingPolicyStored(types::PricingPolicy<T::AccountId>),
        // CertificationCodeStored(types::CertificationCodes),
        FarmingPolicyStored(types::FarmingPolicy<T::BlockNumber>),
        FarmPayoutV2AddressRegistered(u32, Vec<u8>),
        FarmMarkedAsDedicated(u32),
        ConnectionPriceSet(u32),
        NodeCertificationSet(u32, NodeCertification),
        NodeCertifierAdded(T::AccountId),
        NodeCertifierRemoved(T::AccountId),
        FarmingPolicyUpdated(types::FarmingPolicy<T::BlockNumber>),
        FarmingPolicySet(u32, Option<FarmingPolicyLimit>),
        FarmCertificationSet(u32, FarmCertification),

        ZosVersionUpdated(Vec<u8>),
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

        AlreadyCertifier,
        NotCertifier,
        NotAllowedToCertifyNode,

        FarmingPolicyNotExists,

        TwinIpTooShort,
        TwinIpTooLong,
        InvalidTwinIp,

        FarmNameTooShort,
        FarmNameTooLong,
        InvalidPublicIP,
        PublicIPToShort,
        PublicIPToLong,
        GatewayIPToShort,
        GatewayIPToLong,

        IP4ToShort,
        IP4ToLong,
        InvalidIP4,
        GW4ToShort,
        GW4ToLong,
        InvalidGW4,
        IP6ToShort,
        IP6ToLong,
        InvalidIP6,
        GW6ToShort,
        GW6ToLong,
        InvalidGW6,
        DomainToShort,
        DomainToLong,
        InvalidDomain,
        MethodIsDeprecated,
        InterfaceNameToShort,
        InterfaceNameToLong,
        InvalidInterfaceName,
        InterfaceMacToShort,
        InterfaceMacToLong,
        InvalidMacAddress,
        InterfaceIpToShort,
        InterfaceIpToLong,
        InvalidInterfaceIP,
        InvalidZosVersion,
        FarmingPolicyExpired,
    }

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub su_price_value: u32,
        pub su_price_unit: u32,
        pub nu_price_value: u32,
        pub nu_price_unit: u32,
        pub ipu_price_value: u32,
        pub ipu_price_unit: u32,
        pub cu_price_value: u32,
        pub cu_price_unit: u32,
        pub domain_name_price_value: u32,
        pub unique_name_price_value: u32,
        pub foundation_account: Option<T::AccountId>,
        pub sales_account: Option<T::AccountId>,
        pub discount_for_dedication_nodes: u8,
        pub farming_policy_diy_cu: u32,
        pub farming_policy_diy_nu: u32,
        pub farming_policy_diy_su: u32,
        pub farming_policy_diy_ipu: u32,
        pub farming_policy_diy_minimal_uptime: u16,
        pub farming_policy_certified_cu: u32,
        pub farming_policy_certified_nu: u32,
        pub farming_policy_certified_su: u32,
        pub farming_policy_certified_ipu: u32,
        pub farming_policy_certified_minimal_uptime: u16,
        pub connection_price: u32,
    }

    // The default value for the genesis config type.
    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                su_price_value: Default::default(),
                su_price_unit: Default::default(),
                nu_price_value: Default::default(),
                nu_price_unit: Default::default(),
                ipu_price_value: Default::default(),
                ipu_price_unit: Default::default(),
                cu_price_value: Default::default(),
                cu_price_unit: Default::default(),
                domain_name_price_value: Default::default(),
                unique_name_price_value: Default::default(),
                foundation_account: None,
                sales_account: None,
                discount_for_dedication_nodes: Default::default(),
                farming_policy_diy_cu: Default::default(),
                farming_policy_diy_nu: Default::default(),
                farming_policy_diy_su: Default::default(),
                farming_policy_diy_ipu: Default::default(),
                farming_policy_diy_minimal_uptime: Default::default(),
                farming_policy_certified_cu: Default::default(),
                farming_policy_certified_nu: Default::default(),
                farming_policy_certified_su: Default::default(),
                farming_policy_certified_ipu: Default::default(),
                farming_policy_certified_minimal_uptime: Default::default(),
                connection_price: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            let su = types::Policy {
                value: self.su_price_value,
                unit: types::Unit::from_u32(self.su_price_unit),
            };

            let cu = types::Policy {
                value: self.cu_price_value,
                unit: types::Unit::from_u32(self.cu_price_unit),
            };

            let nu = types::Policy {
                value: self.nu_price_value,
                unit: types::Unit::from_u32(self.nu_price_unit),
            };

            let ipu = types::Policy {
                value: self.ipu_price_value,
                unit: types::Unit::from_u32(self.ipu_price_unit),
            };

            let unique_name = types::Policy {
                value: self.unique_name_price_value,
                unit: types::Unit::default(),
            };

            let domain_name = types::Policy {
                value: self.domain_name_price_value,
                unit: types::Unit::default(),
            };

            match &self.foundation_account {
                Some(foundation_account) => match &self.sales_account {
                    Some(certified_sales_account) => {
                        let p_policy = types::PricingPolicy {
                            version: 1,
                            id: 1,
                            name: "threefold_default_pricing_policy".as_bytes().to_vec(),
                            su,
                            cu,
                            nu,
                            ipu,
                            unique_name,
                            domain_name,
                            foundation_account: foundation_account.clone(),
                            certified_sales_account: certified_sales_account.clone(),
                            discount_for_dedication_nodes: self.discount_for_dedication_nodes,
                        };
                        PricingPolicies::<T>::insert(1, p_policy);
                        PricingPolicyID::<T>::put(1);
                    }
                    None => (),
                },
                None => (),
            };

            FarmingPoliciesMap::<T>::insert(
                1,
                types::FarmingPolicy {
                    version: 1,
                    id: 1,
                    name: "threefold_default_diy_farming_policy".as_bytes().to_vec(),
                    su: self.farming_policy_diy_su,
                    cu: self.farming_policy_diy_cu,
                    nu: self.farming_policy_diy_nu,
                    ipv4: self.farming_policy_diy_ipu,
                    minimal_uptime: self.farming_policy_diy_minimal_uptime,
                    policy_created: T::BlockNumber::from(0 as u32),
                    policy_end: T::BlockNumber::from(0 as u32),
                    immutable: false,
                    default: true,
                    node_certification: NodeCertification::Diy,
                    farm_certification: FarmCertification::NotCertified,
                },
            );

            FarmingPoliciesMap::<T>::insert(
                2,
                types::FarmingPolicy {
                    version: 1,
                    id: 2,
                    name: "threefold_default_certified_farming_policy"
                        .as_bytes()
                        .to_vec(),
                    su: self.farming_policy_certified_su,
                    cu: self.farming_policy_certified_cu,
                    nu: self.farming_policy_certified_nu,
                    ipv4: self.farming_policy_certified_ipu,
                    minimal_uptime: self.farming_policy_certified_minimal_uptime,
                    policy_created: T::BlockNumber::from(0 as u32),
                    policy_end: T::BlockNumber::from(0 as u32),
                    immutable: false,
                    default: true,
                    node_certification: NodeCertification::Certified,
                    farm_certification: FarmCertification::NotCertified,
                },
            );
            FarmingPolicyID::<T>::put(2);

            ConnectionPrice::<T>::put(self.connection_price)
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_storage_version(
            origin: OriginFor<T>,
            version: types::StorageVersion,
        ) -> DispatchResultWithPostInfo {
            T::RestrictedOrigin::ensure_origin(origin)?;

            PalletVersion::<T>::set(version);

            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn create_farm(
            origin: OriginFor<T>,
            name: FarmNameInput<T>,
            public_ips: Vec<PublicIP<T::PublicIP, T::GatewayIP>>,
        ) -> DispatchResultWithPostInfo {
            let address = ensure_signed(origin)?;

            let farm_name = FarmNameOf::<T>::try_from(name.to_vec())
                .map_err(DispatchErrorWithPostInfo::from)?;

            ensure!(
                !FarmIdByName::<T>::contains_key(name.clone()),
                Error::<T>::FarmExists
            );
            let twin_id = TwinIdByAccountID::<T>::get(&address).ok_or(Error::<T>::TwinNotExists)?;
            let twin = Twins::<T>::get(twin_id).ok_or(Error::<T>::TwinNotExists)?;
            ensure!(
                twin.account_id == address,
                Error::<T>::CannotCreateFarmWrongTwin
            );

            let mut id = FarmID::<T>::get();
            id = id + 1;

            // reset all public ip contract id's
            // just a safeguard
            // filter out doubles
            let mut public_ips_list: BoundedVec<
                PublicIP<T::PublicIP, T::GatewayIP>,
                ConstU32<256>,
            > = vec![].try_into().unwrap();

            for ip in public_ips {
                match public_ips_list.iter().position(|pub_ip| pub_ip.ip == ip.ip) {
                    Some(_) => return Err(Error::<T>::IpExists.into()),
                    None => {
                        public_ips_list
                            .try_push(PublicIP {
                                ip: ip.ip,
                                gateway: ip.gateway,
                                contract_id: 0,
                            })
                            .or_else(|_| {
                                return Err(DispatchErrorWithPostInfo::from(
                                    Error::<T>::InvalidPublicIP,
                                ));
                            })?;
                    }
                };
            }

            let new_farm = Farm {
                version: TFGRID_FARM_VERSION,
                id,
                twin_id,
                name: farm_name,
                pricing_policy_id: 1,
                certification: FarmCertification::NotCertified,
                public_ips: public_ips_list,
                dedicated_farm: false,
                farming_policy_limits: None,
            };

            Farms::<T>::insert(id, &new_farm);
            FarmIdByName::<T>::insert(name.to_vec().clone(), id);
            FarmID::<T>::put(id);

            Self::deposit_event(Event::FarmStored(new_farm));

            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(3) + T::DbWeight::get().reads(2))]
        pub fn update_farm(
            origin: OriginFor<T>,
            id: u32,
            name: FarmNameInput<T>,
            pricing_policy_id: u32,
        ) -> DispatchResultWithPostInfo {
            let address = ensure_signed(origin)?;

            let new_farm_name = FarmNameOf::<T>::try_from(name.to_vec())
                .map_err(DispatchErrorWithPostInfo::from)?;

            let twin_id = TwinIdByAccountID::<T>::get(&address).ok_or(Error::<T>::TwinNotExists)?;

            let mut stored_farm = Farms::<T>::get(id).ok_or(Error::<T>::FarmNotExists)?;

            ensure!(
                stored_farm.twin_id == twin_id,
                Error::<T>::CannotUpdateFarmWrongTwin
            );

            if FarmIdByName::<T>::contains_key(name.clone()) {
                let farm_id_by_new_name = FarmIdByName::<T>::get(name.clone());
                // if the user picks a new name but it is taken by another farmer, don't allow it
                if farm_id_by_new_name != id {
                    return Err(Error::<T>::InvalidFarmName.into());
                }
            }

            let name: Vec<u8> = stored_farm.name.into();
            // Remove stored farm by name and insert new one
            FarmIdByName::<T>::remove(name.clone());

            stored_farm.name = new_farm_name;
            stored_farm.pricing_policy_id = pricing_policy_id;

            Farms::<T>::insert(id, &stored_farm);
            FarmIdByName::<T>::insert(name, stored_farm.id);

            Self::deposit_event(Event::FarmUpdated(stored_farm));

            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(2))]
        pub fn add_stellar_payout_v2address(
            origin: OriginFor<T>,
            farm_id: u32,
            stellar_address: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let address = ensure_signed(origin)?;

            let twin_id = TwinIdByAccountID::<T>::get(&address).ok_or(Error::<T>::TwinNotExists)?;

            let farm = Farms::<T>::get(farm_id).ok_or(Error::<T>::FarmNotExists)?;

            ensure!(
                farm.twin_id == twin_id,
                Error::<T>::CannotUpdateFarmWrongTwin
            );

            FarmPayoutV2AddressByFarmID::<T>::insert(&farm_id, &stellar_address);

            Self::deposit_event(Event::FarmPayoutV2AddressRegistered(
                farm_id,
                stellar_address,
            ));

            Ok(().into())
        }

        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(1))]
        pub fn set_farm_certification(
            origin: OriginFor<T>,
            farm_id: u32,
            certification: FarmCertification,
        ) -> DispatchResultWithPostInfo {
            T::RestrictedOrigin::ensure_origin(origin)?;

            let mut stored_farm = Farms::<T>::get(farm_id).ok_or(Error::<T>::FarmNotExists)?;

            stored_farm.certification = certification;

            Farms::<T>::insert(farm_id, &stored_farm);

            Self::deposit_event(Event::FarmCertificationSet(farm_id, certification));

            Ok(().into())
        }

        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(2))]
        pub fn add_farm_ip(
            origin: OriginFor<T>,
            id: u32,
            ip: Vec<u8>,
            gateway: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let address = ensure_signed(origin)?;

            let mut stored_farm = Farms::<T>::get(id).ok_or(Error::<T>::FarmNotExists)?;

            let twin = Twins::<T>::get(stored_farm.twin_id).ok_or(Error::<T>::TwinNotExists)?;
            ensure!(
                twin.account_id == address,
                Error::<T>::CannotUpdateFarmWrongTwin
            );

            let parsed_ip =
                <T as Config>::PublicIP::try_from(ip).map_err(DispatchErrorWithPostInfo::from)?;

            let parsed_gateway = <T as Config>::GatewayIP::try_from(gateway)
                .map_err(DispatchErrorWithPostInfo::from)?;

            let new_ip = PublicIP {
                ip: parsed_ip,
                gateway: parsed_gateway,
                contract_id: 0,
            };

            match stored_farm
                .public_ips
                .iter()
                .position(|public_ip| public_ip.ip == new_ip.ip)
            {
                Some(_) => return Err(Error::<T>::IpExists.into()),
                None => {
                    stored_farm.public_ips.try_push(new_ip).or_else(|_| {
                        return Err(DispatchErrorWithPostInfo::from(Error::<T>::InvalidPublicIP));
                    })?;
                    Farms::<T>::insert(stored_farm.id, &stored_farm);
                    Self::deposit_event(Event::FarmUpdated(stored_farm));
                    return Ok(().into());
                }
            };
        }

        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(2))]
        pub fn remove_farm_ip(
            origin: OriginFor<T>,
            id: u32,
            ip: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let address = ensure_signed(origin)?;

            let mut stored_farm = Farms::<T>::get(id).ok_or(Error::<T>::FarmNotExists)?;

            let twin = Twins::<T>::get(stored_farm.twin_id).ok_or(Error::<T>::TwinNotExists)?;
            ensure!(
                twin.account_id == address,
                Error::<T>::CannotUpdateFarmWrongTwin
            );

            let parsed_ip =
                <T as Config>::PublicIP::try_from(ip).map_err(DispatchErrorWithPostInfo::from)?;

            match stored_farm
                .public_ips
                .iter()
                .position(|pubip| pubip.ip == parsed_ip && pubip.contract_id == 0)
            {
                Some(index) => {
                    stored_farm.public_ips.remove(index);
                    Farms::<T>::insert(stored_farm.id, &stored_farm);
                    Self::deposit_event(Event::FarmUpdated(stored_farm));
                    Ok(().into())
                }
                None => Err(Error::<T>::IpNotExists.into()),
            }
        }

        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(2) + T::DbWeight::get().reads(2))]
        pub fn delete_farm(_origin: OriginFor<T>, _id: u32) -> DispatchResultWithPostInfo {
            Err(DispatchErrorWithPostInfo::from(Error::<T>::MethodIsDeprecated).into())
        }

        #[pallet::weight(<T as Config>::WeightInfo::create_node())]
        pub fn create_node(
            origin: OriginFor<T>,
            farm_id: u32,
            resources: Resources,
            location: Location,
            country: Vec<u8>,
            city: Vec<u8>,
            interfaces: Vec<pallet::InterfaceOf<T>>,
            secure_boot: bool,
            virtualized: bool,
            serial_number: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;

            ensure!(Farms::<T>::contains_key(farm_id), Error::<T>::FarmNotExists);
            ensure!(
                TwinIdByAccountID::<T>::contains_key(&account_id),
                Error::<T>::TwinNotExists
            );
            let twin_id =
                TwinIdByAccountID::<T>::get(&account_id).ok_or(Error::<T>::TwinNotExists)?;

            ensure!(
                !NodeIdByTwinID::<T>::contains_key(twin_id),
                Error::<T>::NodeWithTwinIdExists
            );

            let mut id = NodeID::<T>::get();
            id = id + 1;

            let created = <timestamp::Pallet<T>>::get().saturated_into::<u64>() / 1000;

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
                connection_price: ConnectionPrice::<T>::get(),
            };

            let farming_policy = Self::get_farming_policy(&new_node)?;
            new_node.farming_policy_id = farming_policy.id;
            new_node.certification = farming_policy.node_certification;

            Nodes::<T>::insert(id, &new_node);
            NodeID::<T>::put(id);
            NodeIdByTwinID::<T>::insert(twin_id, new_node.id);

            let mut nodes_by_farm = NodesByFarmID::<T>::get(farm_id);
            nodes_by_farm.push(id);
            NodesByFarmID::<T>::insert(farm_id, nodes_by_farm);

            T::NodeChanged::node_changed(None, &new_node);

            Self::deposit_event(Event::NodeStored(new_node));

            Ok(().into())
        }

        #[pallet::weight(<T as Config>::WeightInfo::update_node())]
        pub fn update_node(
            origin: OriginFor<T>,
            node_id: u32,
            farm_id: u32,
            resources: Resources,
            location: Location,
            country: Vec<u8>,
            city: Vec<u8>,
            interfaces: Vec<pallet::InterfaceOf<T>>,
            secure_boot: bool,
            virtualized: bool,
            serial_number: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;

            let mut stored_node = Nodes::<T>::get(&node_id).ok_or(Error::<T>::NodeNotExists)?;
            ensure!(
                TwinIdByAccountID::<T>::contains_key(&account_id),
                Error::<T>::TwinNotExists
            );

            let twin_id =
                TwinIdByAccountID::<T>::get(&account_id).ok_or(Error::<T>::TwinNotExists)?;
            ensure!(
                stored_node.twin_id == twin_id,
                Error::<T>::NodeUpdateNotAuthorized
            );

            ensure!(Farms::<T>::contains_key(farm_id), Error::<T>::FarmNotExists);

            let old_node = Nodes::<T>::get(node_id).ok_or(Error::<T>::NodeNotExists)?;

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
            Nodes::<T>::insert(stored_node.id, &stored_node);

            T::NodeChanged::node_changed(Some(&old_node), &stored_node);

            Self::deposit_event(Event::NodeUpdated(stored_node));

            Ok(Pays::No.into())
        }

        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(1))]
        pub fn set_node_certification(
            origin: OriginFor<T>,
            node_id: u32,
            node_certification: NodeCertification,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;

            if let Some(certifiers) = AllowedNodeCertifiers::<T>::get() {
                ensure!(
                    certifiers.contains(&account_id),
                    Error::<T>::NotAllowedToCertifyNode
                );

                ensure!(
                    Nodes::<T>::contains_key(&node_id),
                    Error::<T>::NodeNotExists
                );
                let mut stored_node = Nodes::<T>::get(node_id).ok_or(Error::<T>::NodeNotExists)?;

                stored_node.certification = node_certification;

                let current_node_policy =
                    FarmingPoliciesMap::<T>::get(stored_node.farming_policy_id);
                if current_node_policy.default {
                    // Refetch farming policy and save it on the node
                    let farming_policy = Self::get_farming_policy(&stored_node)?;
                    stored_node.farming_policy_id = farming_policy.id;
                }

                // override node in storage
                Nodes::<T>::insert(stored_node.id, &stored_node);

                Self::deposit_event(Event::NodeUpdated(stored_node));
                Self::deposit_event(Event::NodeCertificationSet(node_id, node_certification));
            }

            Ok(().into())
        }

        #[pallet::weight(<T as Config>::WeightInfo::report_uptime())]
        pub fn report_uptime(origin: OriginFor<T>, uptime: u64) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;

            let twin_id =
                TwinIdByAccountID::<T>::get(account_id).ok_or(Error::<T>::TwinNotExists)?;

            ensure!(
                NodeIdByTwinID::<T>::contains_key(twin_id),
                Error::<T>::TwinNotExists
            );
            let node_id = NodeIdByTwinID::<T>::get(twin_id);

            ensure!(Nodes::<T>::contains_key(node_id), Error::<T>::NodeNotExists);

            let now = <timestamp::Pallet<T>>::get().saturated_into::<u64>() / 1000;

            Self::deposit_event(Event::NodeUptimeReported(node_id, now, uptime));

            Ok(Pays::No.into())
        }

        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(3))]
        pub fn add_node_public_config(
            origin: OriginFor<T>,
            farm_id: u32,
            node_id: u32,
            public_config: Option<pallet::PubConfigOf<T>>,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;

            // check if this twin can update the farm with id passed
            let farm = Farms::<T>::get(farm_id).ok_or(Error::<T>::FarmNotExists)?;

            ensure!(
                Twins::<T>::contains_key(farm.twin_id),
                Error::<T>::TwinNotExists
            );
            let farm_twin = Twins::<T>::get(farm.twin_id).ok_or(Error::<T>::TwinNotExists)?;
            ensure!(
                farm_twin.account_id == account_id,
                Error::<T>::CannotUpdateFarmWrongTwin
            );

            // check if the node belong to the farm
            let mut node = Nodes::<T>::get(node_id).ok_or(Error::<T>::NodeNotExists)?;
            ensure!(node.farm_id == farm_id, Error::<T>::NodeUpdateNotAuthorized);

            // update the public config and save
            node.public_config = public_config.clone();
            Nodes::<T>::insert(node_id, node);

            Self::deposit_event(Event::NodePublicConfigStored(node_id, public_config));

            Ok(().into())
        }

        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(2))]
        pub fn delete_node(origin: OriginFor<T>, id: u32) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;

            let stored_node = Nodes::<T>::get(id).ok_or(Error::<T>::NodeNotExists)?;
            let twin_id =
                TwinIdByAccountID::<T>::get(&account_id).ok_or(Error::<T>::TwinNotExists)?;
            ensure!(
                stored_node.twin_id == twin_id,
                Error::<T>::NodeUpdateNotAuthorized
            );

            let mut nodes_by_farm = NodesByFarmID::<T>::get(stored_node.farm_id);
            let location = nodes_by_farm
                .binary_search(&id)
                .or(Err(Error::<T>::NodeNotExists))?;
            nodes_by_farm.remove(location);
            NodesByFarmID::<T>::insert(stored_node.farm_id, nodes_by_farm);

            // Call node deleted
            T::NodeChanged::node_deleted(&stored_node);

            Nodes::<T>::remove(id);

            Self::deposit_event(Event::NodeDeleted(id));

            // Call node deleted
            T::NodeChanged::node_deleted(&stored_node);

            Ok(().into())
        }

        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(4) + T::DbWeight::get().reads(3))]
        pub fn create_entity(
            origin: OriginFor<T>,
            target: T::AccountId,
            name: Vec<u8>,
            country: Vec<u8>,
            city: Vec<u8>,
            signature: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let _ = ensure_signed(origin)?;

            ensure!(
                !EntityIdByName::<T>::contains_key(&name),
                Error::<T>::EntityWithNameExists
            );
            ensure!(
                !EntityIdByAccountID::<T>::contains_key(&target),
                Error::<T>::EntityWithPubkeyExists
            );
            ensure!(
                signature.len() == 128,
                Error::<T>::SignatureLenghtIsIncorrect
            );
            let decoded_signature_as_byteslice =
                <[u8; 64]>::from_hex(signature.clone()).expect("Decoding failed");
            let mut message = Vec::new();
            message.extend_from_slice(&name);
            message.extend_from_slice(&country);
            message.extend_from_slice(&city);

            ensure!(
                Self::verify_signature(decoded_signature_as_byteslice, &target, &message),
                Error::<T>::EntitySignatureDoesNotMatch
            );

            let mut id = EntityID::<T>::get();
            id = id + 1;

            let entity = types::Entity::<T::AccountId> {
                version: TFGRID_ENTITY_VERSION,
                id,
                name: name.clone(),
                country,
                city,
                account_id: target.clone(),
            };

            Entities::<T>::insert(&id, &entity);
            EntityIdByName::<T>::insert(&name, id);
            EntityIdByAccountID::<T>::insert(&target, id);
            EntityID::<T>::put(id);

            Self::deposit_event(Event::EntityStored(entity));

            Ok(().into())
        }

        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(3) + T::DbWeight::get().reads(3))]
        pub fn update_entity(
            origin: OriginFor<T>,
            name: Vec<u8>,
            country: Vec<u8>,
            city: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;

            ensure!(
                !EntityIdByName::<T>::contains_key(&name),
                Error::<T>::EntityWithNameExists
            );

            let stored_entity_id =
                EntityIdByAccountID::<T>::get(&account_id).ok_or(Error::<T>::EntityNotExists)?;

            ensure!(
                Entities::<T>::contains_key(&stored_entity_id),
                Error::<T>::EntityNotExists
            );
            let mut stored_entity =
                Entities::<T>::get(stored_entity_id).ok_or(Error::<T>::EntityNotExists)?;

            ensure!(
                stored_entity.account_id == account_id,
                Error::<T>::CannotUpdateEntity
            );

            // remove entity by name id
            EntityIdByName::<T>::remove(&stored_entity.name);

            stored_entity.name = name.clone();
            stored_entity.country = country;
            stored_entity.city = city;

            // overwrite entity
            Entities::<T>::insert(&stored_entity_id, &stored_entity);

            // re-insert with new name
            EntityIdByName::<T>::insert(&name, stored_entity_id);

            Self::deposit_event(Event::EntityUpdated(stored_entity));

            Ok(().into())
        }

        // TODO: delete all object that have an entity id reference?
        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(3) + T::DbWeight::get().reads(2))]
        pub fn delete_entity(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;

            let stored_entity_id =
                EntityIdByAccountID::<T>::get(&account_id).ok_or(Error::<T>::EntityNotExists)?;

            let stored_entity =
                Entities::<T>::get(stored_entity_id).ok_or(Error::<T>::EntityNotExists)?;

            ensure!(
                stored_entity.account_id == account_id,
                Error::<T>::CannotDeleteEntity
            );

            // Remove entity from storage
            Entities::<T>::remove(&stored_entity_id);

            // remove entity by name id
            EntityIdByName::<T>::remove(&stored_entity.name);

            // remove entity by pubkey id
            EntityIdByAccountID::<T>::remove(&account_id);

            Self::deposit_event(Event::EntityDeleted(stored_entity_id));

            Ok(().into())
        }

        #[pallet::weight(<T as Config>::WeightInfo::create_twin())]
        pub fn create_twin(origin: OriginFor<T>, ip: TwinIpInput) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;

            ensure!(
                UsersTermsAndConditions::<T>::contains_key(account_id.clone()),
                Error::<T>::UserDidNotSignTermsAndConditions
            );

            ensure!(
                !TwinIdByAccountID::<T>::contains_key(&account_id),
                Error::<T>::TwinWithPubkeyExists
            );

            let mut twin_id = TwinID::<T>::get();
            twin_id = twin_id + 1;

            let twin_ip = Self::check_twin_ip(ip)?;

            let twin = types::Twin::<T::TwinIp, T::AccountId> {
                version: TFGRID_TWIN_VERSION,
                id: twin_id,
                account_id: account_id.clone(),
                entities: Vec::new(),
                ip: twin_ip,
            };

            Twins::<T>::insert(&twin_id, &twin);
            TwinID::<T>::put(twin_id);

            // add the twin id to this users map of twin ids
            TwinIdByAccountID::<T>::insert(&account_id.clone(), twin_id);

            Self::deposit_event(Event::TwinStored(twin));

            Ok(().into())
        }

        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(3))]
        pub fn update_twin(origin: OriginFor<T>, ip: TwinIpInput) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;

            let twin_id =
                TwinIdByAccountID::<T>::get(account_id.clone()).ok_or(Error::<T>::TwinNotExists)?;

            let mut twin = Twins::<T>::get(&twin_id).ok_or(Error::<T>::TwinNotExists)?;

            // Make sure only the owner of this twin can update his twin
            ensure!(
                twin.account_id == account_id,
                Error::<T>::UnauthorizedToUpdateTwin
            );

            let twin_ip = Self::check_twin_ip(ip)?;

            twin.ip = twin_ip;

            Twins::<T>::insert(&twin_id, &twin);

            Self::deposit_event(Event::TwinUpdated(twin));
            Ok(().into())
        }

        // Method for twins only
        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(2))]
        pub fn add_twin_entity(
            origin: OriginFor<T>,
            twin_id: u32,
            entity_id: u32,
            signature: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;

            let stored_entity = Entities::<T>::get(entity_id).ok_or(Error::<T>::EntityNotExists)?;

            let mut twin = Twins::<T>::get(&twin_id).ok_or(Error::<T>::TwinNotExists)?;
            // Make sure only the owner of this twin can call this method
            ensure!(
                twin.account_id == account_id,
                Error::<T>::UnauthorizedToUpdateTwin
            );

            let entity_proof = types::EntityProof {
                entity_id,
                signature: signature.clone(),
            };

            ensure!(
                !twin.entities.contains(&entity_proof),
                Error::<T>::EntityWithSignatureAlreadyExists
            );

            let decoded_signature_as_byteslice =
                <[u8; 64]>::from_hex(signature.clone()).expect("Decoding failed");

            let mut message = Vec::new();
            message.extend_from_slice(&entity_id.to_be_bytes());
            message.extend_from_slice(&twin_id.to_be_bytes());

            ensure!(
                Self::verify_signature(
                    decoded_signature_as_byteslice,
                    &stored_entity.account_id,
                    &message
                ),
                Error::<T>::EntitySignatureDoesNotMatch
            );

            // Store proof
            twin.entities.push(entity_proof);

            // Update twin
            Twins::<T>::insert(&twin_id, &twin);

            Self::deposit_event(Event::TwinEntityStored(twin_id, entity_id, signature));

            Ok(().into())
        }

        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(1))]
        pub fn delete_twin_entity(
            origin: OriginFor<T>,
            twin_id: u32,
            entity_id: u32,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;

            let mut twin = Twins::<T>::get(&twin_id).ok_or(Error::<T>::TwinNotExists)?;
            // Make sure only the owner of this twin can call this method
            ensure!(
                twin.account_id == account_id,
                Error::<T>::UnauthorizedToUpdateTwin
            );

            ensure!(
                twin.entities.iter().any(|v| v.entity_id == entity_id),
                Error::<T>::EntityNotExists
            );

            let index = twin
                .entities
                .iter()
                .position(|x| x.entity_id == entity_id)
                .unwrap();
            twin.entities.remove(index);

            // Update twin
            Twins::<T>::insert(&twin_id, &twin);

            Self::deposit_event(Event::TwinEntityRemoved(twin_id, entity_id));

            Ok(().into())
        }

        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(2) + T::DbWeight::get().reads(1))]
        pub fn delete_twin(origin: OriginFor<T>, twin_id: u32) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;

            let twin = Twins::<T>::get(&twin_id).ok_or(Error::<T>::TwinNotExists)?;
            // Make sure only the owner of this twin can call this method
            ensure!(
                twin.account_id == account_id,
                Error::<T>::UnauthorizedToUpdateTwin
            );

            Twins::<T>::remove(&twin_id);

            // remove twin id from this users map of twin ids
            TwinIdByAccountID::<T>::remove(&account_id.clone());

            Self::deposit_event(Event::TwinDeleted(twin_id));

            Ok(().into())
        }

        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(3) + T::DbWeight::get().reads(2))]
        pub fn create_pricing_policy(
            origin: OriginFor<T>,
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
        ) -> DispatchResultWithPostInfo {
            T::RestrictedOrigin::ensure_origin(origin)?;

            ensure!(
                !PricingPolicyIdByName::<T>::contains_key(&name),
                Error::<T>::PricingPolicyExists
            );

            let mut id = PricingPolicyID::<T>::get();
            id = id + 1;

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
            PricingPolicyIdByName::<T>::insert(&new_policy.name, &id);
            PricingPolicyID::<T>::put(id);

            Self::deposit_event(Event::PricingPolicyStored(new_policy));
            Ok(().into())
        }

        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(4) + T::DbWeight::get().reads(2))]
        pub fn update_pricing_policy(
            origin: OriginFor<T>,
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
            discount_for_dedication_nodes: u8,
        ) -> DispatchResultWithPostInfo {
            T::RestrictedOrigin::ensure_origin(origin)?;

            // Ensure pricing policy with same id already exists
            let mut pricing_policy =
                PricingPolicies::<T>::get(id).ok_or(Error::<T>::PricingPolicyNotExists)?;

            // if name exists ensure that it belongs to the same policy id
            if PricingPolicyIdByName::<T>::contains_key(&name) {
                let stored_id = PricingPolicyIdByName::<T>::get(&name);
                ensure!(
                    stored_id == id,
                    Error::<T>::PricingPolicyWithDifferentIdExists
                );
            }

            if name != pricing_policy.name {
                PricingPolicyIdByName::<T>::remove(&pricing_policy.name);
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
            PricingPolicyIdByName::<T>::insert(&pricing_policy.name, &id);
            PricingPolicyID::<T>::put(id);

            Self::deposit_event(Event::PricingPolicyStored(pricing_policy));

            Ok(().into())
        }

        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(2) + T::DbWeight::get().reads(3))]
        pub fn create_farming_policy(
            origin: OriginFor<T>,
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
            farm_certification: FarmCertification,
        ) -> DispatchResultWithPostInfo {
            T::RestrictedOrigin::ensure_origin(origin)?;

            let mut id = FarmingPolicyID::<T>::get();
            id = id + 1;

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
            FarmingPolicyID::<T>::put(id);

            Self::deposit_event(Event::FarmingPolicyStored(new_policy));

            Ok(().into())
        }

        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(2))]
        pub fn user_accept_tc(
            origin: OriginFor<T>,
            document_link: Vec<u8>,
            document_hash: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            let timestamp = <timestamp::Pallet<T>>::get().saturated_into::<u64>() / 1000;

            let t_and_c = types::TermsAndConditions {
                account_id: account_id.clone(),
                timestamp,
                document_link,
                document_hash,
            };

            let mut users_terms_and_condition =
                UsersTermsAndConditions::<T>::get(account_id.clone()).unwrap_or(vec![]);
            users_terms_and_condition.push(t_and_c);
            UsersTermsAndConditions::<T>::insert(account_id, users_terms_and_condition);

            Ok(().into())
        }

        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(2) + T::DbWeight::get().reads(5))]
        pub fn delete_node_farm(origin: OriginFor<T>, node_id: u32) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;

            // check if the farmer twin is authorized
            let farm_twin_id =
                TwinIdByAccountID::<T>::get(&account_id).ok_or(Error::<T>::TwinNotExists)?;
            // check if the ndode belong to said farm
            let node = Nodes::<T>::get(&node_id).ok_or(Error::<T>::NodeNotExists)?;
            let farm = Farms::<T>::get(node.farm_id).ok_or(Error::<T>::FarmNotExists)?;

            ensure!(
                Twins::<T>::contains_key(&farm.twin_id),
                Error::<T>::TwinNotExists
            );
            let farm_twin = Twins::<T>::get(farm.twin_id).ok_or(Error::<T>::TwinNotExists)?;
            ensure!(
                farm_twin_id == farm_twin.id,
                Error::<T>::FarmerNotAuthorized
            );

            let mut nodes_by_farm = NodesByFarmID::<T>::get(node.farm_id);
            let location = nodes_by_farm
                .binary_search(&node_id)
                .or(Err(Error::<T>::NodeNotExists))?;
            nodes_by_farm.remove(location);
            NodesByFarmID::<T>::insert(node.farm_id, nodes_by_farm);

            // Call node deleted
            T::NodeChanged::node_deleted(&node);

            Nodes::<T>::remove(node_id);
            NodeIdByTwinID::<T>::remove(node.twin_id);

            // Call node deleted
            T::NodeChanged::node_deleted(&node);

            Self::deposit_event(Event::NodeDeleted(node_id));

            Ok(().into())
        }

        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(3) + T::DbWeight::get().reads(2))]
        pub fn set_farm_dedicated(
            origin: OriginFor<T>,
            farm_id: u32,
            dedicated: bool,
        ) -> DispatchResultWithPostInfo {
            T::RestrictedOrigin::ensure_origin(origin)?;

            let mut farm = Farms::<T>::get(farm_id).ok_or(Error::<T>::FarmNotExists)?;
            farm.dedicated_farm = dedicated;
            Farms::<T>::insert(farm_id, &farm);

            Self::deposit_event(Event::FarmUpdated(farm));

            Ok(().into())
        }

        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(1))]
        pub fn force_reset_farm_ip(
            origin: OriginFor<T>,
            farm_id: u32,
            ip: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            T::RestrictedOrigin::ensure_origin(origin)?;

            ensure!(Farms::<T>::contains_key(farm_id), Error::<T>::FarmNotExists);
            let mut stored_farm = Farms::<T>::get(farm_id).ok_or(Error::<T>::FarmNotExists)?;

            let parsed_ip =
                <T as Config>::PublicIP::try_from(ip).map_err(DispatchErrorWithPostInfo::from)?;

            match stored_farm
                .public_ips
                .iter_mut()
                .find(|pubip| pubip.ip == parsed_ip)
            {
                Some(ip) => {
                    ip.contract_id = 0;
                }
                None => return Err(Error::<T>::IpNotExists.into()),
            };

            Farms::<T>::insert(stored_farm.id, &stored_farm);

            Self::deposit_event(Event::FarmUpdated(stored_farm));

            Ok(().into())
        }

        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(1))]
        pub fn set_connection_price(
            origin: OriginFor<T>,
            price: u32,
        ) -> DispatchResultWithPostInfo {
            T::RestrictedOrigin::ensure_origin(origin)?;

            ConnectionPrice::<T>::set(price);

            Self::deposit_event(Event::ConnectionPriceSet(price));

            Ok(().into())
        }

        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(1))]
        pub fn add_node_certifier(
            origin: OriginFor<T>,
            who: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            T::RestrictedOrigin::ensure_origin(origin)?;

            match AllowedNodeCertifiers::<T>::get() {
                Some(mut certifiers) => {
                    let location = certifiers
                        .binary_search(&who)
                        .err()
                        .ok_or(Error::<T>::AlreadyCertifier)?;
                    certifiers.insert(location, who.clone());
                    AllowedNodeCertifiers::<T>::put(certifiers);

                    Self::deposit_event(Event::NodeCertifierAdded(who));
                }
                None => {
                    let certifiers = vec![who.clone()];
                    AllowedNodeCertifiers::<T>::put(certifiers);
                    Self::deposit_event(Event::NodeCertifierAdded(who));
                }
            }

            Ok(().into())
        }

        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(1))]
        pub fn remove_node_certifier(
            origin: OriginFor<T>,
            who: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            T::RestrictedOrigin::ensure_origin(origin)?;

            if let Some(mut certifiers) = AllowedNodeCertifiers::<T>::get() {
                let location = certifiers
                    .binary_search(&who)
                    .ok()
                    .ok_or(Error::<T>::NotCertifier)?;
                certifiers.remove(location);
                AllowedNodeCertifiers::<T>::put(&certifiers);

                Self::deposit_event(Event::NodeCertifierRemoved(who));
            }
            Ok(().into())
        }

        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(1))]
        pub fn update_farming_policy(
            origin: OriginFor<T>,
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
            farm_certification: FarmCertification,
        ) -> DispatchResultWithPostInfo {
            T::RestrictedOrigin::ensure_origin(origin)?;

            ensure!(
                FarmingPoliciesMap::<T>::contains_key(id),
                Error::<T>::FarmingPolicyNotExists
            );

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

            Self::deposit_event(Event::FarmingPolicyUpdated(farming_policy));

            Ok(().into())
        }

        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(1))]
        pub fn attach_policy_to_farm(
            origin: OriginFor<T>,
            farm_id: u32,
            limits: Option<FarmingPolicyLimit>,
        ) -> DispatchResultWithPostInfo {
            T::RestrictedOrigin::ensure_origin(origin)?;

            if let Some(policy_limits) = limits {
                let farming_policy = FarmingPoliciesMap::<T>::get(policy_limits.farming_policy_id);
                let now = system::Pallet::<T>::block_number();

                // Policy end is expressed in number of blocks
                if farming_policy.policy_end != T::BlockNumber::from(0 as u32)
                    && now >= farming_policy.policy_created + farming_policy.policy_end
                {
                    return Err(DispatchErrorWithPostInfo::from(
                        Error::<T>::FarmingPolicyExpired,
                    ));
                }

                let mut farm = Farms::<T>::get(farm_id).ok_or(Error::<T>::FarmNotExists)?;
                // Save the policy limits and farm certification on the Farm object
                farm.farming_policy_limits = Some(policy_limits.clone());
                farm.certification = farming_policy.farm_certification;
                Farms::<T>::insert(farm_id, &farm);
                Self::deposit_event(Event::FarmUpdated(farm));

                // Give all the nodes in this farm the policy that is attached
                for node_id in NodesByFarmID::<T>::get(farm_id) {
                    match Nodes::<T>::get(node_id) {
                        Some(mut node) => {
                            let current_node_policy =
                                FarmingPoliciesMap::<T>::get(node.farming_policy_id);
                            // If the current policy attached to the node is default one, assign it the newly created policy
                            // because we wouldn't wanna override any existing non-default policies
                            if current_node_policy.default {
                                let policy = Self::get_farming_policy(&node)?;
                                // Save the new policy ID and certification on the Node object
                                node.farming_policy_id = policy.id;
                                node.certification = policy.node_certification;
                                Nodes::<T>::insert(node_id, &node);
                                Self::deposit_event(Event::NodeUpdated(node))
                            }
                        }
                        None => continue,
                    }
                }

                Self::deposit_event(Event::FarmingPolicySet(farm_id, Some(policy_limits)));
            }

            Ok(().into())
        }

        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(1))]
        pub fn set_zos_version(
            origin: OriginFor<T>,
            zos_version: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            T::RestrictedOrigin::ensure_origin(origin)?;

            ensure!(
                ZosVersion::<T>::get() != zos_version,
                Error::<T>::InvalidZosVersion
            );

            ZosVersion::<T>::put(&zos_version);

            Self::deposit_event(Event::ZosVersionUpdated(zos_version));

            Ok(().into())
        }
    }
}

use frame_support::pallet_prelude::DispatchResultWithPostInfo;
// Internal functions of the pallet
impl<T: Config> Pallet<T> {
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

    fn get_farming_policy(
        node: &Node<pallet::PubConfigOf<T>, pallet::InterfaceOf<T>>,
    ) -> Result<types::FarmingPolicy<T::BlockNumber>, DispatchErrorWithPostInfo> {
        let mut farm = Farms::<T>::get(node.farm_id).ok_or(Error::<T>::FarmNotExists)?;

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
                        let now = <timestamp::Pallet<T>>::get().saturated_into::<u64>() / 1000;
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
                // Update farm in farms map
                Farms::<T>::insert(node.farm_id, &farm);
                Self::deposit_event(Event::FarmUpdated(farm));

                let farming_policy = FarmingPoliciesMap::<T>::get(limits.farming_policy_id);
                return Ok(farming_policy);
            }
            None => (),
        };

        // Set the farming policy as the last stored farming
        // policy which certifications are not more qualified
        // than the current node and farm certifications
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
            None => {
                return Err(DispatchErrorWithPostInfo::from(
                    Error::<T>::FarmingPolicyNotExists,
                ))
            }
        }
    }

    // Set to se the default farming policy as the last stored default farming policy
    fn get_default_farming_policy(
    ) -> Result<types::FarmingPolicy<T::BlockNumber>, DispatchErrorWithPostInfo> {
        let mut policies: Vec<types::FarmingPolicy<T::BlockNumber>> =
            FarmingPoliciesMap::<T>::iter().map(|p| p.1).collect();

        policies.sort();
        policies.reverse();

        let possible_policy = policies
            .into_iter()
            .filter(|policy| policy.default)
            .take(1)
            .next();

        match possible_policy {
            Some(policy) => Ok(policy),
            None => {
                return Err(DispatchErrorWithPostInfo::from(
                    Error::<T>::FarmingPolicyNotExists,
                ))
            }
        }
    }

    fn check_twin_ip(ip: TwinIpInput) -> Result<TwinIpOf<T>, DispatchErrorWithPostInfo> {
        let ip = TwinIpOf::<T>::try_from(ip.to_vec()).map_err(DispatchErrorWithPostInfo::from)?;

        Ok(ip)
    }
}

impl<T: Config> tfchain_support::traits::Tfgrid<T::AccountId, T::FarmName, pallet::PublicIpOf<T>>
    for Pallet<T>
{
    fn get_farm(
        farm_id: u32,
    ) -> Option<tfchain_support::types::Farm<T::FarmName, pallet::PublicIpOf<T>>> {
        Farms::<T>::get(farm_id)
    }

    fn is_farm_owner(farm_id: u32, who: T::AccountId) -> bool {
        let farm = Farms::<T>::get(farm_id);
        if let Some(f) = farm {
            match Twins::<T>::get(f.twin_id) {
                Some(twin) => twin.account_id == who,
                None => false,
            }
        } else {
            false
        }
    }

    fn is_twin_owner(twin_id: u32, who: T::AccountId) -> bool {
        match Twins::<T>::get(twin_id) {
            Some(twin) => twin.account_id == who,
            None => false,
        }
    }
}
