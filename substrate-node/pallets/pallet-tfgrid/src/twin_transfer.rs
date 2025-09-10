use crate::*;
use frame_support::traits::{BalanceStatus, ReservableCurrency};
use frame_support::{dispatch::DispatchResultWithPostInfo, ensure};
use frame_system::ensure_signed;
use sp_runtime::traits::Zero;

impl<T: Config> Pallet<T> {
    pub fn _request_twin_transfer(
        origin: T::RuntimeOrigin,
        new_account: T::AccountId,
    ) -> DispatchResultWithPostInfo {
        // Old owner is the signer
        let old_account = ensure_signed(origin)?;

        // Derive twin_id from old owner
        let twin_id = TwinIdByAccountID::<T>::get(&old_account).ok_or(Error::<T>::TwinNotExists)?;
        let twin = Twins::<T>::get(twin_id).ok_or(Error::<T>::TwinNotExists)?;
        // pure defensive check for Stale/corrupted index
        ensure!(
            twin.account_id == old_account,
            Error::<T>::UnauthorizedToUpdateTwin
        );

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
        ensure!(
            PendingTransferByTwin::<T>::get(twin_id).is_none(),
            Error::<T>::TwinTransferPendingExists
        );

        // Create request (pending exists as long as entry is present)
        let mut req_id = TwinTransferRequestID::<T>::get();
        req_id = req_id.saturating_add(1);
        TwinTransferRequestID::<T>::put(req_id);

        let request = TwinTransferRequest::<T> {
            twin_id,
            old_account: old_account.clone(),
            new_account: new_account.clone(),
        };

        TwinTransferRequests::<T>::insert(req_id, &request);
        PendingTransferByTwin::<T>::insert(twin_id, req_id);

        Self::deposit_event(Event::TwinTransferRequested {
            twin_id,
            old_account,
            new_account,
        });

        Ok(().into())
    }

    pub fn _accept_twin_transfer(
        origin: T::RuntimeOrigin,
        request_id: u64,
    ) -> DispatchResultWithPostInfo {
        let signer = ensure_signed(origin)?;

        let req = TwinTransferRequests::<T>::get(request_id)
            .ok_or(Error::<T>::TwinTransferRequestNotFound)?;

        // Only the intended new account can accept
        ensure!(
            req.new_account == signer,
            Error::<T>::UnauthorizedToUpdateTwin
        );

        // Twin must exist and still be owned by old_account
        let mut twin = Twins::<T>::get(req.twin_id).ok_or(Error::<T>::TwinNotExists)?;
        ensure!(
            twin.account_id == req.old_account,
            Error::<T>::UnauthorizedToUpdateTwin
        );

        // New account must still not have a twin
        ensure!(
            !TwinIdByAccountID::<T>::contains_key(&req.new_account),
            Error::<T>::TwinTransferNewAccountHasTwin
        );

        // Move all reserved from old -> new as reserved
        let reserved = T::Currency::reserved_balance(&req.old_account);
        if !reserved.is_zero() {
            let _ = T::Currency::repatriate_reserved(
                &req.old_account,
                &req.new_account,
                reserved,
                BalanceStatus::Reserved,
            );
        }

        // Update twin ownership and indexes
        twin.account_id = req.new_account.clone();
        Twins::<T>::insert(req.twin_id, &twin);

        // Update account->twin mapping
        TwinIdByAccountID::<T>::remove(&req.old_account);
        TwinIdByAccountID::<T>::insert(&req.new_account, req.twin_id);

        // Clear pending index and delete request
        PendingTransferByTwin::<T>::remove(req.twin_id);
        TwinTransferRequests::<T>::remove(request_id);

        // Emit events
        Self::deposit_event(Event::TwinOwnershipTransferred {
            twin_id: req.twin_id,
            old_account: req.old_account.clone(),
            new_account: req.new_account.clone(),
        });
        Self::deposit_event(Event::TwinUpdated(twin));

        Ok(().into())
    }

    pub fn _cancel_twin_transfer(
        origin: T::RuntimeOrigin,
        request_id: u64,
    ) -> DispatchResultWithPostInfo {
        let signer = ensure_signed(origin)?;

        let req = TwinTransferRequests::<T>::get(request_id)
            .ok_or(Error::<T>::TwinTransferRequestNotFound)?;

        // Only current owner (old_account) can cancel
        ensure!(
            req.old_account == signer,
            Error::<T>::UnauthorizedToUpdateTwin
        );

        // Remove request and index
        PendingTransferByTwin::<T>::remove(req.twin_id);
        TwinTransferRequests::<T>::remove(request_id);

        // Emit cancel event
        Self::deposit_event(Event::TwinTransferCanceled {
            twin_id: req.twin_id,
            old_account: req.old_account,
            new_account: req.new_account,
        });

        Ok(().into())
    }
}
