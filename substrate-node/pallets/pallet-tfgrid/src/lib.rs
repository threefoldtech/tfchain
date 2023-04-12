#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame
use sp_std::prelude::*;

use codec::Encode;
use frame_support::dispatch::DispatchErrorWithPostInfo;
use frame_support::{
    dispatch::Pays, ensure, pallet_prelude::DispatchResultWithPostInfo, traits::EnsureOrigin,
    BoundedVec,
};
use frame_system::{self as system, ensure_signed};
use hex::FromHex;
use pallet_timestamp as timestamp;
use sp_runtime::SaturatedConversion;
use tfchain_support::{
    resources::Resources,
    types::{Interface, NodePower as NodePowerType, Power, PowerState, PublicIP},
};

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod weights;

pub mod types;

pub mod farm;
pub mod interface;
// pub mod ip;
pub mod migrations;
pub mod node;
pub mod terms_cond;

// Definition of the pallet logic, to be aggregated at runtime definition
// through `construct_runtime`.
#[frame_support::pallet]
pub mod pallet {
    use super::types;
    use super::weights::WeightInfo;
    use super::*;
    use frame_support::{pallet_prelude::*, Blake2_128Concat};
    use frame_support::{traits::ConstU32, BoundedVec};
    use frame_system::pallet_prelude::*;
    use pallet_timestamp as timestamp;
    use sp_std::{convert::TryInto, fmt::Debug, vec::Vec};
    use tfchain_support::{
        resources::Resources,
        traits::{ChangeNode, PublicIpModifier},
        types::{
            Farm, FarmCertification, FarmingPolicyLimit, Interface, Node, NodeCertification,
            PublicConfig, PublicIP, IP4, MAX_DOMAIN_NAME_LENGTH, MAX_GW4_LENGTH, MAX_GW6_LENGTH,
            MAX_IP4_LENGTH, MAX_IP6_LENGTH,
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

    // Input type for Farm Name
    pub type FarmNameInput<T> = BoundedVec<u8, <T as Config>::MaxFarmNameLength>;
    // Concrete Farm Name type
    pub type FarmNameOf<T> = <T as Config>::FarmName;

    // Input type for IP4 (IP & GW)
    pub type Ip4Input = BoundedVec<u8, ConstU32<{ MAX_IP4_LENGTH }>>;
    pub type Gw4Input = BoundedVec<u8, ConstU32<{ MAX_GW4_LENGTH }>>;

    // Input type for IP6 (IP & GW)
    pub type Ip6Input = BoundedVec<u8, ConstU32<{ MAX_IP6_LENGTH }>>;
    pub type Gw6Input = BoundedVec<u8, ConstU32<{ MAX_GW6_LENGTH }>>;

    // Input type for public ip list
    pub type PublicIpListInput<T> = BoundedVec<IP4, <T as Config>::MaxFarmPublicIps>;
    // Concrete type for public ip list type
    pub type PublicIpListOf = BoundedVec<PublicIP, ConstU32<256>>;

    // Farm information type
    pub type FarmInfoOf<T> = Farm<<T as Config>::FarmName>;

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

    pub type DomainInput = BoundedVec<u8, ConstU32<{ MAX_DOMAIN_NAME_LENGTH }>>;
    // Input type for interfaces
    pub type InterfaceNameInput = BoundedVec<u8, ConstU32<{ interface::MAX_INTF_NAME_LENGTH }>>;
    pub type InterfaceMacInput = BoundedVec<u8, ConstU32<{ interface::INTERFACE_MAC_LENGTH }>>;
    pub type InterfaceIpInput = BoundedVec<u8, ConstU32<{ interface::MAX_INTERFACE_IP_LENGTH }>>;
    pub type InterfaceIpsInput<T> =
        BoundedVec<InterfaceIpInput, <T as Config>::MaxInterfacesLength>;
    pub type InterfaceInput<T> = BoundedVec<
        Interface<InterfaceNameInput, InterfaceMacInput, InterfaceIpsInput<T>>,
        <T as Config>::MaxInterfaceIpsLength,
    >;
    // Concrete type for interfaces
    pub type InterfaceNameOf<T> = <T as Config>::InterfaceName;
    pub type InterfaceMacOf<T> = <T as Config>::InterfaceMac;
    pub type InterfaceIpOf<T> = <T as Config>::InterfaceIP;
    pub type InterfaceIpsOf<T> = BoundedVec<InterfaceIpOf<T>, <T as Config>::MaxInterfaceIpsLength>;
    pub type InterfaceOf<T> = Interface<InterfaceNameOf<T>, InterfaceMacOf<T>, InterfaceIpsOf<T>>;

    // Input type for location
    pub type CityNameInput = BoundedVec<u8, ConstU32<{ node::MAX_CITY_NAME_LENGTH }>>;
    pub type CountryNameInput = BoundedVec<u8, ConstU32<{ node::MAX_COUNTRY_NAME_LENGTH }>>;
    pub type LatitudeInput = BoundedVec<u8, ConstU32<{ node::MAX_LATITUDE_LENGTH }>>;
    pub type LongitudeInput = BoundedVec<u8, ConstU32<{ node::MAX_LONGITUDE_LENGTH }>>;
    pub type LocationInput =
        types::LocationInput<CityNameInput, CountryNameInput, LatitudeInput, LongitudeInput>;
    // Concrete type for location
    pub type CityNameOf<T> = <T as Config>::CityName;
    pub type CountryNameOf<T> = <T as Config>::CountryName;
    pub type LocationOf<T> = <T as Config>::Location;

    // Input type for serial number
    pub type SerialNumberInput = BoundedVec<u8, ConstU32<{ node::MAX_SERIAL_NUMBER_LENGTH }>>;
    // Concrete type for location
    pub type SerialNumberOf<T> = <T as Config>::SerialNumber;

    // Input type for resources
    pub type ResourcesInput = Resources;

    // Input type for terms and conditions
    pub type DocumentLinkInput = BoundedVec<u8, ConstU32<{ terms_cond::MAX_DOCUMENT_LINK_LENGTH }>>;
    pub type DocumentHashInput = BoundedVec<u8, ConstU32<{ terms_cond::MAX_DOCUMENT_HASH_LENGTH }>>;
    pub type TermsAndConditionsInput<T> =
        types::TermsAndConditionsInput<AccountIdOf<T>, DocumentLinkInput, DocumentHashInput>;

    // Concrete type for node
    pub type TfgridNode<T> = Node<LocationOf<T>, InterfaceOf<T>, SerialNumberOf<T>>;

    // Concrete type for entity
    pub type TfgridEntity<T> = types::Entity<AccountIdOf<T>, CityNameOf<T>, CountryNameOf<T>>;

    #[pallet::storage]
    #[pallet::getter(fn nodes)]
    pub type Nodes<T> = StorageMap<_, Blake2_128Concat, u32, TfgridNode<T>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn node_by_twin_id)]
    pub type NodeIdByTwinID<T> = StorageMap<_, Blake2_128Concat, u32, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn entities)]
    pub type Entities<T: Config> =
        StorageMap<_, Blake2_128Concat, u32, TfgridEntity<T>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn entities_by_pubkey_id)]
    pub type EntityIdByAccountID<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u32, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn entities_by_name_id)]
    pub type EntityIdByName<T> = StorageMap<_, Blake2_128Concat, Vec<u8>, u32, ValueQuery>;

    pub type TwinIndex = u32;
    pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    type TwinInfoOf<T> = types::Twin<AccountIdOf<T>>;
    pub type RelayInput = Option<BoundedVec<u8, ConstU32<{ types::MAX_RELAY_LENGTH }>>>;
    pub type PkInput = Option<BoundedVec<u8, ConstU32<{ types::MAX_PK_LENGTH }>>>;

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

    // Concrete type for location
    pub type TermsAndConditionsOf<T> = <T as Config>::TermsAndConditions;

    #[pallet::storage]
    #[pallet::getter(fn users_terms_and_condition)]
    pub type UsersTermsAndConditions<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, Vec<TermsAndConditionsOf<T>>, OptionQuery>;

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

    // This storage map maps a node ID to a power state, they node can modify this state
    // to indicate that it has shut down or came back alive
    #[pallet::storage]
    #[pallet::getter(fn node_power_state)]
    pub type NodePower<T: Config> =
        StorageMap<_, Blake2_128Concat, u32, NodePowerType<T::BlockNumber>, ValueQuery>;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_timestamp::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Origin for restricted extrinsics
        /// Can be the root or another origin configured in the runtime
        type RestrictedOrigin: EnsureOrigin<Self::RuntimeOrigin>;
        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;

        type NodeChanged: ChangeNode<
            super::LocationOf<Self>,
            super::InterfaceOf<Self>,
            super::SerialNumberOf<Self>,
        >;

        type PublicIpModifier: PublicIpModifier;

        /// The type of terms and conditions.
        type TermsAndConditions: FullCodec
            + Debug
            + PartialEq
            + Clone
            + TypeInfo
            + TryFrom<TermsAndConditionsInput<Self>, Error = Error<Self>>
            + MaxEncodedLen;

        /// The type of a farm name.
        type FarmName: FullCodec
            + Debug
            + PartialEq
            + Clone
            + TypeInfo
            + TryFrom<FarmNameInput<Self>, Error = Error<Self>>
            + Into<Vec<u8>>
            + MaxEncodedLen;

        /// The type of an interface name.
        type InterfaceName: FullCodec
            + Debug
            + PartialEq
            + Eq
            + Clone
            + TypeInfo
            + TryFrom<InterfaceNameInput, Error = Error<Self>>
            + MaxEncodedLen;

        /// The type of an interface mac address.
        type InterfaceMac: FullCodec
            + Debug
            + PartialEq
            + Eq
            + Clone
            + TypeInfo
            + TryFrom<InterfaceMacInput, Error = Error<Self>>
            + MaxEncodedLen;

        /// The type of an interface IP.
        type InterfaceIP: FullCodec
            + Debug
            + PartialEq
            + Eq
            + Clone
            + TypeInfo
            + TryFrom<InterfaceIpInput, Error = Error<Self>>
            + MaxEncodedLen;

        /// The type of a city name.
        type CityName: FullCodec
            + Debug
            + Default
            + PartialEq
            + Clone
            + TypeInfo
            + TryFrom<CityNameInput, Error = Error<Self>>
            + MaxEncodedLen;

        /// The type of a country name.
        type CountryName: FullCodec
            + Debug
            + Default
            + PartialEq
            + Clone
            + TypeInfo
            + TryFrom<CountryNameInput, Error = Error<Self>>
            + MaxEncodedLen;

        /// The type of a location.
        type Location: FullCodec
            + Debug
            + Default
            + PartialEq
            + Clone
            + TypeInfo
            + TryFrom<LocationInput, Error = Error<Self>>
            + MaxEncodedLen;

        /// The type of a serial number.
        type SerialNumber: FullCodec
            + Debug
            + Default
            + PartialEq
            + Clone
            + TypeInfo
            + TryFrom<SerialNumberInput, Error = Error<Self>>
            + MaxEncodedLen;

        #[pallet::constant]
        type MaxFarmNameLength: Get<u32>;

        #[pallet::constant]
        type MaxFarmPublicIps: Get<u32>;

        #[pallet::constant]
        type MaxInterfacesLength: Get<u32>;

        #[pallet::constant]
        type MaxInterfaceIpsLength: Get<u32>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        FarmStored(FarmInfoOf<T>),
        FarmUpdated(FarmInfoOf<T>),
        FarmDeleted(u32),

        NodeStored(TfgridNode<T>),
        NodeUpdated(TfgridNode<T>),
        NodeDeleted(u32),
        NodeUptimeReported(u32, u64, u64),
        NodePublicConfigStored(u32, Option<pallet::PublicConfig>),

        EntityStored(TfgridEntity<T>),
        EntityUpdated(TfgridEntity<T>),
        EntityDeleted(u32),

        TwinStored(types::Twin<T::AccountId>),
        TwinUpdated(types::Twin<T::AccountId>),

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

        /// Send an event to zero os to change its state
        PowerTargetChanged {
            farm_id: u32,
            node_id: u32,
            power_target: Power,
        },
        PowerStateChanged {
            farm_id: u32,
            node_id: u32,
            power_state: PowerState<T::BlockNumber>,
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
        SignatureLengthIsIncorrect,

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

        RelayTooShort,
        RelayTooLong,
        InvalidRelay,

        FarmNameTooShort,
        FarmNameTooLong,
        InvalidPublicIP,
        PublicIPTooShort,
        PublicIPTooLong,
        GatewayIPTooShort,
        GatewayIPTooLong,

        IP4TooShort,
        IP4TooLong,
        InvalidIP4,
        GW4TooShort,
        GW4TooLong,
        InvalidGW4,
        IP6TooShort,
        IP6TooLong,
        InvalidIP6,
        GW6TooShort,
        GW6TooLong,
        InvalidGW6,
        DomainTooShort,
        DomainTooLong,
        InvalidDomain,
        MethodIsDeprecated,
        InterfaceNameTooShort,
        InterfaceNameTooLong,
        InvalidInterfaceName,
        InterfaceMacTooShort,
        InterfaceMacTooLong,
        InvalidMacAddress,
        InterfaceIpTooShort,
        InterfaceIpTooLong,
        InvalidInterfaceIP,
        InvalidZosVersion,
        FarmingPolicyExpired,

        InvalidHRUInput,
        InvalidSRUInput,
        InvalidCRUInput,
        InvalidMRUInput,

        LatitudeInputTooShort,
        LatitudeInputTooLong,
        InvalidLatitudeInput,
        LongitudeInputTooShort,
        LongitudeInputTooLong,
        InvalidLongitudeInput,
        CountryNameTooShort,
        CountryNameTooLong,
        InvalidCountryName,
        CityNameTooShort,
        CityNameTooLong,
        InvalidCityName,
        InvalidCountryCityPair,

        SerialNumberTooShort,
        SerialNumberTooLong,
        InvalidSerialNumber,

        DocumentLinkInputTooShort,
        DocumentLinkInputTooLong,
        InvalidDocumentLinkInput,
        DocumentHashInputTooShort,
        DocumentHashInputTooLong,
        InvalidDocumentHashInput,

        InvalidPublicConfig,
        UnauthorizedToChangePowerTarget,
        InvalidRelayAddress,
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
        #[pallet::call_index(0)]
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
        pub fn set_storage_version(
            origin: OriginFor<T>,
            version: types::StorageVersion,
        ) -> DispatchResultWithPostInfo {
            T::RestrictedOrigin::ensure_origin(origin)?;

            PalletVersion::<T>::set(version);

            Ok(().into())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
        pub fn create_farm(
            origin: OriginFor<T>,
            name: FarmNameInput<T>,
            public_ips: PublicIpListInput<T>,
        ) -> DispatchResultWithPostInfo {
            let address = ensure_signed(origin)?;

            let mut id = FarmID::<T>::get();
            id = id + 1;

            let twin_id = TwinIdByAccountID::<T>::get(&address).ok_or(Error::<T>::TwinNotExists)?;
            let twin = Twins::<T>::get(twin_id).ok_or(Error::<T>::TwinNotExists)?;
            ensure!(
                twin.account_id == address,
                Error::<T>::CannotCreateFarmWrongTwin
            );

            ensure!(
                !FarmIdByName::<T>::contains_key(name.clone()),
                Error::<T>::FarmExists
            );
            let farm_name = Self::get_farm_name(name.clone())?;

            let public_ips_list = Self::get_public_ips(public_ips)?;

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
            FarmIdByName::<T>::insert(name.to_vec(), id);
            FarmID::<T>::put(id);

            Self::deposit_event(Event::FarmStored(new_farm));

            Ok(().into())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(10_000 + T::DbWeight::get().writes(3).ref_time() + T::DbWeight::get().reads(2).ref_time())]
        pub fn update_farm(
            origin: OriginFor<T>,
            id: u32,
            name: FarmNameInput<T>,
        ) -> DispatchResultWithPostInfo {
            let address = ensure_signed(origin)?;

            let new_farm_name = Self::get_farm_name(name.clone())?;

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

            Farms::<T>::insert(id, &stored_farm);
            FarmIdByName::<T>::insert(name, stored_farm.id);

            Self::deposit_event(Event::FarmUpdated(stored_farm));

            Ok(().into())
        }

        #[pallet::call_index(3)]
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time() + T::DbWeight::get().reads(2).ref_time())]
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

        #[pallet::call_index(4)]
        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(1).ref_time() + T::DbWeight::get().reads(1).ref_time())]
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

        #[pallet::call_index(5)]
        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(1).ref_time() + T::DbWeight::get().reads(2).ref_time())]
        pub fn add_farm_ip(
            origin: OriginFor<T>,
            id: u32,
            ip: Ip4Input,
            gw: Gw4Input,
        ) -> DispatchResultWithPostInfo {
            let address = ensure_signed(origin)?;

            let mut stored_farm = Farms::<T>::get(id).ok_or(Error::<T>::FarmNotExists)?;

            let twin = Twins::<T>::get(stored_farm.twin_id).ok_or(Error::<T>::TwinNotExists)?;
            ensure!(
                twin.account_id == address,
                Error::<T>::CannotUpdateFarmWrongTwin
            );

            // Check if it's a valid IP4
            let ip4 = IP4 { ip, gw };
            ip4.is_valid().map_err(|_| Error::<T>::InvalidPublicIP)?;

            let new_ip = PublicIP {
                ip: ip4.ip,
                gateway: ip4.gw,
                contract_id: 0,
            };

            match stored_farm
                .public_ips
                .iter()
                .position(|public_ip| public_ip.ip == new_ip.ip)
            {
                Some(_) => return Err(Error::<T>::IpExists.into()),
                None => {
                    stored_farm
                        .public_ips
                        .try_push(new_ip)
                        .map_err(|_| Error::<T>::InvalidPublicIP)?;
                    Farms::<T>::insert(stored_farm.id, &stored_farm);
                    Self::deposit_event(Event::FarmUpdated(stored_farm));
                    return Ok(().into());
                }
            };
        }

        #[pallet::call_index(6)]
        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(1).ref_time() + T::DbWeight::get().reads(2).ref_time())]
        pub fn remove_farm_ip(
            origin: OriginFor<T>,
            id: u32,
            ip: Ip4Input,
        ) -> DispatchResultWithPostInfo {
            let address = ensure_signed(origin)?;

            let mut stored_farm = Farms::<T>::get(id).ok_or(Error::<T>::FarmNotExists)?;

            let twin = Twins::<T>::get(stored_farm.twin_id).ok_or(Error::<T>::TwinNotExists)?;
            ensure!(
                twin.account_id == address,
                Error::<T>::CannotUpdateFarmWrongTwin
            );

            match stored_farm
                .public_ips
                .iter()
                .position(|pubip| pubip.ip == ip && pubip.contract_id == 0)
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

        #[pallet::call_index(8)]
        #[pallet::weight(<T as Config>::WeightInfo::create_node())]
        pub fn create_node(
            origin: OriginFor<T>,
            farm_id: u32,
            resources: ResourcesInput,
            location: LocationInput,
            interfaces: InterfaceInput<T>,
            secure_boot: bool,
            virtualized: bool,
            serial_number: Option<SerialNumberInput>,
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

            let node_resources = Self::get_resources(resources)?;
            let node_location = Self::get_location(location)?;
            let node_interfaces = Self::get_interfaces(&interfaces)?;

            let node_serial_number = if let Some(serial_input) = serial_number {
                Some(Self::get_serial_number(serial_input)?)
            } else {
                None
            };

            let created = <timestamp::Pallet<T>>::get().saturated_into::<u64>() / 1000;

            let mut new_node = Node {
                version: TFGRID_NODE_VERSION,
                id,
                farm_id,
                twin_id,
                resources: node_resources,
                location: node_location,
                public_config: None,
                created,
                farming_policy_id: 0,
                interfaces: node_interfaces,
                certification: NodeCertification::default(),
                secure_boot,
                virtualized,
                serial_number: node_serial_number,
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

        #[pallet::call_index(9)]
        #[pallet::weight(<T as Config>::WeightInfo::update_node())]
        pub fn update_node(
            origin: OriginFor<T>,
            node_id: u32,
            farm_id: u32,
            resources: ResourcesInput,
            location: LocationInput,
            interfaces: InterfaceInput<T>,
            secure_boot: bool,
            virtualized: bool,
            serial_number: Option<SerialNumberInput>,
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

            // If the farm ID changed on the node,
            // remove the node from the old map from the farm and insert into the correct one
            if old_node.farm_id != farm_id {
                let mut old_nodes_by_farm = NodesByFarmID::<T>::get(old_node.farm_id);
                old_nodes_by_farm.retain(|&id| id != node_id);
                NodesByFarmID::<T>::insert(old_node.farm_id, old_nodes_by_farm);

                let mut nodes_by_farm = NodesByFarmID::<T>::get(farm_id);
                nodes_by_farm.push(node_id);
                NodesByFarmID::<T>::insert(farm_id, nodes_by_farm);
            };

            let node_resources = Self::get_resources(resources)?;
            let node_location = Self::get_location(location)?;
            let node_interfaces = Self::get_interfaces(&interfaces)?;

            let node_serial_number = if let Some(serial_input) = serial_number {
                Some(Self::get_serial_number(serial_input)?)
            } else {
                None
            };

            // If the resources on a certified node changed, reset the certification level to DIY
            if Resources::has_changed(&stored_node.resources, &node_resources, 1)
                && stored_node.certification == NodeCertification::Certified
            {
                stored_node.certification = NodeCertification::Diy;
                Self::deposit_event(Event::NodeCertificationSet(
                    node_id,
                    stored_node.certification,
                ));
            }

            stored_node.farm_id = farm_id;
            stored_node.resources = node_resources;
            stored_node.location = node_location;
            stored_node.interfaces = node_interfaces;
            stored_node.secure_boot = secure_boot;
            stored_node.virtualized = virtualized;
            stored_node.serial_number = node_serial_number;

            // override node in storage
            Nodes::<T>::insert(stored_node.id, &stored_node);

            T::NodeChanged::node_changed(Some(&old_node), &stored_node);

            Self::deposit_event(Event::NodeUpdated(stored_node));

            Ok(Pays::No.into())
        }

        #[pallet::call_index(10)]
        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(1).ref_time() + T::DbWeight::get().reads(1).ref_time())]
        pub fn set_node_certification(
            origin: OriginFor<T>,
            node_id: u32,
            node_certification: NodeCertification,
        ) -> DispatchResultWithPostInfo {
            // Only council/root or allowed certifiers can modify node certification
            if !T::RestrictedOrigin::ensure_origin(origin.clone()).is_ok() {
                let account_id = ensure_signed(origin)?;
                if !AllowedNodeCertifiers::<T>::get()
                    .unwrap_or(vec![])
                    .contains(&account_id)
                {
                    return Err(Error::<T>::NotAllowedToCertifyNode.into());
                }
            }

            let mut stored_node = Nodes::<T>::get(node_id).ok_or(Error::<T>::NodeNotExists)?;

            stored_node.certification = node_certification;

            let current_node_policy = FarmingPoliciesMap::<T>::get(stored_node.farming_policy_id);
            if current_node_policy.default {
                // Refetch farming policy and save it on the node
                let farming_policy = Self::get_farming_policy(&stored_node)?;
                stored_node.farming_policy_id = farming_policy.id;
            }

            // override node in storage
            Nodes::<T>::insert(stored_node.id, &stored_node);

            Self::deposit_event(Event::NodeUpdated(stored_node));
            Self::deposit_event(Event::NodeCertificationSet(node_id, node_certification));

            Ok(().into())
        }

        #[pallet::call_index(11)]
        #[pallet::weight(<T as Config>::WeightInfo::report_uptime())]
        pub fn report_uptime(origin: OriginFor<T>, uptime: u64) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;

            let twin_id =
                TwinIdByAccountID::<T>::get(account_id).ok_or(Error::<T>::TwinNotExists)?;

            ensure!(
                NodeIdByTwinID::<T>::contains_key(twin_id),
                Error::<T>::NodeNotExists
            );
            let node_id = NodeIdByTwinID::<T>::get(twin_id);

            ensure!(Nodes::<T>::contains_key(node_id), Error::<T>::NodeNotExists);

            let now = <timestamp::Pallet<T>>::get().saturated_into::<u64>() / 1000;

            Self::deposit_event(Event::NodeUptimeReported(node_id, now, uptime));

            Ok(Pays::No.into())
        }

        #[pallet::call_index(12)]
        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(1).ref_time() + T::DbWeight::get().reads(3).ref_time())]
        pub fn add_node_public_config(
            origin: OriginFor<T>,
            farm_id: u32,
            node_id: u32,
            public_config: Option<PublicConfig>,
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

            if let Some(config) = public_config {
                config
                    .is_valid()
                    .map_err(|_| Error::<T>::InvalidPublicConfig)?;
                // update the public config and save
                node.public_config = Some(config);
            } else {
                node.public_config = None;
            }

            Nodes::<T>::insert(node_id, &node);
            Self::deposit_event(Event::NodePublicConfigStored(node_id, node.public_config));

            Ok(().into())
        }

        #[pallet::call_index(13)]
        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(1).ref_time() + T::DbWeight::get().reads(2).ref_time())]
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

            Ok(().into())
        }

        #[pallet::call_index(14)]
        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(4).ref_time() + T::DbWeight::get().reads(3).ref_time())]
        pub fn create_entity(
            origin: OriginFor<T>,
            target: T::AccountId,
            name: Vec<u8>,
            country: CountryNameInput,
            city: CityNameInput,
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
                Error::<T>::SignatureLengthIsIncorrect
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

            let entity = TfgridEntity::<T> {
                version: TFGRID_ENTITY_VERSION,
                id,
                name: name.clone(),
                country: Self::get_country_name(country)?,
                city: Self::get_city_name(city)?,
                account_id: target.clone(),
            };

            Entities::<T>::insert(&id, &entity);
            EntityIdByName::<T>::insert(&name, id);
            EntityIdByAccountID::<T>::insert(&target, id);
            EntityID::<T>::put(id);

            Self::deposit_event(Event::EntityStored(entity));

            Ok(().into())
        }

        #[pallet::call_index(15)]
        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(3).ref_time() + T::DbWeight::get().reads(3).ref_time())]
        pub fn update_entity(
            origin: OriginFor<T>,
            name: Vec<u8>,
            country: CountryNameInput,
            city: CityNameInput,
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
            stored_entity.country = Self::get_country_name(country)?;
            stored_entity.city = Self::get_city_name(city)?;

            // overwrite entity
            Entities::<T>::insert(&stored_entity_id, &stored_entity);

            // re-insert with new name
            EntityIdByName::<T>::insert(&name, stored_entity_id);

            Self::deposit_event(Event::EntityUpdated(stored_entity));

            Ok(().into())
        }

        // TODO: delete all object that have an entity id reference?
        #[pallet::call_index(16)]
        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(3).ref_time() + T::DbWeight::get().reads(2).ref_time())]
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

        #[pallet::call_index(17)]
        #[pallet::weight(<T as Config>::WeightInfo::create_twin())]
        pub fn create_twin(
            origin: OriginFor<T>,
            relay: RelayInput,
            pk: PkInput,
        ) -> DispatchResultWithPostInfo {
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

            if let Some(relay_addr) = relay.clone() {
                ensure!(
                    Self::validate_relay_address(relay_addr.into()),
                    Error::<T>::InvalidRelayAddress
                );
            }

            let twin = types::Twin::<T::AccountId> {
                id: twin_id,
                account_id: account_id.clone(),
                relay,
                entities: Vec::new(),
                pk,
            };

            Twins::<T>::insert(&twin_id, &twin);
            TwinID::<T>::put(twin_id);

            // add the twin id to this users map of twin ids
            TwinIdByAccountID::<T>::insert(&account_id.clone(), twin_id);

            Self::deposit_event(Event::TwinStored(twin));

            Ok(().into())
        }

        #[pallet::call_index(18)]
        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(1).ref_time() + T::DbWeight::get().reads(3).ref_time())]
        pub fn update_twin(
            origin: OriginFor<T>,
            relay: RelayInput,
            pk: PkInput,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;

            let twin_id =
                TwinIdByAccountID::<T>::get(account_id.clone()).ok_or(Error::<T>::TwinNotExists)?;

            let mut twin = Twins::<T>::get(&twin_id).ok_or(Error::<T>::TwinNotExists)?;

            // Make sure only the owner of this twin can update his twin
            ensure!(
                twin.account_id == account_id,
                Error::<T>::UnauthorizedToUpdateTwin
            );

            if let Some(relay_addr) = relay.clone() {
                ensure!(
                    Self::validate_relay_address(relay_addr.into()),
                    Error::<T>::InvalidRelayAddress
                );
            }

            twin.relay = relay;
            twin.pk = pk;

            Twins::<T>::insert(&twin_id, &twin);

            Self::deposit_event(Event::TwinUpdated(twin));
            Ok(().into())
        }

        // Method for twins only
        #[pallet::call_index(19)]
        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(1).ref_time() + T::DbWeight::get().reads(2).ref_time())]
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

        #[pallet::call_index(20)]
        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(1).ref_time() + T::DbWeight::get().reads(1).ref_time())]
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

        #[pallet::call_index(22)]
        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(3).ref_time() + T::DbWeight::get().reads(2).ref_time())]
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

        #[pallet::call_index(23)]
        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(4).ref_time() + T::DbWeight::get().reads(2).ref_time())]
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

        #[pallet::call_index(24)]
        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(2).ref_time() + T::DbWeight::get().reads(3).ref_time())]
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

        #[pallet::call_index(25)]
        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(1).ref_time() + T::DbWeight::get().reads(2).ref_time())]
        pub fn user_accept_tc(
            origin: OriginFor<T>,
            document_link: DocumentLinkInput,
            document_hash: DocumentHashInput,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            let timestamp = <timestamp::Pallet<T>>::get().saturated_into::<u64>() / 1000;

            let input = TermsAndConditionsInput::<T> {
                account_id: account_id.clone(),
                timestamp,
                document_link,
                document_hash,
            };

            let t_and_c = Self::get_terms_and_conditions(input)?;

            let mut users_terms_and_condition =
                UsersTermsAndConditions::<T>::get(account_id.clone()).unwrap_or(vec![]);
            users_terms_and_condition.push(t_and_c);
            UsersTermsAndConditions::<T>::insert(account_id, users_terms_and_condition);

            Ok(().into())
        }

        #[pallet::call_index(26)]
        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(2).ref_time() + T::DbWeight::get().reads(5).ref_time())]
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

            Self::deposit_event(Event::NodeDeleted(node_id));

            Ok(().into())
        }

        #[pallet::call_index(27)]
        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(3).ref_time() + T::DbWeight::get().reads(2).ref_time())]
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

        #[pallet::call_index(28)]
        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(1).ref_time() + T::DbWeight::get().reads(1).ref_time())]
        pub fn force_reset_farm_ip(
            origin: OriginFor<T>,
            farm_id: u32,
            ip: Ip4Input,
        ) -> DispatchResultWithPostInfo {
            T::RestrictedOrigin::ensure_origin(origin)?;

            ensure!(Farms::<T>::contains_key(farm_id), Error::<T>::FarmNotExists);
            let mut stored_farm = Farms::<T>::get(farm_id).ok_or(Error::<T>::FarmNotExists)?;

            match stored_farm
                .public_ips
                .iter_mut()
                .find(|pubip| pubip.ip == ip)
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

        #[pallet::call_index(29)]
        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(1).ref_time() + T::DbWeight::get().reads(1).ref_time())]
        pub fn set_connection_price(
            origin: OriginFor<T>,
            price: u32,
        ) -> DispatchResultWithPostInfo {
            T::RestrictedOrigin::ensure_origin(origin)?;

            ConnectionPrice::<T>::set(price);

            Self::deposit_event(Event::ConnectionPriceSet(price));

            Ok(().into())
        }

        #[pallet::call_index(30)]
        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(1).ref_time() + T::DbWeight::get().reads(1).ref_time())]
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

        #[pallet::call_index(31)]
        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(1).ref_time() + T::DbWeight::get().reads(1).ref_time())]
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

        #[pallet::call_index(32)]
        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(1).ref_time() + T::DbWeight::get().reads(1).ref_time())]
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

        #[pallet::call_index(33)]
        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(1).ref_time() + T::DbWeight::get().reads(1).ref_time())]
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

        #[pallet::call_index(34)]
        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(1).ref_time() + T::DbWeight::get().reads(1).ref_time())]
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

        #[pallet::call_index(35)]
        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(1).ref_time() + T::DbWeight::get().reads(1).ref_time())]
        pub fn change_power_state(
            origin: OriginFor<T>,
            power_state: Power,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_change_power_state(account_id, power_state)
        }

        #[pallet::call_index(36)]
        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(1).ref_time() + T::DbWeight::get().reads(1).ref_time())]
        pub fn change_power_target(
            origin: OriginFor<T>,
            node_id: u32,
            power_target: Power,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_change_power_target(account_id, node_id, power_target)
        }
    }
}

// Internal functions of the pallet
impl<T: Config> Pallet<T> {
    pub fn verify_signature(signature: [u8; 64], target: &T::AccountId, payload: &Vec<u8>) -> bool {
        Self::verify_ed_signature(signature, target, payload)
            || Self::verify_sr_signature(signature, target, payload)
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
        node: &TfgridNode<T>,
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
                        let cu = node.resources.get_cu();
                        if cu > cu_limit {
                            return Self::get_default_farming_policy();
                        }
                        limits.cu = Some(cu_limit - cu);
                    }
                    None => (),
                };

                match limits.su {
                    Some(su_limit) => {
                        let su = node.resources.get_su();
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

    fn get_terms_and_conditions(
        terms_cond: TermsAndConditionsInput<T>,
    ) -> Result<TermsAndConditionsOf<T>, DispatchErrorWithPostInfo> {
        let parsed_terms_cond = <T as Config>::TermsAndConditions::try_from(terms_cond)?;
        Ok(parsed_terms_cond)
    }

    fn get_farm_name(name: FarmNameInput<T>) -> Result<FarmNameOf<T>, DispatchErrorWithPostInfo> {
        let name_parsed = <T as Config>::FarmName::try_from(name)?;
        Ok(name_parsed)
    }

    fn get_public_ips(
        public_ips: PublicIpListInput<T>,
    ) -> Result<PublicIpListOf, DispatchErrorWithPostInfo> {
        let mut public_ips_list: PublicIpListOf =
            vec![].try_into().map_err(|_| Error::<T>::InvalidPublicIP)?;

        for ip in public_ips {
            let pub_ip = PublicIP {
                ip: ip.ip,
                gateway: ip.gw,
                contract_id: 0,
            };

            if public_ips_list.contains(&pub_ip) {
                return Err(DispatchErrorWithPostInfo::from(Error::<T>::IpExists));
            }

            public_ips_list
                .try_push(pub_ip)
                .map_err(|_| Error::<T>::InvalidPublicIP)?;
        }

        Ok(public_ips_list)
    }

    pub fn get_resources(
        resources: pallet::ResourcesInput,
    ) -> Result<Resources, DispatchErrorWithPostInfo> {
        ensure!(resources.validate_hru(), Error::<T>::InvalidHRUInput);
        ensure!(resources.validate_sru(), Error::<T>::InvalidSRUInput);
        ensure!(resources.validate_cru(), Error::<T>::InvalidCRUInput);
        ensure!(resources.validate_mru(), Error::<T>::InvalidMRUInput);

        Ok(resources)
    }

    fn get_interface_name(
        if_name: InterfaceNameInput,
    ) -> Result<InterfaceNameOf<T>, DispatchErrorWithPostInfo> {
        let if_name_parsed = <T as Config>::InterfaceName::try_from(if_name)?;
        Ok(if_name_parsed)
    }

    fn get_interface_mac(
        if_mac: InterfaceMacInput,
    ) -> Result<InterfaceMacOf<T>, DispatchErrorWithPostInfo> {
        let if_mac_parsed = <T as Config>::InterfaceMac::try_from(if_mac)?;
        Ok(if_mac_parsed)
    }

    fn get_interface_ip(
        if_ip: InterfaceIpInput,
    ) -> Result<InterfaceIpOf<T>, DispatchErrorWithPostInfo> {
        let if_ip_parsed = <T as Config>::InterfaceIP::try_from(if_ip)?;
        Ok(if_ip_parsed)
    }

    fn get_interfaces(
        interfaces: &InterfaceInput<T>,
    ) -> Result<Vec<InterfaceOf<T>>, DispatchErrorWithPostInfo> {
        let mut parsed_interfaces = Vec::new();
        if interfaces.len() == 0 {
            return Ok(parsed_interfaces);
        }

        for intf in interfaces.iter() {
            let intf_name = Self::get_interface_name(intf.name.clone())?;
            let intf_mac = Self::get_interface_mac(intf.mac.clone())?;

            let mut parsed_interfaces_ips: BoundedVec<
                InterfaceIpOf<T>,
                <T as Config>::MaxInterfaceIpsLength,
            > = vec![]
                .try_into()
                .map_err(|_| Error::<T>::InvalidInterfaceIP)?;

            for ip in intf.ips.iter() {
                let intf_ip = Self::get_interface_ip(ip.clone())?;
                parsed_interfaces_ips
                    .try_push(intf_ip)
                    .map_err(|_| Error::<T>::InvalidInterfaceIP)?;
            }

            parsed_interfaces.push(Interface {
                name: intf_name,
                mac: intf_mac,
                ips: parsed_interfaces_ips,
            });
        }

        Ok(parsed_interfaces)
    }

    pub fn get_city_name(city: CityNameInput) -> Result<CityNameOf<T>, DispatchErrorWithPostInfo> {
        let parsed_city = <T as Config>::CityName::try_from(city)?;
        Ok(parsed_city)
    }

    pub fn get_country_name(
        country: CountryNameInput,
    ) -> Result<CountryNameOf<T>, DispatchErrorWithPostInfo> {
        let parsed_country = <T as Config>::CountryName::try_from(country)?;
        Ok(parsed_country)
    }

    pub fn get_location(
        location: pallet::LocationInput,
    ) -> Result<LocationOf<T>, DispatchErrorWithPostInfo> {
        let parsed_location = <T as Config>::Location::try_from(location)?;
        Ok(parsed_location)
    }

    fn get_serial_number(
        serial_number: pallet::SerialNumberInput,
    ) -> Result<SerialNumberOf<T>, DispatchErrorWithPostInfo> {
        let parsed_serial_number = <T as Config>::SerialNumber::try_from(serial_number)?;
        Ok(parsed_serial_number)
    }

    fn _change_power_state(
        account_id: T::AccountId,
        power_state: Power,
    ) -> DispatchResultWithPostInfo {
        let twin_id = TwinIdByAccountID::<T>::get(&account_id).ok_or(Error::<T>::TwinNotExists)?;
        ensure!(
            NodeIdByTwinID::<T>::contains_key(twin_id),
            Error::<T>::NodeNotExists
        );
        let node_id = NodeIdByTwinID::<T>::get(twin_id);
        let node = Nodes::<T>::get(node_id).ok_or(Error::<T>::NodeNotExists)?;

        let power_state = match power_state {
            Power::Up => PowerState::Up,
            Power::Down => PowerState::Down(system::Pallet::<T>::block_number()),
        };

        let mut node_power = NodePower::<T>::get(node_id);

        // if the power state is not correct => change it and emit event
        if node_power.state != power_state {
            node_power.state = power_state.clone();

            NodePower::<T>::insert(node_id, node_power);
            Self::deposit_event(Event::PowerStateChanged {
                farm_id: node.farm_id,
                node_id,
                power_state,
            });
        }

        Ok(Pays::No.into())
    }

    fn _change_power_target(
        account_id: T::AccountId,
        node_id: u32,
        power_target: Power,
    ) -> DispatchResultWithPostInfo {
        let twin_id = TwinIdByAccountID::<T>::get(&account_id).ok_or(Error::<T>::TwinNotExists)?;
        let node = Nodes::<T>::get(node_id).ok_or(Error::<T>::NodeNotExists)?;
        let farm = Farms::<T>::get(node.farm_id).ok_or(Error::<T>::FarmNotExists)?;
        ensure!(
            twin_id == farm.twin_id,
            Error::<T>::UnauthorizedToChangePowerTarget
        );
        // Make sure only the farmer that owns this node can change the power target
        ensure!(
            node.farm_id == farm.id,
            Error::<T>::UnauthorizedToChangePowerTarget
        );

        Self::_change_power_target_on_node(node.id, node.farm_id, power_target);

        Ok(().into())
    }

    fn _change_power_target_on_node(node_id: u32, farm_id: u32, power_target: Power) {
        let mut node_power = NodePower::<T>::get(node_id);
        node_power.target = power_target.clone();
        NodePower::<T>::insert(node_id, &node_power);

        Self::deposit_event(Event::PowerTargetChanged {
            farm_id,
            node_id,
            power_target,
        });
    }

    fn validate_relay_address(relay_input: Vec<u8>) -> bool {
        if relay_input.len() == 0 {
            return false;
        }

        if relay_input[relay_input.len() - 1] == b'.' {
            return false;
        }

        let mut prev_idx = 0;

        for (idx, c) in relay_input.iter().enumerate() {
            match c {
                b'.' => {
                    if idx == 0 || idx - prev_idx == 1 {
                        return false;
                    } else {
                        prev_idx = idx
                    }
                }
                b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' => (),
                _ => return false,
            }
        }

        true
    }
}

impl<T: Config> tfchain_support::traits::Tfgrid<T::AccountId, T::FarmName> for Pallet<T> {
    fn get_farm(farm_id: u32) -> Option<tfchain_support::types::Farm<T::FarmName>> {
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
