use crate::*;
use frame_support::{
    dispatch::{DispatchErrorWithPostInfo, DispatchResultWithPostInfo},
    ensure,
    traits::{Currency, ExistenceRequirement, LockableCurrency, OnUnbalanced, WithdrawReasons},
};
use frame_system::{
    offchain::{SendSignedTransaction, SignMessage, Signer},
    pallet_prelude::BlockNumberFor,
};
use sp_core::Get;
use sp_runtime::{
    traits::{CheckedAdd, CheckedSub, Convert, Zero},
    DispatchResult, Perbill, SaturatedConversion,
};
use sp_std::vec::Vec;

impl<T: Config> Pallet<T> {
    pub fn bill_conttracts_for_block(block_number: BlockNumberFor<T>) {
        // Let offchain worker check if there are contracts on
        // billing loop at current index and try to bill them
        let index = Self::get_billing_loop_index_from_block_number(block_number);

        let contract_ids = ContractsToBillAt::<T>::get(index);
        if contract_ids.is_empty() {
            log::info!(
                "No contracts to bill at block {:?}, index: {:?}",
                block_number,
                index
            );
            return;
        }

        log::info!(
            "{:?} contracts to bill at block {:?}",
            contract_ids,
            block_number
        );

        for contract_id in contract_ids {
            if let Some(c) = Contracts::<T>::get(contract_id) {
                if let types::ContractData::NodeContract(node_contract) = c.contract_type {
                    // Is there IP consumption to bill?
                    let bill_ip = node_contract.public_ips > 0;

                    // Is there CU/SU consumption to bill?
                    // No need for preliminary call to contains_key() because default resource value is empty
                    let bill_cu_su = !NodeContractResources::<T>::get(contract_id).used.is_empty();

                    // Is there NU consumption to bill?
                    // No need for preliminary call to contains_key() because default amount_unbilled is 0
                    let bill_nu =
                        ContractBillingInformationByID::<T>::get(contract_id).amount_unbilled > 0;

                    // Don't bill if no IP/CU/SU/NU to be billed
                    if !bill_ip && !bill_cu_su && !bill_nu {
                        continue;
                    }
                }
            }
            let _res = Self::bill_contract_using_signed_transaction(contract_id);
        }
    }

    pub fn bill_contract_using_signed_transaction(contract_id: u64) -> Result<(), Error<T>> {
        let signer = Signer::<T, <T as pallet::Config>::AuthorityId>::any_account();

        // Only allow the author of the next block to trigger the billing
        Self::is_next_block_author(&signer)?;

        if !signer.can_sign() {
            log::error!(
                "failed billing contract {:?} account cannot be used to sign transaction",
                contract_id,
            );
            return Err(<Error<T>>::OffchainSignedTxCannotSign);
        }

        let result =
            signer.send_signed_transaction(|_acct| Call::bill_contract_for_block { contract_id });

        if let Some((acc, res)) = result {
            // if res is an error this means sending the transaction failed
            // this means the transaction was already send before (probably by another node)
            // unfortunately the error is always empty (substrate just logs the error and
            // returns Err())
            if res.is_err() {
                log::error!(
                    "signed transaction failed for billing contract {:?} using account {:?}",
                    contract_id,
                    acc.id
                );
                return Err(<Error<T>>::OffchainSignedTxAlreadySent);
            }
            return Ok(());
        }
        log::error!("No local account available");
        return Err(<Error<T>>::OffchainSignedTxNoLocalAccountAvailable);
    }

    // Bills a contract (NodeContract, NameContract or RentContract)
    // Calculates how much TFT is due by the user and distributes the rewards
    pub fn bill_contract(contract_id: u64) -> DispatchResultWithPostInfo {
        let mut contract = Contracts::<T>::get(contract_id).ok_or(Error::<T>::ContractNotExists)?;

        let twin =
            pallet_tfgrid::Twins::<T>::get(contract.twin_id).ok_or(Error::<T>::TwinNotExists)?;
        let usable_balance = Self::get_usable_balance(&twin.account_id);
        let stash_balance = Self::get_stash_balance(twin.id);
        let total_balance = usable_balance
            .checked_add(&stash_balance)
            .unwrap_or(BalanceOf::<T>::zero());

        let now = Self::get_current_timestamp_in_secs();

        // Calculate amount of seconds elapsed based on the contract lock struct
        let mut contract_lock = ContractLock::<T>::get(contract.contract_id);
        let seconds_elapsed = now.checked_sub(contract_lock.lock_updated).unwrap_or(0);

        // Calculate total amount due
        let (regular_amount_due, discount_received) =
            contract.calculate_contract_cost_tft(total_balance, seconds_elapsed)?;
        let extra_amount_due = match &contract.contract_type {
            types::ContractData::RentContract(rc) => {
                contract.calculate_extra_fee_cost_tft(rc.node_id, seconds_elapsed)?
            }
            _ => BalanceOf::<T>::zero(),
        };
        let amount_due = regular_amount_due
            .checked_add(&extra_amount_due)
            .unwrap_or(BalanceOf::<T>::zero());

        // If there is nothing to be paid and the contract is not in state delete, return
        // Can be that the users cancels the contract in the same block that it's getting billed
        // where elapsed seconds would be 0, but we still have to distribute rewards
        if amount_due == BalanceOf::<T>::zero() && !contract.is_state_delete() {
            log::debug!("amount to be billed is 0, nothing to do");
            return Ok(().into());
        };

        // Calculate total amount locked
        let regular_lock_amount = contract_lock
            .amount_locked
            .checked_add(&regular_amount_due)
            .unwrap_or(BalanceOf::<T>::zero());
        let extra_lock_amount = contract_lock
            .extra_amount_locked
            .checked_add(&extra_amount_due)
            .unwrap_or(BalanceOf::<T>::zero());
        let lock_amount = regular_lock_amount
            .checked_add(&extra_lock_amount)
            .unwrap_or(BalanceOf::<T>::zero());

        // Handle grace
        let contract = Self::handle_grace(&mut contract, usable_balance, lock_amount)?;

        // Only update contract lock in state (Created, GracePeriod)
        if !matches!(contract.state, types::ContractState::Deleted(_)) {
            // increment cycles billed and update the internal lock struct
            contract_lock.lock_updated = now;
            contract_lock.cycles += 1;
            contract_lock.amount_locked = regular_lock_amount;
            contract_lock.extra_amount_locked = extra_lock_amount;
        }

        // If still in grace period, no need to continue doing locking and other stuff
        if matches!(contract.state, types::ContractState::GracePeriod(_)) {
            log::info!("contract {} is still in grace", contract.contract_id);
            ContractLock::<T>::insert(contract.contract_id, &contract_lock);
            return Ok(().into());
        }

        // Handle contract lock operations
        Self::handle_lock(contract, &mut contract_lock, amount_due)?;

        // Always emit a contract billed event
        let contract_bill = types::ContractBill {
            contract_id: contract.contract_id,
            timestamp: Self::get_current_timestamp_in_secs(),
            discount_level: discount_received.clone(),
            amount_billed: amount_due.saturated_into::<u128>(),
        };
        Self::deposit_event(Event::ContractBilled(contract_bill));

        // If the contract is in delete state, remove all associated storage
        if matches!(contract.state, types::ContractState::Deleted(_)) {
            return Self::remove_contract(contract.contract_id);
        }

        // If contract is node contract, set the amount unbilled back to 0
        if matches!(contract.contract_type, types::ContractData::NodeContract(_)) {
            let mut contract_billing_info =
                ContractBillingInformationByID::<T>::get(contract.contract_id);
            contract_billing_info.amount_unbilled = 0;
            ContractBillingInformationByID::<T>::insert(
                contract.contract_id,
                &contract_billing_info,
            );
        }

        // Finally update the lock
        ContractLock::<T>::insert(contract.contract_id, &contract_lock);

        log::info!("successfully billed contract with id {:?}", contract_id,);

        Ok(().into())
    }

    fn handle_grace(
        contract: &mut types::Contract<T>,
        usable_balance: BalanceOf<T>,
        amount_due: BalanceOf<T>,
    ) -> Result<&mut types::Contract<T>, DispatchErrorWithPostInfo> {
        let current_block = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
        let node_id = contract.get_node_id();

        match contract.state {
            types::ContractState::GracePeriod(grace_start) => {
                // if the usable balance is recharged, we can move the contract to created state again
                if usable_balance > amount_due {
                    Self::update_contract_state(contract, &types::ContractState::Created)?;
                    Self::deposit_event(Event::ContractGracePeriodEnded {
                        contract_id: contract.contract_id,
                        node_id,
                        twin_id: contract.twin_id,
                    });
                    // If the contract is a rent contract, also move state on associated node contracts
                    Self::handle_grace_rent_contract(contract, types::ContractState::Created)?;
                } else {
                    let diff = current_block.checked_sub(grace_start).unwrap_or(0);
                    // If the contract grace period ran out, we can decomission the contract
                    if diff >= T::GracePeriod::get() {
                        Self::update_contract_state(
                            contract,
                            &types::ContractState::Deleted(types::Cause::OutOfFunds),
                        )?;
                    }
                }
            }
            types::ContractState::Created => {
                // if the user ran out of funds, move the contract to be in a grace period
                // dont lock the tokens because there is nothing to lock
                // we can still update the internal contract lock object to figure out later how much was due
                // whilst in grace period
                if amount_due >= usable_balance {
                    log::info!(
                        "Grace period started at block {:?} due to lack of funds",
                        current_block
                    );
                    Self::update_contract_state(
                        contract,
                        &types::ContractState::GracePeriod(current_block),
                    )?;
                    // We can't lock the amount due on the contract's lock because the user ran out of funds
                    Self::deposit_event(Event::ContractGracePeriodStarted {
                        contract_id: contract.contract_id,
                        node_id,
                        twin_id: contract.twin_id,
                        block_number: current_block.saturated_into(),
                    });
                    // If the contract is a rent contract, also move associated node contract to grace period
                    Self::handle_grace_rent_contract(
                        contract,
                        types::ContractState::GracePeriod(current_block),
                    )?;
                }
            }
            _ => (),
        }

        Ok(contract)
    }

    fn handle_grace_rent_contract(
        contract: &mut types::Contract<T>,
        state: types::ContractState,
    ) -> DispatchResultWithPostInfo {
        match &contract.contract_type {
            types::ContractData::RentContract(rc) => {
                let active_node_contracts = ActiveNodeContracts::<T>::get(rc.node_id);
                for ctr_id in active_node_contracts {
                    let mut ctr =
                        Contracts::<T>::get(ctr_id).ok_or(Error::<T>::ContractNotExists)?;
                    Self::update_contract_state(&mut ctr, &state)?;

                    match state {
                        types::ContractState::Created => {
                            Self::deposit_event(Event::ContractGracePeriodEnded {
                                contract_id: ctr_id,
                                node_id: rc.node_id,
                                twin_id: ctr.twin_id,
                            });
                        }
                        types::ContractState::GracePeriod(block_number) => {
                            Self::deposit_event(Event::ContractGracePeriodStarted {
                                contract_id: ctr_id,
                                node_id: rc.node_id,
                                twin_id: ctr.twin_id,
                                block_number,
                            });
                        }
                        _ => (),
                    };
                }
            }
            _ => (),
        };

        Ok(().into())
    }

    fn handle_lock(
        contract: &mut types::Contract<T>,
        contract_lock: &mut types::ContractLock<BalanceOf<T>>,
        amount_due: BalanceOf<T>,
    ) -> DispatchResultWithPostInfo {
        let now = Self::get_current_timestamp_in_secs();

        // Only lock an amount from the user's balance if the contract is in create state
        // The lock is specified on the user's account, since a user can have multiple contracts
        // Just extend the lock with the amount due for this contract billing period (lock will be created if not exists)
        let twin =
            pallet_tfgrid::Twins::<T>::get(contract.twin_id).ok_or(Error::<T>::TwinNotExists)?;
        if matches!(contract.state, types::ContractState::Created) {
            let mut locked_balance = Self::get_locked_balance(&twin.account_id);
            locked_balance = locked_balance
                .checked_add(&amount_due)
                .unwrap_or(BalanceOf::<T>::zero());
            <T as Config>::Currency::extend_lock(
                GRID_LOCK_ID,
                &twin.account_id,
                locked_balance,
                WithdrawReasons::all(),
            );
        }

        let canceled_and_not_zero =
            contract.is_state_delete() && contract_lock.has_some_amount_locked();
        // When the cultivation rewards are ready to be distributed or it's in delete state
        // Unlock all reserved balance and distribute
        if contract_lock.cycles >= T::DistributionFrequency::get() || canceled_and_not_zero {
            // First remove the lock, calculate how much locked balance needs to be unlocked and re-lock the remaining locked balance
            let locked_balance = Self::get_locked_balance(&twin.account_id);
            let new_locked_balance =
                match locked_balance.checked_sub(&contract_lock.total_amount_locked()) {
                    Some(b) => b,
                    None => BalanceOf::<T>::zero(),
                };
            <T as Config>::Currency::remove_lock(GRID_LOCK_ID, &twin.account_id);

            // Fetch twin balance, if the amount locked in the contract lock exceeds the current unlocked
            // balance we can only transfer out the remaining balance
            // https://github.com/threefoldtech/tfchain/issues/479
            let min_balance = <T as Config>::Currency::minimum_balance();
            let mut twin_balance = match new_locked_balance {
                bal if bal > min_balance => {
                    <T as Config>::Currency::set_lock(
                        GRID_LOCK_ID,
                        &twin.account_id,
                        new_locked_balance,
                        WithdrawReasons::all(),
                    );
                    Self::get_usable_balance(&twin.account_id)
                }
                _ => Self::get_usable_balance(&twin.account_id)
                    .checked_sub(&min_balance)
                    .unwrap_or(BalanceOf::<T>::zero()),
            };

            // First, distribute extra cultivation rewards if any
            if contract_lock.has_extra_amount_locked() {
                log::info!(
                    "twin balance {:?} contract lock extra amount {:?}",
                    twin_balance,
                    contract_lock.extra_amount_locked
                );

                match Self::distribute_extra_cultivation_rewards(
                    &contract,
                    twin_balance.min(contract_lock.extra_amount_locked),
                ) {
                    Ok(_) => {}
                    Err(err) => {
                        log::error!(
                            "error while distributing extra cultivation rewards {:?}",
                            err
                        );
                        return Err(err);
                    }
                };

                // Update twin balance after distribution
                twin_balance = Self::get_usable_balance(&twin.account_id);
            }

            log::info!(
                "twin balance {:?} contract lock amount {:?}",
                twin_balance,
                contract_lock.amount_locked
            );

            // Fetch the default pricing policy
            let pricing_policy = pallet_tfgrid::PricingPolicies::<T>::get(1)
                .ok_or(Error::<T>::PricingPolicyNotExists)?;

            // Then, distribute cultivation rewards
            match Self::distribute_cultivation_rewards(
                &contract,
                &pricing_policy,
                twin_balance.min(contract_lock.amount_locked),
            ) {
                Ok(_) => {}
                Err(err) => {
                    log::error!("error while distributing cultivation rewards {:?}", err);
                    return Err(err);
                }
            };

            // Reset contract lock values
            contract_lock.lock_updated = now;
            contract_lock.amount_locked = BalanceOf::<T>::zero();
            contract_lock.extra_amount_locked = BalanceOf::<T>::zero();
            contract_lock.cycles = 0;
        }

        Ok(().into())
    }

    fn distribute_extra_cultivation_rewards(
        contract: &types::Contract<T>,
        amount: BalanceOf<T>,
    ) -> DispatchResultWithPostInfo {
        log::info!(
            "Distributing extra cultivation rewards for contract {:?} with amount {:?}",
            contract.contract_id,
            amount,
        );

        // If the amount is zero, return
        if amount == BalanceOf::<T>::zero() {
            return Ok(().into());
        }

        // Fetch source twin = dedicated node user
        let src_twin =
            pallet_tfgrid::Twins::<T>::get(contract.twin_id).ok_or(Error::<T>::TwinNotExists)?;

        // Fetch destination twin = farmer
        let dst_twin = match &contract.contract_type {
            types::ContractData::RentContract(rc) => {
                let node =
                    pallet_tfgrid::Nodes::<T>::get(rc.node_id).ok_or(Error::<T>::NodeNotExists)?;
                let farm = pallet_tfgrid::Farms::<T>::get(node.farm_id)
                    .ok_or(Error::<T>::FarmNotExists)?;
                pallet_tfgrid::Twins::<T>::get(farm.twin_id).ok_or(Error::<T>::TwinNotExists)?
            }
            _ => {
                return Err(DispatchErrorWithPostInfo::from(
                    Error::<T>::InvalidContractType,
                ));
            }
        };

        // Send 100% to the node's owner (farmer)
        log::debug!(
            "Transfering: {:?} from contract twin {:?} to farmer account {:?}",
            &amount,
            &src_twin.account_id,
            &dst_twin.account_id,
        );
        <T as Config>::Currency::transfer(
            &src_twin.account_id,
            &dst_twin.account_id,
            amount,
            ExistenceRequirement::KeepAlive,
        )?;

        Ok(().into())
    }

    // Following: https://library.threefold.me/info/threefold#/tfgrid/farming/threefold__proof_of_utilization
    fn distribute_cultivation_rewards(
        contract: &types::Contract<T>,
        pricing_policy: &pallet_tfgrid::types::PricingPolicy<T::AccountId>,
        amount: BalanceOf<T>,
    ) -> DispatchResultWithPostInfo {
        log::info!(
            "Distributing cultivation rewards for contract {:?} with amount {:?}",
            contract.contract_id,
            amount,
        );

        // If the amount is zero, return
        if amount == BalanceOf::<T>::zero() {
            return Ok(().into());
        }

        // fetch source twin
        let twin =
            pallet_tfgrid::Twins::<T>::get(contract.twin_id).ok_or(Error::<T>::TwinNotExists)?;

        // Send 10% to the foundation
        let foundation_share = Perbill::from_percent(10) * amount;
        log::debug!(
            "Transfering: {:?} from contract twin {:?} to foundation account {:?}",
            &foundation_share,
            &twin.account_id,
            &pricing_policy.foundation_account
        );
        <T as Config>::Currency::transfer(
            &twin.account_id,
            &pricing_policy.foundation_account,
            foundation_share,
            ExistenceRequirement::KeepAlive,
        )?;

        // TODO: send 5% to the staking pool account
        let staking_pool_share = Perbill::from_percent(5) * amount;
        let staking_pool_account = T::StakingPoolAccount::get();
        log::debug!(
            "Transfering: {:?} from contract twin {:?} to staking pool account {:?}",
            &staking_pool_share,
            &twin.account_id,
            &staking_pool_account,
        );
        <T as Config>::Currency::transfer(
            &twin.account_id,
            &staking_pool_account,
            staking_pool_share,
            ExistenceRequirement::KeepAlive,
        )?;

        let mut sales_share = 50;

        if let Some(provider_id) = contract.solution_provider_id {
            if let Some(solution_provider) = SolutionProviders::<T>::get(provider_id) {
                let total_take: u8 = solution_provider
                    .providers
                    .iter()
                    .map(|provider| provider.take)
                    .sum();
                sales_share -= total_take;

                if !solution_provider
                    .providers
                    .iter()
                    .map(|provider| {
                        let share = Perbill::from_percent(provider.take as u32) * amount;
                        log::debug!(
                            "Transfering: {:?} from contract twin {:?} to provider account {:?}",
                            &share,
                            &twin.account_id,
                            &provider.who
                        );
                        <T as Config>::Currency::transfer(
                            &twin.account_id,
                            &provider.who,
                            share,
                            ExistenceRequirement::KeepAlive,
                        )
                    })
                    .filter(|result| result.is_err())
                    .collect::<Vec<DispatchResult>>()
                    .is_empty()
                {
                    return Err(DispatchErrorWithPostInfo::from(
                        Error::<T>::InvalidProviderConfiguration,
                    ));
                }
            }
        };

        if sales_share > 0 {
            let share = Perbill::from_percent(sales_share.into()) * amount;
            // Transfer the remaining share to the sales account
            // By default it is 50%, if a contract has solution providers it can be less
            log::debug!(
                "Transfering: {:?} from contract twin {:?} to sales account {:?}",
                &share,
                &twin.account_id,
                &pricing_policy.certified_sales_account
            );
            <T as Config>::Currency::transfer(
                &twin.account_id,
                &pricing_policy.certified_sales_account,
                share,
                ExistenceRequirement::KeepAlive,
            )?;
        }

        // Burn 35%, to not have any imbalance in the system, subtract all previously send amounts with the initial
        let amount_to_burn =
            (Perbill::from_percent(50) * amount) - foundation_share - staking_pool_share;

        let to_burn = T::Currency::withdraw(
            &twin.account_id,
            amount_to_burn,
            WithdrawReasons::FEE,
            ExistenceRequirement::KeepAlive,
        )?;

        log::debug!(
            "Burning: {:?} from contract twin {:?}",
            amount_to_burn,
            &twin.account_id
        );
        T::Burn::on_unbalanced(to_burn);

        Self::deposit_event(Event::TokensBurned {
            contract_id: contract.contract_id,
            amount: amount_to_burn,
        });

        Ok(().into())
    }

    // Billing index is contract id % (mod) Billing Frequency
    // So index belongs to [0; billing_frequency - 1] range
    pub fn get_billing_loop_index_from_contract_id(contract_id: u64) -> u64 {
        contract_id % BillingFrequency::<T>::get()
    }

    // Billing index is block number % (mod) Billing Frequency
    // So index belongs to [0; billing_frequency - 1] range
    pub fn get_billing_loop_index_from_block_number(block_number: BlockNumberFor<T>) -> u64 {
        block_number.saturated_into::<u64>() % BillingFrequency::<T>::get()
    }

    // Inserts a contract in a billing loop where the index is the contract id % billing frequency
    // This way, we don't need to reinsert the contract everytime it gets billed
    pub fn insert_contract_in_billing_loop(contract_id: u64) {
        let index = Self::get_billing_loop_index_from_contract_id(contract_id);
        let mut contract_ids = ContractsToBillAt::<T>::get(index);

        if !contract_ids.contains(&contract_id) {
            contract_ids.push(contract_id);
            ContractsToBillAt::<T>::insert(index, &contract_ids);
            log::debug!(
                "Updated contracts after insertion: {:?}, to be billed at index {:?}",
                contract_ids,
                index
            );
        }
    }

    // Removes contract from billing loop where the index is the contract id % billing frequency
    pub fn remove_contract_from_billing_loop(
        contract_id: u64,
    ) -> Result<(), DispatchErrorWithPostInfo> {
        let index = Self::get_billing_loop_index_from_contract_id(contract_id);
        let mut contract_ids = ContractsToBillAt::<T>::get(index);

        ensure!(
            contract_ids.contains(&contract_id),
            Error::<T>::ContractWrongBillingLoopIndex
        );

        contract_ids.retain(|&c| c != contract_id);
        ContractsToBillAt::<T>::insert(index, &contract_ids);
        log::debug!(
            "Updated contracts after removal: {:?}, to be billed at index {:?}",
            contract_ids,
            index
        );

        Ok(())
    }

    pub fn _change_billing_frequency(frequency: u64) -> DispatchResultWithPostInfo {
        let billing_frequency = BillingFrequency::<T>::get();
        ensure!(
            frequency > billing_frequency,
            Error::<T>::CanOnlyIncreaseFrequency
        );

        BillingFrequency::<T>::put(frequency);
        Self::deposit_event(Event::BillingFrequencyChanged(frequency));

        Ok(().into())
    }

    // Get the usable balance of an account
    // This is the balance minus the minimum balance
    pub fn get_usable_balance(account_id: &T::AccountId) -> BalanceOf<T> {
        let balance = pallet_balances::pallet::Pallet::<T>::usable_balance(account_id);
        let b = balance.saturated_into::<u128>();
        BalanceOf::<T>::saturated_from(b)
    }

    fn get_locked_balance(account_id: &T::AccountId) -> BalanceOf<T> {
        let usable_balance = Self::get_usable_balance(account_id);
        let free_balance = <T as Config>::Currency::free_balance(account_id);

        let locked_balance = free_balance.checked_sub(&usable_balance);
        match locked_balance {
            Some(balance) => balance,
            None => BalanceOf::<T>::zero(),
        }
    }

    fn get_stash_balance(twin_id: u32) -> BalanceOf<T> {
        let account_id = pallet_tfgrid::TwinBoundedAccountID::<T>::get(twin_id);
        match account_id {
            Some(account) => Self::get_usable_balance(&account),
            None => BalanceOf::<T>::zero(),
        }
    }

    // Validates if the given signer is the next block author based on the validators in session
    // This can be used if an extrinsic should be refunded by the author in the same block
    // It also requires that the keytype inserted for the offchain workers is the validator key
    fn is_next_block_author(
        signer: &Signer<T, <T as Config>::AuthorityId>,
    ) -> Result<(), Error<T>> {
        let author = <pallet_authorship::Pallet<T>>::author();
        let validators = <pallet_session::Pallet<T>>::validators();

        // Sign some arbitrary data in order to get the AccountId, maybe there is another way to do this?
        let signed_message = signer.sign_message(&[0]);
        if let Some(signed_message_data) = signed_message {
            if let Some(block_author) = author {
                let validator =
                    <T as pallet_session::Config>::ValidatorIdOf::convert(block_author.clone())
                        .ok_or(Error::<T>::IsNotAnAuthority)?;

                let validator_count = validators.len();
                let author_index = (validators.iter().position(|a| a == &validator).unwrap_or(0)
                    + 1)
                    % validator_count;

                let signer_validator_account =
                    <T as pallet_session::Config>::ValidatorIdOf::convert(
                        signed_message_data.0.id.clone(),
                    )
                    .ok_or(Error::<T>::IsNotAnAuthority)?;

                if signer_validator_account != validators[author_index] {
                    return Err(Error::<T>::WrongAuthority);
                }
            }
        }

        Ok(().into())
    }

    pub fn get_current_timestamp_in_secs() -> u64 {
        <pallet_timestamp::Pallet<T>>::get().saturated_into::<u64>() / 1000
    }
}
