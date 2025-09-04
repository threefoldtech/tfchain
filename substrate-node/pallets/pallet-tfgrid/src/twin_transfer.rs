use crate::*;
use frame_support::{dispatch::DispatchResultWithPostInfo, ensure};
use frame_system::ensure_signed;
use sp_runtime::traits::{Saturating, Zero};
use sp_runtime::SaturatedConversion;
use tfchain_support::constants::time::HOURS;
use frame_support::traits::{BalanceStatus, ReservableCurrency};

impl<T: Config> Pallet<T> {
    pub fn _request_twin_transfer(
        origin: T::RuntimeOrigin,
        twin_id: u32,
    ) -> DispatchResultWithPostInfo {
        // Derive the new account directly from the signer
        let new_account = ensure_signed(origin)?;

        // Twin must exist
        let twin = Twins::<T>::get(&twin_id).ok_or(Error::<T>::TwinNotExists)?;

        // New account must have signed T&C
        ensure!(
            UsersTermsAndConditions::<T>::contains_key(new_account.clone()),
            Error::<T>::UserDidNotSignTermsAndConditions
        );

        // New account must not already have a twin
        ensure!(
            !TwinIdByAccountID::<T>::contains_key(&new_account),
            Error::<T>::TwinTransferNewAccountHasTwin
        );

        // Only one pending transfer per twin
        if let Some(existing_id) = PendingTransferByTwin::<T>::get(&twin_id) {
            if let Some(existing) = TwinTransferRequests::<T>::get(existing_id) {
                ensure!(
                    existing.status != TransferStatus::Pending,
                    Error::<T>::TwinTransferPendingExists
                );
            }
        }

        // Create request
        let mut req_id = TwinTransferRequestID::<T>::get();
        req_id = req_id.saturating_add(1);
        TwinTransferRequestID::<T>::put(req_id);

        let expiry_block = frame_system::Pallet::<T>::block_number()
            .saturating_add(HOURS.saturated_into());

        let request = TwinTransferRequest::<T> {
            twin_id,
            old_account: twin.account_id.clone(),
            new_account: new_account.clone(),
            expiry_block,
            status: TransferStatus::Pending,
        };

        TwinTransferRequests::<T>::insert(req_id, &request);
        PendingTransferByTwin::<T>::insert(twin_id, req_id);

        Self::deposit_event(Event::TwinTransferRequested {
            twin_id,
            old_account: request.old_account,
            new_account,
        });

        Ok(().into())
    }

    pub fn _accept_twin_transfer(
        origin: T::RuntimeOrigin,
        request_id: u64,
    ) -> DispatchResultWithPostInfo {
        let signer = ensure_signed(origin)?;

        let mut req = TwinTransferRequests::<T>::get(request_id)
            .ok_or(Error::<T>::TwinTransferRequestNotFound)?;

        ensure!(
            req.status == TransferStatus::Pending,
            Error::<T>::TwinTransferRequestAlreadyCompleted
        );

        let now = frame_system::Pallet::<T>::block_number();
        ensure!(now <= req.expiry_block, Error::<T>::TwinTransferRequestExpired);

        // Twin must exist and signer must be the current owner
        let mut twin = Twins::<T>::get(&req.twin_id).ok_or(Error::<T>::TwinNotExists)?;
        ensure!(twin.account_id == signer, Error::<T>::UnauthorizedToUpdateTwin);

        // New account must still not have a twin
        ensure!(
            !TwinIdByAccountID::<T>::contains_key(&req.new_account),
            Error::<T>::TwinTransferNewAccountHasTwin
        );

        // Move all reserved from old -> new as reserved
        let reserved = T::Currency::reserved_balance(&signer);
        if !reserved.is_zero() {
            let _ = T::Currency::repatriate_reserved(
                &signer,
                &req.new_account,
                reserved,
                BalanceStatus::Reserved,
            );
        }

        // Update twin ownership and indexes
        let old_account = signer.clone();
        twin.account_id = req.new_account.clone();
        Twins::<T>::insert(&req.twin_id, &twin);

        // Update account->twin mapping
        TwinIdByAccountID::<T>::remove(&old_account);
        TwinIdByAccountID::<T>::insert(&req.new_account, req.twin_id);

        // Mark request completed and clear pending index
        req.status = TransferStatus::Completed;
        TwinTransferRequests::<T>::insert(request_id, &req);
        PendingTransferByTwin::<T>::remove(req.twin_id);

        // Emit events
        Self::deposit_event(Event::TwinOwnershipTransferred {
            twin_id: req.twin_id,
            old_account: old_account.clone(),
            new_account: req.new_account.clone(),
        });
        Self::deposit_event(Event::TwinUpdated(twin));

        Ok(().into())
    }
}
