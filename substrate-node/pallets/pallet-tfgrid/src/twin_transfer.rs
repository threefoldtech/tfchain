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

        let now = <frame_system::Pallet<T>>::block_number();
        let request = TwinTransferRequest::<T> {
            twin_id,
            from: old_account.clone(),
            to: new_account.clone(),
            created_at: now,
        };

        TwinTransferRequests::<T>::insert(req_id, &request);
        PendingTransferByTwin::<T>::insert(twin_id, req_id);

        Self::deposit_event(Event::TwinTransferRequested {
            request_id: req_id,
            twin_id,
            from: old_account,
            to: new_account,
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
        ensure!(req.to == signer, Error::<T>::UnauthorizedToUpdateTwin);

        // Twin must exist and still be owned by old_account
        let mut twin = Twins::<T>::get(req.twin_id).ok_or(Error::<T>::TwinNotExists)?;
        ensure!(
            twin.account_id == req.from,
            Error::<T>::UnauthorizedToUpdateTwin
        );

        // New account must still not have a twin
        ensure!(
            !TwinIdByAccountID::<T>::contains_key(&req.to),
            Error::<T>::TwinTransferNewAccountHasTwin
        );

        // Move all reserved from old -> new as reserved
        let reserved = T::Currency::reserved_balance(&req.from);
        if !reserved.is_zero() {
            let _ = T::Currency::repatriate_reserved(
                &req.from,
                &req.to,
                reserved,
                BalanceStatus::Reserved,
            );
        }

        // Update twin ownership and indexes
        twin.account_id = req.to.clone();
        Twins::<T>::insert(req.twin_id, &twin);

        // Update account->twin mapping
        TwinIdByAccountID::<T>::remove(&req.from);
        TwinIdByAccountID::<T>::insert(&req.to, req.twin_id);

        // Clear pending index and delete request
        PendingTransferByTwin::<T>::remove(req.twin_id);
        TwinTransferRequests::<T>::remove(request_id);

        // Emit events
        Self::deposit_event(Event::TwinOwnershipTransferred {
            request_id,
            twin_id: req.twin_id,
            from: req.from.clone(),
            to: req.to.clone(),
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
        ensure!(req.from == signer, Error::<T>::UnauthorizedToUpdateTwin);

        // Remove request and index
        PendingTransferByTwin::<T>::remove(req.twin_id);
        TwinTransferRequests::<T>::remove(request_id);

        // Emit cancel event
        Self::deposit_event(Event::TwinTransferCanceled {
            request_id,
            twin_id: req.twin_id,
            from: req.from,
            to: req.to,
        });

        Ok(().into())
    }
}
