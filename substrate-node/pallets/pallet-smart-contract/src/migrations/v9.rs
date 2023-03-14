use crate::*;
use frame_support::{traits::OnRuntimeUpgrade, weights::Weight};
use log::{debug, info};
use sp_std::marker::PhantomData;

#[cfg(feature = "try-runtime")]
use codec::{Decode, Encode};
#[cfg(feature = "try-runtime")]
use sp_std::vec::Vec;

pub struct CleanBillingLoop<T: Config>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for CleanBillingLoop<T> {
    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
        info!("current pallet version: {:?}", PalletVersion::<T>::get());
        assert!(PalletVersion::<T>::get() >= types::StorageVersion::V8);

        let contracts_to_bill_count: u64 = ContractsToBillAt::<T>::iter().count() as u64;
        info!(
            "🔎 CleanBillingLoop pre migration: Number of existing billing loop slots {:?}",
            contracts_to_bill_count
        );

        info!("👥  Smart Contract pallet to V9 passes PRE migrate checks ✅",);
        Ok(contracts_to_bill_count.encode())
    }

    fn on_runtime_upgrade() -> Weight {
        migrate_to_version_9::<T>()
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(pre_contracts_to_bill_count: Vec<u8>) -> Result<(), &'static str> {
        info!("current pallet version: {:?}", PalletVersion::<T>::get());
        assert!(PalletVersion::<T>::get() >= types::StorageVersion::V9);

        // Check number of billing loop slots against pre-check result
        let pre_contracts_to_bill_count: u64 = Decode::decode(
            &mut pre_contracts_to_bill_count.as_slice(),
        )
        .expect("the state parameter should be something that was generated by pre_upgrade");
        assert_eq!(
            ContractsToBillAt::<T>::iter().count() as u64,
            pre_contracts_to_bill_count,
            "Number of billing loop slots migrated does not match"
        );

        info!(
            "👥  Smart Contract pallet to {:?} passes POST migrate checks ✅",
            PalletVersion::<T>::get()
        );

        Ok(())
    }
}

pub fn migrate_to_version_9<T: Config>() -> frame_support::weights::Weight {
    if PalletVersion::<T>::get() == types::StorageVersion::V8 {
        info!(
            " >>> Starting contract pallet migration, pallet version: {:?}",
            PalletVersion::<T>::get()
        );

        let mut slot_count = 0;
        let billing_loop_size = BillingFrequency::<T>::get();
        let mut contract_count = vec![0; billing_loop_size as usize];
        let mut keep_count = vec![0; billing_loop_size as usize];
        let mut nobill_count = vec![0; billing_loop_size as usize];
        let mut rogue_count = vec![0; billing_loop_size as usize];

        // Collect ContractsToBillAt storage in memory
        let contracts_to_bill_at = ContractsToBillAt::<T>::iter().collect::<Vec<_>>();

        for (index, contract_ids) in contracts_to_bill_at {
            let mut new_contract_ids = Vec::new();
            for contract_id in contract_ids {
                contract_count[index as usize] += 1;
                if let Some(contract) = Contracts::<T>::get(contract_id) {
                    match contract.contract_type {
                        types::ContractData::NodeContract(node_contract) => {
                            let contract_resource = NodeContractResources::<T>::get(contract_id);
                            if node_contract.public_ips > 0 || !contract_resource.used.is_empty() {
                                new_contract_ids.push(contract.contract_id);
                                keep_count[index as usize] += 1;
                            } else {
                                debug!(
                                    "node contract with id {} is removed from billing loop [no pub ips + no resources]",
                                    contract_id
                                );
                                nobill_count[index as usize] += 1;
                            }
                        }
                        _ => {
                            new_contract_ids.push(contract.contract_id);
                            keep_count[index as usize] += 1;
                        }
                    };
                } else {
                    debug!(
                        "contract with id {} is removed from billing loop [rogue contract]",
                        contract_id
                    );
                    rogue_count[index as usize] += 1;
                }
            }

            // Update ContractsToBillAt storage in memory
            slot_count += 1;
            ContractsToBillAt::<T>::insert(index, new_contract_ids);
        }

        for i in 0..billing_loop_size {
            debug!(
                " # Billing loop slot {}: [total: {}, keep: {}, nobill: {}, rogue {}]",
                i,
                contract_count[i as usize],
                keep_count[i as usize],
                nobill_count[i as usize],
                rogue_count[i as usize],
            );
            assert_eq!(
                contract_count[i as usize],
                keep_count[i as usize] + nobill_count[i as usize] + rogue_count[i as usize]
            );
        }

        info!(" <<< ContractsToBillAt storage updated! ✅");
        info!(
            " <<< There are {} contracts in billing loop",
            contract_count.iter().sum::<u64>()
        );
        info!(
            " <<< Kept {} contracts in billing loop",
            keep_count.iter().sum::<u64>()
        );
        info!(
            " <<< Removed {} contracts from billing loop",
            nobill_count.iter().sum::<u64>() + rogue_count.iter().sum::<u64>()
        );
        info!(" <<< Migrated {} billing loop slots", slot_count);

        // Update pallet storage version
        PalletVersion::<T>::set(types::StorageVersion::V9);
        info!(" <<< Storage version upgraded");

        // Return the weight consumed by the migration.
        T::DbWeight::get().reads_writes(slot_count, slot_count + 1)
    } else {
        info!(" >>> Unused Smart Contract pallet V9 migration");
        Weight::zero()
    }
}
