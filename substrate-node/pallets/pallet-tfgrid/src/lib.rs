#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

pub mod farm;
pub mod interface;
pub mod migrations;
pub mod node;
pub mod pricing;
pub mod terms_cond;
pub mod twin;
pub mod types;
pub mod weights;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

// Definition of the pallet logic, to be aggregated at runtime definition
// through `construct_runtime`.
#[frame_support::pallet]
pub mod pallet {
    use super::weights::WeightInfo;
    use super::*;
    use frame_support::{
        dispatch::DispatchResultWithPostInfo, ensure, pallet_prelude::*,
        storage::bounded_vec::BoundedVec, traits::ConstU32, traits::EnsureOrigin, Blake2_128Concat,
    };
    use frame_system::{ensure_signed, pallet_prelude::*};
    use parity_scale_codec::FullCodec;
    use sp_core::Get;
    use sp_runtime::SaturatedConversion;
    use sp_std::{convert::TryInto, fmt::Debug, vec, vec::Vec};
    use tfchain_support::{
        resources::Resources,
        traits::{ChangeNode, PublicIpModifier},
        types::*,
    };

    #[pallet::pallet]
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
    #[pallet::getter(fn twin_bonded_account)]
    pub type TwinBoundedAccountID<T: Config> =
        StorageMap<_, Blake2_128Concat, u32, T::AccountId, OptionQuery>;

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
    pub type NodePower<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u32,
        tfchain_support::types::NodePower<T::BlockNumber>,
        ValueQuery,
    >;

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

        #[pallet::constant]
        type TimestampHintDrift: Get<u64>;
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
        TwinAccountBounded(u32, T::AccountId),

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
        TwinCannotBoundToItself,

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
        InvalidTimestampHint,
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
        #[pallet::weight(<T as Config>::WeightInfo::set_storage_version())]
        pub fn set_storage_version(
            origin: OriginFor<T>,
            version: types::StorageVersion,
        ) -> DispatchResultWithPostInfo {
            T::RestrictedOrigin::ensure_origin(origin)?;
            PalletVersion::<T>::set(version);
            Ok(().into())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::create_farm())]
        pub fn create_farm(
            origin: OriginFor<T>,
            name: FarmNameInput<T>,
            public_ips: PublicIpListInput<T>,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_create_farm(account_id, name, public_ips)
        }

        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::update_farm())]
        pub fn update_farm(
            origin: OriginFor<T>,
            farm_id: u32,
            name: FarmNameInput<T>,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_update_farm(account_id, farm_id, name)
        }

        #[pallet::call_index(3)]
        #[pallet::weight(<T as Config>::WeightInfo::add_stellar_payout_v2address())]
        pub fn add_stellar_payout_v2address(
            origin: OriginFor<T>,
            farm_id: u32,
            stellar_address: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_add_stellar_payout_v2address(account_id, farm_id, stellar_address)
        }

        #[pallet::call_index(4)]
        #[pallet::weight(<T as Config>::WeightInfo::set_farm_certification())]
        pub fn set_farm_certification(
            origin: OriginFor<T>,
            farm_id: u32,
            certification: FarmCertification,
        ) -> DispatchResultWithPostInfo {
            T::RestrictedOrigin::ensure_origin(origin)?;
            Self::_set_farm_certification(farm_id, certification)
        }

        #[pallet::call_index(5)]
        #[pallet::weight(<T as Config>::WeightInfo::add_farm_ip())]
        pub fn add_farm_ip(
            origin: OriginFor<T>,
            farm_id: u32,
            ip: Ip4Input,
            gw: Gw4Input,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_add_farm_ip(account_id, farm_id, ip, gw)
        }

        #[pallet::call_index(6)]
        #[pallet::weight(<T as Config>::WeightInfo::remove_farm_ip())]
        pub fn remove_farm_ip(
            origin: OriginFor<T>,
            farm_id: u32,
            ip: Ip4Input,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_remove_farm_ip(account_id, farm_id, ip)
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
            Self::_create_node(
                &account_id,
                farm_id,
                resources,
                location,
                interfaces,
                secure_boot,
                virtualized,
                serial_number,
            )
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
            Self::_update_node(
                &account_id,
                node_id,
                farm_id,
                resources,
                location,
                interfaces,
                secure_boot,
                virtualized,
                serial_number,
            )
        }

        #[pallet::call_index(10)]
        #[pallet::weight(<T as Config>::WeightInfo::set_node_certification())]
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
            Self::_set_node_certification(node_id, node_certification)
        }

        #[pallet::call_index(11)]
        #[pallet::weight(<T as Config>::WeightInfo::report_uptime())]
        pub fn report_uptime(origin: OriginFor<T>, uptime: u64) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            let timestamp_hint =
                <pallet_timestamp::Pallet<T>>::get().saturated_into::<u64>() / 1000;
            Self::_report_uptime(&account_id, uptime, timestamp_hint)
        }

        #[pallet::call_index(12)]
        #[pallet::weight(<T as Config>::WeightInfo::add_node_public_config())]
        pub fn add_node_public_config(
            origin: OriginFor<T>,
            farm_id: u32,
            node_id: u32,
            public_config: Option<PublicConfig>,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_add_node_public_config(account_id, farm_id, node_id, public_config)
        }

        #[pallet::call_index(13)]
        #[pallet::weight(<T as Config>::WeightInfo::delete_node())]
        pub fn delete_node(origin: OriginFor<T>, node_id: u32) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_delete_node(&account_id, node_id)
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
            Self::_create_entity(target, name, country, city, signature)
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
            Self::_update_entity(account_id, name, country, city)
        }

        // TODO: delete all object that have an entity id reference?
        #[pallet::call_index(16)]
        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(3).ref_time() + T::DbWeight::get().reads(2).ref_time())]
        pub fn delete_entity(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_delete_entity(account_id)
        }

        #[pallet::call_index(17)]
        #[pallet::weight(<T as Config>::WeightInfo::create_twin())]
        pub fn create_twin(
            origin: OriginFor<T>,
            relay: RelayInput,
            pk: PkInput,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_create_twin(account_id, relay, pk)
        }

        #[pallet::call_index(18)]
        #[pallet::weight(<T as Config>::WeightInfo::update_twin())]
        pub fn update_twin(
            origin: OriginFor<T>,
            relay: RelayInput,
            pk: PkInput,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_update_twin(account_id, relay, pk)
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
            Self::_add_twin_entity(account_id, twin_id, entity_id, signature)
        }

        #[pallet::call_index(20)]
        #[pallet::weight(100_000_000 + T::DbWeight::get().writes(1).ref_time() + T::DbWeight::get().reads(1).ref_time())]
        pub fn delete_twin_entity(
            origin: OriginFor<T>,
            twin_id: u32,
            entity_id: u32,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_delete_twin_entity(account_id, twin_id, entity_id)
        }

        #[pallet::call_index(22)]
        #[pallet::weight(<T as Config>::WeightInfo::create_pricing_policy())]
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
            Self::_create_pricing_policy(
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
            )
        }

        #[pallet::call_index(23)]
        #[pallet::weight(<T as Config>::WeightInfo::update_pricing_policy())]
        pub fn update_pricing_policy(
            origin: OriginFor<T>,
            pricing_policy_id: u32,
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
            Self::_update_pricing_policy(
                pricing_policy_id,
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
            )
        }

        #[pallet::call_index(24)]
        #[pallet::weight(<T as Config>::WeightInfo::create_farming_policy())]
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
            Self::_create_farming_policy(
                name,
                su,
                cu,
                nu,
                ipv4,
                minimal_uptime,
                policy_end,
                immutable,
                default,
                node_certification,
                farm_certification,
            )
        }

        #[pallet::call_index(25)]
        #[pallet::weight(<T as Config>::WeightInfo::user_accept_tc())]
        pub fn user_accept_tc(
            origin: OriginFor<T>,
            document_link: DocumentLinkInput,
            document_hash: DocumentHashInput,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_user_accept_tc(account_id, document_link, document_hash)
        }

        #[pallet::call_index(26)]
        #[pallet::weight(<T as Config>::WeightInfo::delete_node_farm())]
        pub fn delete_node_farm(origin: OriginFor<T>, node_id: u32) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_delete_node_farm(account_id, node_id)
        }

        #[pallet::call_index(27)]
        #[pallet::weight(<T as Config>::WeightInfo::set_farm_dedicated())]
        pub fn set_farm_dedicated(
            origin: OriginFor<T>,
            farm_id: u32,
            dedicated: bool,
        ) -> DispatchResultWithPostInfo {
            T::RestrictedOrigin::ensure_origin(origin)?;
            Self::_set_farm_dedicated(farm_id, dedicated)
        }

        #[pallet::call_index(28)]
        #[pallet::weight(<T as Config>::WeightInfo::force_reset_farm_ip())]
        pub fn force_reset_farm_ip(
            origin: OriginFor<T>,
            farm_id: u32,
            ip: Ip4Input,
        ) -> DispatchResultWithPostInfo {
            T::RestrictedOrigin::ensure_origin(origin)?;
            Self::_force_reset_farm_ip(farm_id, ip)
        }

        #[pallet::call_index(29)]
        #[pallet::weight(<T as Config>::WeightInfo::set_connection_price())]
        pub fn set_connection_price(
            origin: OriginFor<T>,
            price: u32,
        ) -> DispatchResultWithPostInfo {
            T::RestrictedOrigin::ensure_origin(origin)?;
            Self::_set_connection_price(price)
        }

        #[pallet::call_index(30)]
        #[pallet::weight(<T as Config>::WeightInfo::add_node_certifier())]
        pub fn add_node_certifier(
            origin: OriginFor<T>,
            certifier: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            T::RestrictedOrigin::ensure_origin(origin)?;
            Self::_add_node_certifier(certifier)
        }

        #[pallet::call_index(31)]
        #[pallet::weight(<T as Config>::WeightInfo::remove_node_certifier())]
        pub fn remove_node_certifier(
            origin: OriginFor<T>,
            certifier: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            T::RestrictedOrigin::ensure_origin(origin)?;
            Self::_remove_node_certifier(certifier)
        }

        #[pallet::call_index(32)]
        #[pallet::weight(<T as Config>::WeightInfo::update_farming_policy())]
        pub fn update_farming_policy(
            origin: OriginFor<T>,
            farming_policy_id: u32,
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
            Self::_update_farming_policy(
                farming_policy_id,
                name,
                su,
                cu,
                nu,
                ipv4,
                minimal_uptime,
                policy_end,
                default,
                node_certification,
                farm_certification,
            )
        }

        #[pallet::call_index(33)]
        #[pallet::weight(<T as Config>::WeightInfo::attach_policy_to_farm())]
        pub fn attach_policy_to_farm(
            origin: OriginFor<T>,
            farm_id: u32,
            limits: Option<FarmingPolicyLimit>,
        ) -> DispatchResultWithPostInfo {
            T::RestrictedOrigin::ensure_origin(origin)?;
            Self::_attach_policy_to_farm(farm_id, limits)
        }

        #[pallet::call_index(34)]
        #[pallet::weight(<T as Config>::WeightInfo::set_zos_version())]
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
        #[pallet::weight(<T as Config>::WeightInfo::change_power_state())]
        pub fn change_power_state(
            origin: OriginFor<T>,
            power_state: Power,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_change_power_state(&account_id, power_state)
        }

        #[pallet::call_index(36)]
        #[pallet::weight(<T as Config>::WeightInfo::change_power_target())]
        pub fn change_power_target(
            origin: OriginFor<T>,
            node_id: u32,
            power_target: Power,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_change_power_target(&account_id, node_id, power_target)
        }

        #[pallet::call_index(37)]
        #[pallet::weight(<T as Config>::WeightInfo::bond_twin_account())]
        pub fn bond_twin_account(origin: OriginFor<T>, twin_id: u32) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_bond_twin_account(account_id, twin_id)
        }

        #[pallet::call_index(38)]
        #[pallet::weight(<T as Config>::WeightInfo::report_uptime_v2())]
        pub fn report_uptime_v2(
            origin: OriginFor<T>,
            uptime: u64,
            timestamp_hint: u64,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_report_uptime(&account_id, uptime, timestamp_hint)
        }

        // Deprecated! Use index 40 for next extrinsic
        // #[pallet::call_index(39)]
        // #[pallet::weight(<T as Config>::WeightInfo::set_node_gpu_status())]
    }
}