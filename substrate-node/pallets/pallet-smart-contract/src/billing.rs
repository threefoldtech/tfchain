use crate::*;
use frame_support::{
    dispatch::{DispatchErrorWithPostInfo, DispatchResultWithPostInfo},
    ensure,
    traits::{
        fungible::Inspect,
        tokens::{Fortitude::Polite, Preservation::Preserve},
        Currency, ExistenceRequirement, LockableCurrency, OnUnbalanced, WithdrawReasons,
    },
};
use frame_system::{
    offchain::{SendSignedTransaction, Signer},
    pallet_prelude::BlockNumberFor,
};
use sp_core::Get;
use sp_runtime::{
    traits::{Bounded, CheckedAdd, CheckedSub, Zero},
    DispatchResult, Perbill, SaturatedConversion,
};
use sp_std::vec::Vec;

impl<T: Config> Pallet<T> {
    // Let offchain worker check if there are contracts on
    // billing loop at current index and try to bill them
    pub fn bill_contracts_for_block(block_number: BlockNumberFor<T>) {
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
            if let Some(contract) = Contracts::<T>::get(contract_id) {
                if Self::should_bill_contract(&contract) {
                    log::info!(
                        "Starting billing contract {:?}, type: {:?}",
                        contract_id,
                        contract.contract_type
                    );
                    if Self::bill_contract_using_signed_transaction(contract_id).is_ok() {
                        log::info!(
                            "Successfully submitted signed transaction for contract {:?}",
                            contract_id
                        );
                    }
                } else {
                    log::debug!(
                        "Skipping billing node contract {:?}, no IP/CU/SU/NU to bill",
                        contract_id,
                    );
                }
            } else {
                log::debug!("Contract {:?} not exists!", contract_id);
            }
        }
        log::debug!("Finished billing contracts at block {:?}", block_number);
    }

    fn should_bill_contract(contract: &types::Contract<T>) -> bool {
        if let types::ContractData::NodeContract(node_contract) = contract.contract_type.clone() {
            let bill_ip = node_contract.public_ips > 0;
            let bill_cu_su = !NodeContractResources::<T>::get(contract.contract_id)
                .used
                .is_empty();
            let bill_nu =
                ContractBillingInformationByID::<T>::get(contract.contract_id).amount_unbilled > 0;

            return bill_ip || bill_cu_su || bill_nu;
        }
        true
    }

    pub fn bill_contract_using_signed_transaction(contract_id: u64) -> Result<(), Error<T>> {
        let signer = Signer::<T, <T as pallet::Config>::AuthorityId>::all_accounts();

        if !signer.can_sign() {
            log::error!(
                "failed billing contract {:?}, account cannot be used to sign transaction",
                contract_id
            );
            return Err(<Error<T>>::OffchainSignedTxCannotSign);
        }

        let result =
            signer.send_signed_transaction(|_acct| Call::bill_contract_for_block { contract_id });

        if result.iter().any(|(_, res)| res.is_ok()) {
            return Ok(());
        }

        log::error!(
            "All local accounts failed to submit signed transaction for contract {:?}",
            contract_id
        );
        for (_, res) in result {
            if let Err(e) = res {
                log::error!("error: {:?}", e);
            }
        }

        Err(<Error<T>>::OffchainSignedTxAlreadySent)
    }

    // Bills a contract (NodeContract, NameContract or RentContract)
    // Calculates how much TFT is due by the user and distributes the rewards
    pub fn bill_contract(contract_id: u64) -> DispatchResultWithPostInfo {
        
        let mut seen_contracts = SeenContracts::<T>::get();
        ensure!(
            !seen_contracts.contains(&contract_id),
            "this contract already processed in this block",
        );
        seen_contracts.push(contract_id);
        SeenContracts::<T>::put(seen_contracts);
        
        let mut contract = Contracts::<T>::get(contract_id).ok_or(Error::<T>::ContractNotExists)?;
        let twin =
            pallet_tfgrid::Twins::<T>::get(contract.twin_id).ok_or(Error::<T>::TwinNotExists)?;
        // if contract is not name contract ensure the node exists
        if !matches!(contract.contract_type, types::ContractData::NameContract(_)) {
            pallet_tfgrid::Nodes::<T>::get(contract.get_node_id()).ok_or(Error::<T>::NodeNotExists)?;
        }

        let usable_balance = Self::get_usable_balance(&twin.account_id);
        let stash_balance = Self::get_stash_balance(twin.id);
        // cap to max value of Balance type
        let total_balance = usable_balance.checked_add(&stash_balance).unwrap_or_else(|| {
            log::warn!(
                "overflow while calculating total balance for twin {:?}, usable_balance: {:?}, stash_balance: {:?}, max value for balance type assumed",
                contract.twin_id,
                usable_balance,
                stash_balance
            );
            BalanceOf::<T>::max_value() // TODO: check if this is the correct behavior or should we return an error
        });
        let now = Self::get_current_timestamp_in_secs();

        // Calculate amount of seconds elapsed based on the contract lock struct
        let mut contract_lock = ContractLock::<T>::get(contract.contract_id);

        // calculate the seconds elapsed since the last lock update, if the lock updated time is 0 assume 0
        let seconds_elapsed = now.checked_sub(contract_lock.lock_updated).unwrap_or_else(|| {
            log::warn!(
                "overflow while calculating seconds elapsed for contract {:?}, now: {:?}, lock_updated: {:?}, 0 assumed",
                contract_id,
                now,
                contract_lock.lock_updated
            );
            0
        });

        // Calculate total amount due / note that calculate_contract_cost_tft function uses the default pricing policy
        let (regular_amount_due, discount_received) = contract
            .calculate_contract_cost_tft(total_balance, seconds_elapsed)
            .map_err(|e| {
                log::error!("error while calculating contract cost: {:?}", e);
                e
            })?;
        let extra_amount_due = match &contract.contract_type {
            types::ContractData::RentContract(rc) => contract
                .calculate_extra_fee_cost_tft(rc.node_id, seconds_elapsed)
                .map_err(|e| {
                    log::error!("error while calculating extra fee cost: {:?}", e);
                    e
                })?,
            _ => BalanceOf::<T>::zero(),
        };
        let amount_due = regular_amount_due.checked_add(&extra_amount_due).unwrap_or_else(|| {
            log::warn!(
                "overflow while calculating total amount due for contract {:?}, regular_amount_due: {:?}, extra_amount_due: {:?}, max value for balance type assumed",
                contract_id,
                regular_amount_due,
                extra_amount_due
            );
            BalanceOf::<T>::max_value() // TODO: check if this is the correct behavior or should we return an error
        });

        // Calculate total amount locked
        let regular_lock_amount = contract_lock
            .amount_locked
            .checked_add(&regular_amount_due)
            .unwrap_or_else(|| {
                log::warn!(
                    "overflow while calculating regular lock amount for contract {:?}, amount_locked: {:?}, regular_amount_due: {:?}, max value for balance type assumed",
                    contract_id,
                    contract_lock.amount_locked,
                    regular_amount_due
                );
                BalanceOf::<T>::max_value() // TODO: check if this is the correct behavior or should we return an error
             });
        let extra_lock_amount = contract_lock
            .extra_amount_locked
            .checked_add(&extra_amount_due)
            .unwrap_or_else(|| {
                log::warn!(
                    "overflow while calculating extra lock amount for contract {:?}, extra_amount_locked: {:?}, extra_amount_due: {:?}, max value for balance type assumed",
                    contract_id,
                    contract_lock.extra_amount_locked,
                    extra_amount_due
                );
                BalanceOf::<T>::max_value() // TODO: check if this is the correct behavior or should we return an error
            });
        let lock_amount = regular_lock_amount
            .checked_add(&extra_lock_amount)
            .unwrap_or_else(|| {
                log::warn!(
                    "overflow while calculating total lock amount for contract {:?}, regular_lock_amount: {:?}, extra_lock_amount: {:?}, max value for balance type assumed",
                    contract_id,
                    regular_lock_amount,
                    extra_lock_amount
                );
                BalanceOf::<T>::max_value() // TODO: check if this is the correct behavior or should we return an error
            });

        // ____________________________________________________________________________________________________________________________ //
        // return early (contract lock not updated, no event)!
        // ____________________________________________________________________________________________________________________________ //
        // Bill rent contract only if node is online
        // TODO: ensure contract state not in (deleted, grace period)
        if let types::ContractData::RentContract(rc) = &contract.contract_type {
            // No need for preliminary call to contains_key() because default node power value is Up
            let node_power = pallet_tfgrid::NodePower::<T>::get(rc.node_id);
            if node_power.is_standby() {
                log::debug!(
                    "Skipping billing rent contract {:?}, node {:?} is in standby",
                    contract_id,
                    rc.node_id,
                );
                return Ok(().into());
            }
        }

        // ____________________________________________________________________________________________________________________________ //
        // return early (contract lock not updated, no event)!
        // ____________________________________________________________________________________________________________________________ //
        // If there is nothing to be paid and the contract is not in state delete, return
        // Can be that the users cancels the contract in the same block that it's getting billed
        // where elapsed seconds would be 0, but we still have to distribute rewards
        if amount_due.is_zero() && !contract.is_state_delete() { // TODO: check if this is valid case, i expect there is always something to be paid
            if matches!(contract.state, types::ContractState::GracePeriod(_)) {
                // TODO: check if this is a valid case
                // Oh well, the contract is in grace and for some reason the amount due is zero, now contract could stuck in grace
                log::debug!(
                    "contract {} is in grace and amount due is zero!",
                    contract.contract_id
                );
            }
            // TODO: this is likley okay, but double check if contract lock should be updated even if amount due is zero
            log::info!(
                "amount to be billed is 0, contract state {:?}, nothing to do with contract {:?}",
                contract.state,
                contract_id
            );
            return Ok(().into());
        };

        // Handle grace
        Self::handle_grace(&mut contract, usable_balance, lock_amount).or_else(|e| {
            log::error!("error while handling grace: {:?}", e);
            Err(e)
        })?;

        // TODO: verfiy if this is the correct behavior
        // Only update contract lock in state (Created, GracePeriod)
        if !matches!(contract.state, types::ContractState::Deleted(_)) {
            // increment cycles billed and update the internal lock struct
            contract_lock.lock_updated = now;
            contract_lock.cycles += 1;
            contract_lock.amount_locked = regular_lock_amount;
            contract_lock.extra_amount_locked = extra_lock_amount;
        }

        // ____________________________________________________________________________________________________________________________ //
        // return early (contract lock updated, no event)!
        // ____________________________________________________________________________________________________________________________ //
        // If still in grace period, no need to continue doing balance locking and other stuff
        if matches!(contract.state, types::ContractState::GracePeriod(_)) {
            log::info!("contract {} is still in grace", contract.contract_id);
            ContractLock::<T>::insert(contract.contract_id, &contract_lock);
            return Ok(().into());
        }

        // Handle balance lock operations for deleted, created contracts
        Self::handle_lock(&contract, &mut contract_lock, amount_due, &twin)?;

        // #########################################################################################################3

        // Always emit a contract billed event
        let contract_bill = types::ContractBill {
            contract_id: contract.contract_id,
            timestamp: Self::get_current_timestamp_in_secs(),
            discount_level: discount_received.clone(),
            amount_billed: amount_due.saturated_into::<u128>(),
        };
        Self::deposit_event(Event::ContractBilled(contract_bill));

        // ____________________________________________________________________________________________________________________________ //
        // return early (contract lock not updated, event emitted)!
        // ____________________________________________________________________________________________________________________________ //
        // If the contract is in delete state, remove all associated storage
        if matches!(contract.state, types::ContractState::Deleted(_)) {
            return Self::remove_contract(contract.contract_id);
        }

        // If contract is node contract, set the amount unbilled back to 0
        // TODO: can zos report this info to rent contract ??? most likely no, you reserve ip with a node contract 
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
        Ok(().into())
    }

    // The handle_grace function manages the contract state when the user does not have sufficient funds to cover the amount due.
    // It handles transitions to and from the grace period, ensuring that contracts can recover from or be terminated due to insufficient funds.
    fn handle_grace(
        contract: &mut types::Contract<T>,
        usable_balance: BalanceOf<T>,
        amount_due: BalanceOf<T>,
    ) -> DispatchResultWithPostInfo {
        let current_block = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
        let node_id = contract.get_node_id();

        match contract.state {
            types::ContractState::GracePeriod(grace_start) => {
                // if the usable balance is recharged, we can move the contract to created state again
                if usable_balance > amount_due {
                    log::info!(
                        "Contract {:?} is in grace period, but balance is recharged, moving to created state at block {:?}",
                        contract.contract_id,
                        current_block
                    );
                    Self::update_contract_state(contract, &types::ContractState::Created)?;
                    Self::deposit_event(Event::ContractGracePeriodEnded {
                        contract_id: contract.contract_id,
                        node_id,
                        twin_id: contract.twin_id,
                    });
                    // If the contract is a rent contract, also move state on associated node contracts
                    Self::handle_grace_rent_contract(contract, types::ContractState::Created)?;
                } else {
                    let diff = current_block.checked_sub(grace_start).unwrap_or_else(|| {
                        log::warn!(
                            "overflow while calculating elapsed grace period for contract {:?}, current_block: {:?}, grace_start: {:?}, 0 assumed",
                            contract.contract_id,
                            current_block,
                            grace_start
                        );
                        0
                    });
                    // If the contract grace period ran out, we can decomission the contract
                    if diff >= T::GracePeriod::get() {
                        log::info!(
                            "Contract state chanegd to deleted at block {:?} due to an expired grace period. elapsed blocks: {:?}",
                            current_block,
                            diff
                        );
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

        Ok(().into())
    }

    // handling rent contracts, associated node contracts are also transitioned to the appropriate state (either Created or GracePeriod).
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

    // The handle_lock function manages the locking of tokens for a contract.
    // It ensures that the correct amount of tokens is locked based on the contract's billing cycle and distributes rewards if necessary.
    fn handle_lock(
        contract: &types::Contract<T>,
        contract_lock: &mut types::ContractLock<BalanceOf<T>>,
        amount_due: BalanceOf<T>,
        twin: &pallet_tfgrid::types::Twin<T::AccountId>,
    ) -> DispatchResultWithPostInfo {
        // Only lock an amount from the user's balance if the contract is in create state
        // The lock is specified on the user's account, since a user can have multiple contracts
        // Just extend the lock with the amount due for this contract billing period (lock will be created if not exists)
        let is_rewards_ready = contract_lock.cycles >= T::DistributionFrequency::get();
        let canceled_and_not_zero: bool =
            contract.is_state_delete() && contract_lock.has_some_amount_locked();

        if matches!(contract.state, types::ContractState::Created) {
            let mut locked_balance = Self::get_locked_balance(&twin.account_id);
            locked_balance = locked_balance
                .checked_add(&amount_due)
                .unwrap_or(BalanceOf::<T>::zero()); // TODO: check if this is the correct behavior or should we return an error
            <T as Config>::Currency::set_lock(
                GRID_LOCK_ID,
                &twin.account_id,
                locked_balance,
                WithdrawReasons::all(),
            );
        }

        // When the cultivation rewards are ready to be distributed or it's in delete state
        // Unlock all reserved balance and distribute
        if is_rewards_ready || canceled_and_not_zero {
            // First remove the lock, calculate how much locked balance needs to be unlocked and re-lock the remaining locked balance
            let locked_balance = Self::get_locked_balance(&twin.account_id);
            let new_locked_balance = locked_balance
                .checked_sub(&contract_lock.total_amount_locked())
                .unwrap_or(BalanceOf::<T>::zero()); // TODO: check if this is the correct behavior or should we return an error
            <T as Config>::Currency::set_lock(
                GRID_LOCK_ID,
                &twin.account_id,
                new_locked_balance,
                WithdrawReasons::all(),
            );
            // locks are uselss in this use case, it is not granty anything
            // we can lock more fund than user have
            // and its not granted that when we unlock some amount that it will be avilable to distrbute
            // we still need to ensure that user have enough usable balance to cover payments
            // better to use hold (they are stacked not overlaped) or just contract lock (no balance lock at all)
            // just update contract lock every hour and at the end of the distrubution period check if he have enough balance, transfer else move to grace period
            // how to migrate from locks to hold
            // do lazy migration, when a conatrct get to be billed, it trigger a check:
            // - checks if user have a gridlcok 
            // loop over all user conatrct locks, remove gridlock and add hold equal to sum of all contract locks
            // if can't hold all, try to hold as much as possible
            // we set also flag on map twin: bool to indicate that this user is migrated
            let mut twin_balance = Self::get_usable_balance(&twin.account_id);
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
            let now: u64 = Self::get_current_timestamp_in_secs();
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
    // This is the balance minus the minimum balance (spendable = free - max(frozen - on_hold, ED))
    pub fn get_usable_balance(account_id: &T::AccountId) -> BalanceOf<T> {
        let spendable =
            pallet_balances::pallet::Pallet::<T>::reducible_balance(account_id, Preserve, Polite);
        let b = spendable.saturated_into::<u128>();
        BalanceOf::<T>::saturated_from(b)
    }

    // TODO: fix me / remove me
    fn get_locked_balance(account_id: &T::AccountId) -> BalanceOf<T> {
        // get locked balance by Grid lock id
        let grid_lock = pallet_balances::pallet::Pallet::<T>::locks(account_id)
            .into_iter()
            .find(|l| l.id == GRID_LOCK_ID);

        let b = grid_lock.map_or(0, |l| l.amount.saturated_into::<u128>());
        BalanceOf::<T>::saturated_from(b)
    }

    fn get_stash_balance(twin_id: u32) -> BalanceOf<T> {
        let account_id = pallet_tfgrid::TwinBoundedAccountID::<T>::get(twin_id);
        match account_id {
            Some(account) => Self::get_usable_balance(&account),
            None => BalanceOf::<T>::zero(),
        }
    }

    pub fn get_current_timestamp_in_secs() -> u64 {
        <pallet_timestamp::Pallet<T>>::get().saturated_into::<u64>() / 1000
    }
}
