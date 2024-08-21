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

pub mod v12 {
    use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
    use scale_info::TypeInfo;

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
}
