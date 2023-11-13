pub mod v1 {
    use frame_support::{storage_alias, StorageMap, Blake2_128Concat, pallet_prelude::OptionQuery};
    use parity_scale_codec::{Decode, Encode};
    use scale_info::{TypeInfo, prelude::vec::Vec};

    use crate::{Config, Pallet, types::StellarSignature};

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
    pub type BurnTransactions<T: Config> =
    StorageMap<Pallet<T>, Blake2_128Concat, u64, BurnTransaction<<T as frame_system::Config>::BlockNumber>, OptionQuery>;

    #[cfg(feature = "try-runtime")]
    #[storage_alias]  
    pub type ExecutedBurnTransactions<T: Config> =
        StorageMap<Pallet<T>, Blake2_128Concat, u64, BurnTransaction<<T as frame_system::Config>::BlockNumber>, OptionQuery>;

}
