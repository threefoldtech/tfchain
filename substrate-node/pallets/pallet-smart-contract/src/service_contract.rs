use crate::*;
use frame_support::{
    dispatch::{DispatchErrorWithPostInfo, DispatchResultWithPostInfo},
    ensure,
    traits::{Currency, ExistenceRequirement},
    transactional, BoundedVec,
};
use sp_core::Get;
use sp_std::{vec, vec::Vec};
use substrate_fixed::types::U64F64;

impl<T: Config> Pallet<T> {
    pub fn _service_contract_create(
        caller: T::AccountId,
        service: T::AccountId,
        consumer: T::AccountId,
    ) -> DispatchResultWithPostInfo {
        let caller_twin_id =
            pallet_tfgrid::TwinIdByAccountID::<T>::get(&caller).ok_or(Error::<T>::TwinNotExists)?;

        let service_twin_id = pallet_tfgrid::TwinIdByAccountID::<T>::get(&service)
            .ok_or(Error::<T>::TwinNotExists)?;

        let consumer_twin_id = pallet_tfgrid::TwinIdByAccountID::<T>::get(&consumer)
            .ok_or(Error::<T>::TwinNotExists)?;

        // Only service or consumer can create contract
        ensure!(
            caller_twin_id == service_twin_id || caller_twin_id == consumer_twin_id,
            Error::<T>::TwinNotAuthorized,
        );

        // Service twin and consumer twin can not be the same
        ensure!(
            service_twin_id != consumer_twin_id,
            Error::<T>::ServiceContractCreationNotAllowed,
        );

        // Get the service contract ID map and increment
        let mut id = ServiceContractID::<T>::get();
        id = id + 1;

        // Create service contract
        let service_contract = types::ServiceContract {
            service_contract_id: id,
            service_twin_id,
            consumer_twin_id,
            base_fee: 0,
            variable_fee: 0,
            metadata: vec![].try_into().unwrap(),
            accepted_by_service: false,
            accepted_by_consumer: false,
            last_bill: 0,
            state: types::ServiceContractState::Created,
        };

        // Insert into service contract map
        ServiceContracts::<T>::insert(id, &service_contract);

        // Update Contract ID
        ServiceContractID::<T>::put(id);

        // Trigger event for service contract creation
        Self::deposit_event(Event::ServiceContractCreated(service_contract));

        Ok(().into())
    }

    pub fn _service_contract_set_metadata(
        account_id: T::AccountId,
        service_contract_id: u64,
        metadata: Vec<u8>,
    ) -> DispatchResultWithPostInfo {
        let twin_id = pallet_tfgrid::TwinIdByAccountID::<T>::get(&account_id)
            .ok_or(Error::<T>::TwinNotExists)?;

        let mut service_contract = ServiceContracts::<T>::get(service_contract_id)
            .ok_or(Error::<T>::ServiceContractNotExists)?;

        // Only service or consumer can set metadata
        ensure!(
            twin_id == service_contract.service_twin_id
                || twin_id == service_contract.consumer_twin_id,
            Error::<T>::TwinNotAuthorized,
        );

        // Only allow to modify metadata if contract still not approved by both parties
        ensure!(
            !matches!(
                service_contract.state,
                types::ServiceContractState::ApprovedByBoth
            ),
            Error::<T>::ServiceContractModificationNotAllowed,
        );

        service_contract.metadata = BoundedVec::try_from(metadata)
            .map_err(|_| Error::<T>::ServiceContractMetadataTooLong)?;

        // If base_fee is set and non-zero (mandatory)
        if service_contract.base_fee != 0 {
            service_contract.state = types::ServiceContractState::AgreementReady;
        }

        // Update service contract in map after modification
        ServiceContracts::<T>::insert(service_contract_id, service_contract.clone());

        // Trigger event for service contract metadata setting
        Self::deposit_event(Event::ServiceContractMetadataSet(service_contract));

        Ok(().into())
    }

    pub fn _service_contract_set_fees(
        account_id: T::AccountId,
        service_contract_id: u64,
        base_fee: u64,
        variable_fee: u64,
    ) -> DispatchResultWithPostInfo {
        let twin_id = pallet_tfgrid::TwinIdByAccountID::<T>::get(&account_id)
            .ok_or(Error::<T>::TwinNotExists)?;

        let mut service_contract = ServiceContracts::<T>::get(service_contract_id)
            .ok_or(Error::<T>::ServiceContractNotExists)?;

        // Only service can set fees
        ensure!(
            twin_id == service_contract.service_twin_id,
            Error::<T>::TwinNotAuthorized,
        );

        // Only allow to modify fees if contract still not approved by both parties
        ensure!(
            !matches!(
                service_contract.state,
                types::ServiceContractState::ApprovedByBoth
            ),
            Error::<T>::ServiceContractModificationNotAllowed,
        );

        service_contract.base_fee = base_fee;
        service_contract.variable_fee = variable_fee;

        // If metadata is filled and not empty (mandatory)
        if !service_contract.metadata.is_empty() {
            service_contract.state = types::ServiceContractState::AgreementReady;
        }

        // Update service contract in map after modification
        ServiceContracts::<T>::insert(service_contract_id, service_contract.clone());

        // Trigger event for service contract fees setting
        Self::deposit_event(Event::ServiceContractFeesSet(service_contract));

        Ok(().into())
    }

    pub fn _service_contract_approve(
        account_id: T::AccountId,
        service_contract_id: u64,
    ) -> DispatchResultWithPostInfo {
        let twin_id = pallet_tfgrid::TwinIdByAccountID::<T>::get(&account_id)
            .ok_or(Error::<T>::TwinNotExists)?;

        let mut service_contract = ServiceContracts::<T>::get(service_contract_id)
            .ok_or(Error::<T>::ServiceContractNotExists)?;

        // Allow to approve contract only if agreement is ready
        ensure!(
            matches!(
                service_contract.state,
                types::ServiceContractState::AgreementReady
            ),
            Error::<T>::ServiceContractApprovalNotAllowed,
        );

        // Only service or consumer can accept agreement
        if twin_id == service_contract.service_twin_id {
            service_contract.accepted_by_service = true;
        } else if twin_id == service_contract.consumer_twin_id {
            service_contract.accepted_by_consumer = true
        } else {
            return Err(DispatchErrorWithPostInfo::from(
                Error::<T>::TwinNotAuthorized,
            ));
        }

        // If both parties (service and consumer) accept then contract is approved and can be billed
        if service_contract.accepted_by_service && service_contract.accepted_by_consumer {
            // Change contract state to approved and emit event
            service_contract.state = types::ServiceContractState::ApprovedByBoth;

            // Initialize billing time
            let now = Self::get_current_timestamp_in_secs();
            service_contract.last_bill = now;
        }

        // Update service contract in map after modification
        ServiceContracts::<T>::insert(service_contract_id, service_contract.clone());

        // Trigger event for service contract approval
        Self::deposit_event(Event::ServiceContractApproved(service_contract));

        Ok(().into())
    }

    pub fn _service_contract_reject(
        account_id: T::AccountId,
        service_contract_id: u64,
    ) -> DispatchResultWithPostInfo {
        let service_contract = ServiceContracts::<T>::get(service_contract_id)
            .ok_or(Error::<T>::ServiceContractNotExists)?;

        // Allow to reject contract only if agreement is ready
        ensure!(
            matches!(
                service_contract.state,
                types::ServiceContractState::AgreementReady
            ),
            Error::<T>::ServiceContractRejectionNotAllowed,
        );

        // If one party (service or consumer) rejects agreement
        // then contract is canceled and removed from service contract map
        Self::_service_contract_cancel(
            account_id,
            service_contract_id,
            types::Cause::CanceledByUser,
        )?;

        Ok(().into())
    }

    pub fn _service_contract_cancel(
        account_id: T::AccountId,
        service_contract_id: u64,
        cause: types::Cause,
    ) -> DispatchResultWithPostInfo {
        let twin_id = pallet_tfgrid::TwinIdByAccountID::<T>::get(&account_id)
            .ok_or(Error::<T>::TwinNotExists)?;

        let service_contract = ServiceContracts::<T>::get(service_contract_id)
            .ok_or(Error::<T>::ServiceContractNotExists)?;

        // Only service or consumer can cancel contract
        ensure!(
            twin_id == service_contract.service_twin_id
                || twin_id == service_contract.consumer_twin_id,
            Error::<T>::TwinNotAuthorized,
        );

        // Remove contract from service contract map
        // Can be done at any state of contract
        // so no need to handle state validation
        ServiceContracts::<T>::remove(service_contract_id);

        // Trigger event for service contract cancelation
        Self::deposit_event(Event::ServiceContractCanceled {
            service_contract_id,
            cause,
        });

        log::debug!(
            "successfully removed service contract with id {:?}",
            service_contract_id,
        );

        Ok(().into())
    }

    #[transactional]
    pub fn _service_contract_bill(
        account_id: T::AccountId,
        service_contract_id: u64,
        variable_amount: u64,
        metadata: Vec<u8>,
    ) -> DispatchResultWithPostInfo {
        let twin_id = pallet_tfgrid::TwinIdByAccountID::<T>::get(&account_id)
            .ok_or(Error::<T>::TwinNotExists)?;

        let mut service_contract = ServiceContracts::<T>::get(service_contract_id)
            .ok_or(Error::<T>::ServiceContractNotExists)?;

        // Only service can bill consumer for service contract
        ensure!(
            twin_id == service_contract.service_twin_id,
            Error::<T>::TwinNotAuthorized,
        );

        // Allow to bill contract only if approved by both
        ensure!(
            matches!(
                service_contract.state,
                types::ServiceContractState::ApprovedByBoth
            ),
            Error::<T>::ServiceContractBillingNotApprovedByBoth,
        );

        // Get elapsed time (in seconds) to bill for service
        let now = Self::get_current_timestamp_in_secs();
        let elapsed_seconds_since_last_bill = now - service_contract.last_bill;

        // Billing time (window) is max 1h by design
        // So extra time will not be billed
        // It is the service responsability to bill on right frequency
        let window = elapsed_seconds_since_last_bill.min(T::BillingReferencePeriod::get());

        // Billing variable amount is bounded by contract variable fee
        ensure!(
            variable_amount
                <= ((U64F64::from_num(window)
                    / U64F64::from_num(T::BillingReferencePeriod::get()))
                    * U64F64::from_num(service_contract.variable_fee))
                .round()
                .to_num::<u64>(),
            Error::<T>::ServiceContractBillingVariableAmountTooHigh,
        );

        let bill_metadata = BoundedVec::try_from(metadata)
            .map_err(|_| Error::<T>::ServiceContractBillMetadataTooLong)?;

        // Create service contract bill
        let service_contract_bill = types::ServiceContractBill {
            variable_amount,
            window,
            metadata: bill_metadata,
        };

        // Make consumer pay for service contract bill
        let amount =
            Self::_service_contract_pay_bill(service_contract_id, service_contract_bill.clone())?;

        // Update contract in list after modification
        service_contract.last_bill = now;
        ServiceContracts::<T>::insert(service_contract_id, service_contract.clone());

        // Trigger event for service contract billing
        Self::deposit_event(Event::ServiceContractBilled {
            service_contract,
            bill: service_contract_bill,
            amount,
        });

        Ok(().into())
    }

    // Pay a service contract bill
    // Calculates how much TFT is due by the consumer and pay the amount to the service
    fn _service_contract_pay_bill(
        service_contract_id: u64,
        bill: types::ServiceContractBill,
    ) -> Result<BalanceOf<T>, DispatchErrorWithPostInfo> {
        let service_contract = ServiceContracts::<T>::get(service_contract_id)
            .ok_or(Error::<T>::ServiceContractNotExists)?;
        let amount = service_contract.calculate_bill_cost_tft::<T>(bill.clone())?;

        let service_twin_id = service_contract.service_twin_id;
        let service_twin =
            pallet_tfgrid::Twins::<T>::get(service_twin_id).ok_or(Error::<T>::TwinNotExists)?;

        let consumer_twin = pallet_tfgrid::Twins::<T>::get(service_contract.consumer_twin_id)
            .ok_or(Error::<T>::TwinNotExists)?;

        let usable_balance = Self::get_safe_usable_balance(&consumer_twin.account_id);

        // If consumer is out of funds then contract is canceled
        // by service and removed from service contract map
        if usable_balance < amount {
            Self::_service_contract_cancel(
                service_twin.account_id,
                service_contract_id,
                types::Cause::OutOfFunds,
            )?;
            return Err(DispatchErrorWithPostInfo::from(
                Error::<T>::ServiceContractNotEnoughFundsToPayBill,
            ));
        }

        // Transfer amount due from consumer account to service account
        <T as Config>::Currency::transfer(
            &consumer_twin.account_id,
            &service_twin.account_id,
            amount,
            ExistenceRequirement::KeepAlive,
        )?;

        log::debug!(
            "bill successfully payed by consumer for service contract with id {:?}",
            service_contract_id,
        );

        Ok(amount)
    }
}
