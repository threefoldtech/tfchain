pub mod v11 {
    use codec::{Decode, Encode};
    use core::cmp::{Ord, PartialOrd};
    use scale_info::TypeInfo;
    use sp_std::{prelude::*, vec::Vec};
    use tfchain_support::{
        resources::Resources,
        types::{NodeCertification, PublicConfig},
    };

    #[derive(Encode, Decode, Debug, Default, PartialEq, Eq, Clone, TypeInfo)]
    pub struct Entity<AccountId> {
        pub version: u32,
        pub id: u32,
        pub name: Vec<u8>,
        pub account_id: AccountId,
        pub country: Vec<u8>,
        pub city: Vec<u8>,
    }

    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo)]
    pub struct Location {
        pub longitude: Vec<u8>,
        pub latitude: Vec<u8>,
    }

    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo)]
    pub struct Node<If> {
        pub version: u32,
        pub id: u32,
        pub farm_id: u32,
        pub twin_id: u32,
        pub resources: Resources,
        pub location: Location,
        pub country: Vec<u8>,
        pub city: Vec<u8>,
        pub public_config: Option<PublicConfig>,
        pub created: u64,
        pub farming_policy_id: u32,
        pub interfaces: Vec<If>,
        pub certification: NodeCertification,
        pub secure_boot: bool,
        pub virtualized: bool,
        pub serial_number: Vec<u8>,
        pub connection_price: u32,
    }
}

pub mod v12 {
    use codec::{Decode, Encode};
    use core::cmp::{Ord, PartialOrd};
    use scale_info::TypeInfo;
    use sp_std::{prelude::*, vec::Vec};
    use tfchain_support::{
        resources::Resources,
        types::{NodeCertification, PublicConfig},
    };

    #[derive(Encode, Decode, Debug, Default, PartialEq, Eq, Clone, TypeInfo)]
    pub struct Entity<AccountId, City, Country> {
        pub version: u32,
        pub id: u32,
        pub name: Vec<u8>,
        pub account_id: AccountId,
        pub country: Country,
        pub city: City,
    }

    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo)]
    pub struct Node<Location, If, SerialNumber> {
        pub version: u32,
        pub id: u32,
        pub farm_id: u32,
        pub twin_id: u32,
        pub resources: Resources,
        pub location: Location,
        pub public_config: Option<PublicConfig>,
        pub created: u64,
        pub farming_policy_id: u32,
        pub interfaces: Vec<If>,
        pub certification: NodeCertification,
        pub secure_boot: bool,
        pub virtualized: bool,
        pub serial_number: Option<SerialNumber>,
        pub connection_price: u32,
    }
}

pub mod v13 {
    use codec::{Decode, Encode, MaxEncodedLen};
    use core::cmp::{Ord, PartialOrd};
    use frame_support::{traits::ConstU32, BoundedVec};
    use scale_info::TypeInfo;
    use sp_std::{prelude::*, vec::Vec};
    use tfchain_support::types::{
        FarmCertification, FarmingPolicyLimit, NodeCertification, PublicConfig, MAX_GW4_LENGTH,
        MAX_IP4_LENGTH,
    };

    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo)]
    pub struct Node<Location, If, SerialNumber> {
        pub version: u32,
        pub id: u32,
        pub farm_id: u32,
        pub twin_id: u32,
        pub location: Location,
        pub public_config: Option<PublicConfig>,
        pub created: u64,
        pub farming_policy_id: u32,
        pub interfaces: Vec<If>,
        pub certification: NodeCertification,
        pub secure_boot: bool,
        pub virtualized: bool,
        pub serial_number: Option<SerialNumber>,
        pub connection_price: u32,
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
        Debug,
        TypeInfo,
        MaxEncodedLen,
    )]
    pub struct PublicIP {
        pub ip: BoundedVec<u8, ConstU32<MAX_IP4_LENGTH>>,
        pub gateway: BoundedVec<u8, ConstU32<MAX_GW4_LENGTH>>,
        pub contract_id: u64,
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
        Debug,
        TypeInfo,
        MaxEncodedLen,
    )]
    pub struct Farm<Name> {
        pub version: u32,
        pub id: u32,
        pub name: Name,
        pub twin_id: u32,
        pub pricing_policy_id: u32,
        pub certification: FarmCertification,
        pub public_ips: BoundedVec<PublicIP, ConstU32<256>>,
        pub dedicated_farm: bool,
        pub farming_policy_limits: Option<FarmingPolicyLimit>,
    }
}
