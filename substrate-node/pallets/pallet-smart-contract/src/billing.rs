use crate::*;
use frame_support::traits::DefensiveSaturating;
use frame_support::{
    dispatch::{DispatchErrorWithPostInfo, DispatchResultWithPostInfo},
    ensure,
    traits::{
        fungible::Inspect,
        tokens::{Fortitude::Polite, Preservation::Preserve},
        LockableCurrency, OnUnbalanced, ReservableCurrency,
    },
};

use frame_support::traits::BalanceStatus;
use frame_support::traits::StoredMap;
use frame_system::{
    offchain::{SendSignedTransaction, Signer},
    pallet_prelude::BlockNumberFor,
};
use sp_core::Get;
use sp_runtime::{
    traits::{Saturating, Zero},
    DispatchResult, Perbill, SaturatedConversion,
};

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
        log::debug!("Starting billing for contract_id: {:?}", contract_id);

        // pre check if contract is already processed in this block
        let mut seen_contracts = SeenContracts::<T>::get();
        ensure!(
            !seen_contracts.contains(&contract_id),
            Error::<T>::ContractAlreadyProcessedInBlock,
        );
        seen_contracts.push(contract_id);
        SeenContracts::<T>::put(seen_contracts);

        let mut contract = Contracts::<T>::get(contract_id).ok_or_else(|| {
            log::error!("Contract not exists: {:?}", contract_id);
            Error::<T>::ContractNotExists
        })?;
        let src_twin = pallet_tfgrid::Twins::<T>::get(contract.twin_id).ok_or_else(|| {
            log::error!("Twin not exists: {:?}", contract.twin_id);
            Error::<T>::TwinNotExists
        })?;
        let pricing_policy = pallet_tfgrid::PricingPolicies::<T>::get(1).ok_or_else(|| {
            log::error!("Pricing policy not exists");
            Error::<T>::PricingPolicyNotExists
        })?;

        // if contract is not name contract ensure the node, farm and farmer twin exists
        let farmer_twin = if !matches!(contract.contract_type, types::ContractData::NameContract(_))
        {
            let node = pallet_tfgrid::Nodes::<T>::get(contract.get_node_id()).ok_or_else(|| {
                log::error!("Node not exists for contract_id: {:?}", contract_id);
                Error::<T>::NodeNotExists
            })?;
            let farm = pallet_tfgrid::Farms::<T>::get(node.farm_id).ok_or_else(|| {
                log::error!("Farm not exists for node_id: {:?}", node.farm_id);
                Error::<T>::FarmNotExists
            })?;
            let farmer_twin = pallet_tfgrid::Twins::<T>::get(farm.twin_id).ok_or_else(|| {
                log::error!("Twin not exists for farm_id: {:?}", farm.twin_id);
                Error::<T>::TwinNotExists
            })?;
            Some(farmer_twin)
        } else {
            None
        };

        // Update contract payment state and contract lock
        let mut contract_payment_state = ContractPaymentState::<T>::get(contract.contract_id);
        let old_contract_lock = ContractLock::<T>::take(contract.contract_id); // get and remove the contract lock
                                                                               // this is no-op if the contract lock is empty (migrated)
        Self::ensure_contract_migrated(
            &src_twin.account_id,
            &old_contract_lock,
            &mut contract_payment_state,
        );

        // Calculate user total usable balance
        let twin_usable_balance = Self::get_usable_balance(&src_twin.account_id);
        let stash_usable_balance = Self::get_stash_balance(src_twin.id);
        let total_usable_balance =
            twin_usable_balance.defensive_saturating_add(stash_usable_balance);
        let now: u64 = Self::get_current_timestamp_in_secs();

        // Calculate amount of seconds elapsed based on the contract payment state
        let seconds_elapsed =
            now.defensive_saturating_sub(contract_payment_state.last_updated_seconds);

        let should_waive_payment = match &contract.contract_type {
            types::ContractData::RentContract(rc) => {
                let node_power = pallet_tfgrid::NodePower::<T>::get(rc.node_id);
                node_power.is_standby()
            }
            _ => false,
        };

        if should_waive_payment {
            log::debug!("Waiving rent for contract_id: {:?}", contract.contract_id);
            Self::deposit_event(Event::RentWaived {
                contract_id: contract.contract_id,
            });
            // although no billing required here, in deleted state, we should continue the billing processs for conatrcts to distribute rewards if any and clean up storage
            if matches!(contract.state, types::ContractState::Created) {
                // so for created rent contracts, if the node is in standby, don't expect the billing cycle to advance
                return Ok(().into());
            }
        }

        // Calculate the due amount
        let (standard_amount_due, discount_received) = if should_waive_payment {
            (BalanceOf::<T>::zero(), types::DiscountLevel::None)
        } else {
            contract
                .calculate_contract_cost_tft(total_usable_balance, seconds_elapsed)
                .map_err(|e| {
                    log::error!("Error while calculating contract cost: {:?}", e);
                    e
                })?
        };

        let additional_amount_due =
            if let types::ContractData::RentContract(rc) = &contract.contract_type {
                if should_waive_payment {
                    BalanceOf::<T>::zero()
                } else {
                    contract.calculate_extra_fee_cost_tft(rc.node_id, seconds_elapsed)
                }
            } else {
                BalanceOf::<T>::zero()
            };

        let total_amount_due = standard_amount_due.defensive_saturating_add(additional_amount_due);

        // if the amount due is zero and the contract is not in deleted, don't bill the contract (mostly node contarct on a rented node)
        if total_amount_due.is_zero() && !matches!(contract.state, types::ContractState::Deleted(_))
        {
            log::debug!(
                "Amount to be billed is 0, contract state: {:?}, nothing to do with contract_id: {:?}",
                contract.state,
                contract.contract_id
            );
            return Ok(().into());
        }

        // Calculate the amount needed to be reserved from the user's balance
        // should be the total amount due for current cycle + any overdrafted amount from previous cycles
        let standard_amount_to_reserve = standard_amount_due
            .defensive_saturating_add(contract_payment_state.standard_overdrafted);
        let additional_amount_to_reserve = additional_amount_due
            .defensive_saturating_add(contract_payment_state.additional_overdrafted);
        let total_amount_to_reserve =
            standard_amount_to_reserve.defensive_saturating_add(additional_amount_to_reserve);

        let has_sufficient_fund =
            <T as Config>::Currency::can_reserve(&src_twin.account_id, total_amount_to_reserve);
        let current_block = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();

        _ = Self::handle_grace(&mut contract, has_sufficient_fund, current_block);

        if has_sufficient_fund {
            log::info!("Billing contract_id: {:?}, This cycle amount due: {:?}, Total (include previous overdraft) {:?}", contract.contract_id, total_amount_due, total_amount_to_reserve);
            Self::reserve_funds(
                &mut contract_payment_state,
                standard_amount_due,
                additional_amount_due,
                &src_twin,
                &contract,
                now,
                total_amount_to_reserve,
                discount_received,
            )?;
        } else {
            log::info!(
                "Contract payment overdrafted for contract_id: {:?}",
                contract.contract_id
            );
            Self::overdraft_funds(
                &mut contract_payment_state,
                standard_amount_due,
                additional_amount_due,
                &src_twin,
                &contract,
                now,
            )?;
        }

        // distribute rewards, no-op if contract nither in deleted state nor the distribution frequency is reached
        Self::remit_funds(
            &contract,
            &mut contract_payment_state,
            &src_twin,
            farmer_twin,
            &pricing_policy,
        )?;
        log::info!(
            "Rewards distributed for contract_id: {:?}",
            contract.contract_id
        );

        // housekeeping for contracts in deleted state
        if matches!(contract.state, types::ContractState::Deleted(_)) {
            log::info!(
                "contract id {:?} in deleted state. clean up storage.",
                contract.contract_id
            );
            return Self::remove_contract(contract.contract_id);
        }

        // reset NU amount if the contract is a node contract
        if matches!(contract.contract_type, types::ContractData::NodeContract(_)) {
            let mut contract_billing_info =
                ContractBillingInformationByID::<T>::get(contract.contract_id);
            contract_billing_info.amount_unbilled = 0;
            ContractBillingInformationByID::<T>::insert(
                contract.contract_id,
                &contract_billing_info,
            );
        }

        contract_payment_state.last_updated_seconds = now;
        contract_payment_state.cycles.defensive_saturating_inc();
        ContractPaymentState::<T>::insert(contract.contract_id, &contract_payment_state);
        log::info!(
            "Billing completed for contract_id: {:?}",
            contract.contract_id
        );
        Ok(().into())
    }

    fn ensure_contract_migrated(
        account_id: &T::AccountId,
        contract_lock: &types::ContractLock<BalanceOf<T>>,
        contract_payment_state: &mut types::ContractPaymentState<BalanceOf<T>>,
    ) {
        if !contract_lock.is_migrated() {
            contract_payment_state.last_updated_seconds = contract_lock.lock_updated;
            contract_payment_state.standard_overdrafted = contract_lock.amount_locked;
            contract_payment_state.additional_overdrafted = contract_lock.extra_amount_locked;
            contract_payment_state.cycles = contract_lock.cycles;

            let locks = pallet_balances::Pallet::<T>::locks(&account_id);
            for lock in locks {
                pallet_balances::Pallet::<T>::remove_lock(lock.id, &account_id);
            }
        }
    }

    // Handles the grace period for a contract
    fn handle_grace(
        contract: &mut types::Contract<T>,
        has_sufficient_fund: bool,
        current_block: u64,
    ) -> DispatchResultWithPostInfo {
        Ok((match contract.state {
            types::ContractState::GracePeriod(_) if has_sufficient_fund => {
                log::info!("Contract {:?} is in grace period, but balance is recharged, moving to created state at block {:?}", contract.contract_id, current_block);
                Self::update_contract_state(contract, &types::ContractState::Created)?;
                Self::deposit_event(Event::ContractGracePeriodEnded {
                    contract_id: contract.contract_id,
                    node_id: contract.get_node_id(),
                    twin_id: contract.twin_id,
                });
                Self::handle_grace_rent_contract(contract, types::ContractState::Created)?;
            }
            types::ContractState::GracePeriod(grace_start) => {
                let diff = current_block.defensive_saturating_sub(grace_start);
                if diff >= T::GracePeriod::get() {
                    log::info!("Contract {:?} state changed to deleted at block {:?} due to an expired grace period. Elapsed blocks: {:?}", contract.contract_id, current_block, diff);
                    Self::deposit_event(Event::ContractGracePeriodElapsed {
                        contract_id: contract.contract_id,
                        grace_period: diff
                    });
                    Self::update_contract_state(
                        contract,
                        &types::ContractState::Deleted(types::Cause::OutOfFunds),
                    )?;
                }
            }
            types::ContractState::Created if !has_sufficient_fund => {
                log::info!(
                    "Grace period started at block {:?} due to lack of funds",
                    current_block
                );
                Self::update_contract_state(
                    contract,
                    &types::ContractState::GracePeriod(current_block.saturated_into()),
                )?;
                Self::deposit_event(Event::ContractGracePeriodStarted {
                    contract_id: contract.contract_id,
                    node_id: contract.get_node_id(),
                    twin_id: contract.twin_id,
                    block_number: current_block.saturated_into(),
                });
                Self::handle_grace_rent_contract(
                    contract,
                    types::ContractState::GracePeriod(current_block),
                )?;
            }
            _ => (),
        }).into())
    }

    //
    fn reserve_funds(
        contract_payment_state: &mut types::ContractPaymentState<BalanceOf<T>>,
        standard_amount_due: BalanceOf<T>,
        additional_amount_due: BalanceOf<T>,
        src_twin: &pallet_tfgrid::types::Twin<T::AccountId>,
        contract: &types::Contract<T>,
        now: u64,
        total_amount_to_reserve: BalanceOf<T>,
        discount_received: types::DiscountLevel,
    ) -> DispatchResultWithPostInfo {
        <T as Config>::Currency::reserve(&src_twin.account_id, total_amount_to_reserve).map_err(
            |e| {
                log::error!("Error while reserving amount due: {:?}", e);
                e
            },
        )?;
        contract_payment_state.settle_overdrafted();
        contract_payment_state.reserve_standard_amount(standard_amount_due);
        contract_payment_state.reserve_additional_amount(additional_amount_due);
        let contract_bill = types::ContractBill {
            contract_id: contract.contract_id,
            timestamp: now,
            discount_level: discount_received.clone(),
            amount_billed: total_amount_to_reserve.saturated_into::<u128>(),
        };
        Self::deposit_event(Event::ContractBilled(contract_bill));
        Ok(().into())
    }

    fn overdraft_funds(
        contract_payment_state: &mut types::ContractPaymentState<BalanceOf<T>>,
        standard_amount_due: BalanceOf<T>,
        additional_amount_due: BalanceOf<T>,
        src_twin: &pallet_tfgrid::types::Twin<T::AccountId>,
        contract: &types::Contract<T>,
        now: u64,
    ) -> DispatchResultWithPostInfo {
        contract_payment_state.overdraft_standard_amount(standard_amount_due);
        contract_payment_state.overdraft_additional_amount(additional_amount_due);
        let reservable = Self::get_reservable_balance(&src_twin.account_id);
        <T as Config>::Currency::reserve(&src_twin.account_id, reservable).map_err(|e| {
            log::error!("Error while reserving partial amount due: {:?}", e);
            e
        })?;
        contract_payment_state.settle_partial_overdrafted(reservable);

        Self::deposit_event(Event::ContractPaymentOverdrafted {
            contract_id: contract.contract_id,
            timestamp: now,
            partial_billed_amount: reservable,
            // this is the total overdrafted amount for this contract since grace period started not per cycle
            overdrafted_amount: contract_payment_state.get_overdrafted(),
        });
        Ok(().into())
    }

    // orcastrate the distribution of rewards
    // emits RewardDistributed event
    fn remit_funds(
        contract: &types::Contract<T>,
        contract_payment_state: &mut types::ContractPaymentState<BalanceOf<T>>,
        src_twin: &pallet_tfgrid::types::Twin<T::AccountId>,
        farmer_twin: Option<pallet_tfgrid::types::Twin<T::AccountId>>,
        pricing_policy: &pallet_tfgrid::types::PricingPolicy<T::AccountId>,
    ) -> DispatchResultWithPostInfo {
        let is_deleted = matches!(contract.state, types::ContractState::Deleted(_));
        let should_distribute_rewards =
            (contract_payment_state.cycles >= T::DistributionFrequency::get() || is_deleted)
                && contract_payment_state.has_reserved_amount();
        if should_distribute_rewards {
            let standard_rewards = contract_payment_state.standard_reserved;
            let additional_rewards = contract_payment_state.additional_reserved;
            // distribute additional rewards to the farm twin
            let reminder = if let types::ContractData::RentContract(_) = &contract.contract_type {
                let dst_twin = farmer_twin.ok_or(Error::<T>::TwinNotExists)?;
                Self::transfer_reserved(
                    &src_twin.account_id,
                    &dst_twin.account_id,
                    additional_rewards,
                )
            } else {
                BalanceOf::<T>::zero()
            };
            let distributed_additional_amount = additional_rewards.saturating_sub(reminder);
            if reminder > BalanceOf::<T>::zero() {
                log::warn!(
                    "distributing additional rewards, should rewarded: {:?}, actual {:?}",
                    additional_rewards,
                    distributed_additional_amount
                );
            }
            contract_payment_state.additional_reserved = reminder;

            // distribute standard rewards
            Self::distribute_standard_rewards(
                &src_twin,
                contract.contract_id,
                contract.solution_provider_id,
                &pricing_policy,
                standard_rewards,
            )
            .map_err(|e| {
                log::error!("Error while distributing standard rewards: {:?}", e);
                e
            })?;

            contract_payment_state.standard_reserved = BalanceOf::<T>::zero();
            contract_payment_state.cycles = 0;

            Self::deposit_event(Event::RewardDistributed {
                contract_id: contract.contract_id,
                standard_rewards,
                additional_rewards: distributed_additional_amount,
            });
        }
        Ok(().into())
    }

    // transferring the held or reserved funds from the user's account to the beneficiaries (foundation, staking pool, solution providers, sales account) and burning the remainder
    fn distribute_standard_rewards(
        src_twin: &pallet_tfgrid::types::Twin<T::AccountId>,
        contract_id: u64,
        solution_provider_id: Option<u64>,
        pricing_policy: &pallet_tfgrid::types::PricingPolicy<T::AccountId>,
        amount: BalanceOf<T>,
    ) -> DispatchResult {
        log::info!(
            "Distributing standard rewards for twin {:?} with amount {:?}",
            src_twin.id,
            amount,
        );

        if amount.is_zero() {
            return Ok(().into());
        }

        let foundation_share = Perbill::from_percent(10) * amount;
        log::debug!(
            "Transferring: {:?} from twin {:?} to foundation account {:?}",
            &foundation_share,
            &src_twin.id,
            &pricing_policy.foundation_account
        );
        Self::transfer_reserved(
            &src_twin.account_id,
            &pricing_policy.foundation_account,
            foundation_share,
        );

        // Calculate staking pool share (5%)
        let staking_pool_share = Perbill::from_percent(5) * amount;
        let staking_pool_account = T::StakingPoolAccount::get();
        log::debug!(
            "Transferring: {:?} from twin {:?} to staking pool account {:?}",
            &staking_pool_share,
            &src_twin.id,
            &staking_pool_account,
        );
        Self::transfer_reserved(
            &src_twin.account_id,
            &staking_pool_account,
            staking_pool_share,
        );

        let mut sales_percentage = 50;
        let mut total_provider_share = BalanceOf::<T>::zero();
        if let Some(provider_id) = solution_provider_id {
            if let Some(solution_provider) = SolutionProviders::<T>::get(provider_id) {
                let total_take: u8 = solution_provider
                    .providers
                    .iter()
                    .map(|provider| provider.take)
                    .sum();

                sales_percentage.defensive_saturating_reduce(total_take);
                for provider in solution_provider.providers.iter() {
                    let share = Perbill::from_percent(provider.take as u32) * amount;
                    log::debug!(
                        "Transferring: {:?} from twin {:?} to provider account {:?}",
                        &share,
                        &src_twin.id,
                        &provider.who
                    );
                    Self::transfer_reserved(&src_twin.account_id, &provider.who, share);

                    total_provider_share.defensive_saturating_accrue(share);
                }
            }
        }

        let sales_share = if sales_percentage > 0 {
            let share = Perbill::from_percent(sales_percentage.into()) * amount;
            log::debug!(
                "Transferring: {:?} from twin {:?} to sales account {:?}",
                &share,
                &src_twin.id,
                &pricing_policy.certified_sales_account
            );
            Self::transfer_reserved(
                &src_twin.account_id,
                &pricing_policy.certified_sales_account,
                share,
            );
            share
        } else {
            BalanceOf::<T>::zero()
        };

        let total_distributed =
            foundation_share + staking_pool_share + total_provider_share + sales_share;
        let amount_to_burn = amount.defensive_saturating_sub(total_distributed);
        let (to_burn, reminder) = T::Currency::slash_reserved(&src_twin.account_id, amount_to_burn);
        if !reminder.is_zero() {
            // assertion
            log::warn!(
                "Failed to burn the whole amount: want {:?}, remainder {:?}",
                amount_to_burn,
                reminder
            );
        };
        log::debug!(
            "Burning: {:?} from twin {:?}",
            amount_to_burn - reminder,
            &src_twin.id
        );
        T::Burn::on_unbalanced(to_burn);
        Self::deposit_event(Event::TokensBurned {
            contract_id,
            amount: amount_to_burn - reminder,
        });
        Ok(().into())
    }

    fn transfer_reserved(
        src_account: &T::AccountId,
        dst_account: &T::AccountId,
        amount: BalanceOf<T>,
    ) -> BalanceOf<T> {
        if amount.is_zero() {
            return amount;
        }
        let res = <T as Config>::Currency::repatriate_reserved(
            &src_account,
            &dst_account,
            amount,
            BalanceStatus::Free,
        );
        match res {
            Ok(reminder) => {
                if !(reminder.is_zero()) {
                    // assertion
                    log::warn!(
                        "Failed to distribute the whole amount: want {:?}, reminder {:?}",
                        amount,
                        reminder
                    );
                }
                reminder
            }
            Err(e) => {
                log::error!("Error while repatriating reserved balance: {:?}. source: {:?}, destnation: {:?}", e, src_account, dst_account);
                amount
            }
        }
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

    fn get_stash_balance(twin_id: u32) -> BalanceOf<T> {
        let account_id = pallet_tfgrid::TwinBoundedAccountID::<T>::get(twin_id);
        match account_id {
            Some(account) => Self::get_usable_balance(&account),
            None => BalanceOf::<T>::zero(),
        }
    }

    // Get the reservable balance of an account
    // This is the balance minus the minimum balance (reservable = free - ED - Frozen )
    fn get_reservable_balance(account_id: &T::AccountId) -> BalanceOf<T> {
        let account = T::AccountStore::get(account_id);
        let free = account.free;
        let frozen = account.frozen;

        // Get the reservable balance
        let reservable = free
            .saturating_sub(
                <pallet_balances::Pallet<T> as frame_support::traits::fungible::Inspect<
                    T::AccountId,
                >>::minimum_balance(),
            )
            .saturating_sub(frozen);
        BalanceOf::<T>::saturated_from(reservable.saturated_into::<u128>())
    }

    pub fn get_current_timestamp_in_secs() -> u64 {
        <pallet_timestamp::Pallet<T>>::get().saturated_into::<u64>() / 1000
    }
}
