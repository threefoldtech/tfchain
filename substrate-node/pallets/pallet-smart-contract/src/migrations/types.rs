pub mod v10 {
    use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
    use scale_info::TypeInfo;

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
    pub struct ContractLock<BalanceOf> {
        pub amount_locked: BalanceOf,
        pub lock_updated: u64,
        pub cycles: u16,
    }
}

pub mod v11 {
    use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
    use scale_info::TypeInfo;

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
    pub struct ContractLock<BalanceOf> {
        pub amount_locked: BalanceOf,
        pub extra_amount_locked: BalanceOf,
        pub lock_updated: u64,
        pub cycles: u16,
    }
}
