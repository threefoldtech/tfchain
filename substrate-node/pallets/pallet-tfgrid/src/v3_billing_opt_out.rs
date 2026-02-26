use crate::pallet::{
    AllowedTwinAdmins, Error, Event, Farms, NodeV3BillingOptOut, NodeV3OptOutMetadata, Nodes,
    TwinIdByAccountID,
};
use crate::Config;
use frame_support::{dispatch::DispatchResultWithPostInfo, ensure, BoundedVec};
use sp_runtime::SaturatedConversion;
use sp_std::prelude::Vec;

impl<T: Config> crate::Pallet<T> {
    pub fn _opt_out_of_v3_billing(
        account_id: T::AccountId,
        node_id: u32,
    ) -> DispatchResultWithPostInfo {
        let caller_twin_id =
            TwinIdByAccountID::<T>::get(&account_id).ok_or(Error::<T>::TwinNotExists)?;

        let node = Nodes::<T>::get(node_id).ok_or(Error::<T>::NodeNotExists)?;

        let farm = Farms::<T>::get(node.farm_id).ok_or(Error::<T>::FarmNotExists)?;

        ensure!(
            caller_twin_id == farm.twin_id,
            Error::<T>::NodeUpdateNotAuthorized
        );

        ensure!(
            !NodeV3BillingOptOut::<T>::contains_key(node_id),
            Error::<T>::NodeV3BillingOptOutAlreadyEnabled
        );

        let now: u64 =
            <pallet_timestamp::Pallet<T>>::get().saturated_into::<u64>() / 1000;

        NodeV3BillingOptOut::<T>::insert(node_id, now);

        Self::deposit_event(Event::NodeV3BillingOptedOut {
            node_id,
            opted_out_at: now,
        });

        Ok(().into())
    }

    pub fn _set_node_v3_opt_out_metadata(
        account_id: T::AccountId,
        node_id: u32,
        metadata: Vec<u8>,
    ) -> DispatchResultWithPostInfo {
        let caller_twin_id =
            TwinIdByAccountID::<T>::get(&account_id).ok_or(Error::<T>::TwinNotExists)?;

        let node = Nodes::<T>::get(node_id).ok_or(Error::<T>::NodeNotExists)?;

        let farm = Farms::<T>::get(node.farm_id).ok_or(Error::<T>::FarmNotExists)?;

        ensure!(
            caller_twin_id == farm.twin_id,
            Error::<T>::NodeUpdateNotAuthorized
        );

        ensure!(
            NodeV3BillingOptOut::<T>::contains_key(node_id),
            Error::<T>::NodeNotOptedOutOfV3Billing
        );

        if metadata.is_empty() {
            NodeV3OptOutMetadata::<T>::remove(node_id);
            Self::deposit_event(Event::NodeV3OptOutMetadataCleared { node_id });
        } else {
            let bounded: BoundedVec<u8, frame_support::traits::ConstU32<256>> =
                metadata.clone().try_into().map_err(|_| Error::<T>::NodeV3OptOutMetadataTooLong)?;
            NodeV3OptOutMetadata::<T>::insert(node_id, bounded);
            Self::deposit_event(Event::NodeV3OptOutMetadataSet { node_id, metadata });
        }

        Ok(().into())
    }

    pub fn _add_twin_admin(account: T::AccountId) -> DispatchResultWithPostInfo {
        let mut admins = AllowedTwinAdmins::<T>::get()
            .unwrap_or_else(|| BoundedVec::new());

        let location = admins
            .binary_search(&account)
            .err()
            .ok_or(Error::<T>::AlreadyTwinAdmin)?;

        admins
            .try_insert(location, account.clone())
            .map_err(|_| Error::<T>::TwinAdminListFull)?;

        AllowedTwinAdmins::<T>::put(admins);

        Self::deposit_event(Event::TwinAdminAdded(account));

        Ok(().into())
    }

    pub fn _remove_twin_admin(account: T::AccountId) -> DispatchResultWithPostInfo {
        let mut admins = AllowedTwinAdmins::<T>::get().ok_or(Error::<T>::NotTwinAdmin)?;
        let location = admins
            .binary_search(&account)
            .ok()
            .ok_or(Error::<T>::NotTwinAdmin)?;
        admins.remove(location);
        AllowedTwinAdmins::<T>::put(admins);

        Self::deposit_event(Event::TwinAdminRemoved(account));

        Ok(().into())
    }
}
