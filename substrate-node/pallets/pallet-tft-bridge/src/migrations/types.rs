pub mod v1 {
    use frame_support::{pallet_prelude::OptionQuery, storage_alias, Blake2_128Concat, StorageMap};
    use parity_scale_codec::{Decode, Encode};
    use scale_info::{prelude::vec::Vec, TypeInfo};

    use crate::{types::StellarSignature, Config, Pallet};

    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo)]
    pub struct BurnTransaction<BlockNumber> {
        pub block: BlockNumber,
        pub amount: u64,
        pub target: Vec<u8>,
        pub signatures: Vec<StellarSignature>,
        pub sequence_number: u64,
    }

    #[cfg(feature = "try-runtime")]
    #[storage_alias]
    pub type BurnTransactions<T: Config> = StorageMap<
        Pallet<T>,
        Blake2_128Concat,
        u64,
        BurnTransaction<<T as frame_system::Config>::BlockNumber>,
        OptionQuery,
    >;

    #[cfg(feature = "try-runtime")]
    #[storage_alias]
    pub type ExecutedBurnTransactions<T: Config> = StorageMap<
        Pallet<T>,
        Blake2_128Concat,
        u64,
        BurnTransaction<<T as frame_system::Config>::BlockNumber>,
        OptionQuery,
    >;
}
