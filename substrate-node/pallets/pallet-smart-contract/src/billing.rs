use crate::*;
use frame_support::traits::{BalanceStatus, DefensiveSaturating};
use frame_support::{
    dispatch::{DispatchErrorWithPostInfo, DispatchResult, DispatchResultWithPostInfo, Vec},
    ensure,
    traits::{Currency, OnUnbalanced, ReservableCurrency},
};

use frame_support::traits::StoredMap;
use frame_system::{
    offchain::{SendSignedTransaction, Signer},
    pallet_prelude::BlockNumberFor,
};
use sp_core::Get;
use sp_runtime::{
    traits::{Convert, Saturating, Zero},
    Perbill, SaturatedConversion,
};
use sp_std::cmp::max;

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
            "Contracts to bill at block {:?}: {:?}",
            block_number,
            contract_ids,
        );

        let mut succeeded_contracts = Vec::new();
        let mut failed_contracts = Vec::new();
        let mut skipped_contracts = Vec::new();
        let mut missing_contracts = Vec::new();
        let mut already_sent_contracts = Vec::new();

        for contract_id in contract_ids {
            if let Some(contract) = Contracts::<T>::get(contract_id) {
                if Self::should_bill_contract(&contract) {
                    match Self::submit_signed_transaction_for_contract_billing(contract_id) {
                        Ok(()) => succeeded_contracts.push(contract_id),
                        Err(Error::<T>::OffchainSignedTxCannotSign) => {
                            failed_contracts.push(contract_id);
                        }
                        Err(Error::<T>::OffchainSignedTxAlreadySent) => {
                            already_sent_contracts.push(contract_id);
                        }
                        Err(_) => {
                            failed_contracts.push(contract_id);
                        }
                    }
                } else {
                    skipped_contracts.push(contract_id);
                }
            } else {
                missing_contracts.push(contract_id);
            }
        }

        // Log the results at the end of the function
        if !succeeded_contracts.is_empty() {
            log::info!(
                "Successfully submitted signed transactions for contracts: {:?}",
                succeeded_contracts
            );
        }

        if !already_sent_contracts.is_empty() {
            log::info!(
                "Signed transactions for contracts were already sent: {:?}",
                already_sent_contracts
            );
        }

        if !skipped_contracts.is_empty() {
            log::info!(
                "Skipped billing node contracts (no IP/CU/SU/NU to bill): {:?}",
                skipped_contracts
            );
        }

        if !failed_contracts.is_empty() {
            log::error!(
                "Failed to submit signed transactions for contracts: {:?}",
                failed_contracts
            );
        }

        if !missing_contracts.is_empty() {
            log::error!("Contracts not found in storage: {:?}", missing_contracts);
        }
    }

    fn should_bill_contract(contract: &types::Contract<T>) -> bool {
        match &contract.contract_type {
            types::ContractData::NodeContract(node_contract) => {
                let bill_ip = node_contract.public_ips > 0;
                let bill_cu_su = !NodeContractResources::<T>::get(contract.contract_id)
                    .used
                    .is_empty();
                let bill_nu = ContractBillingInformationByID::<T>::get(contract.contract_id)
                    .amount_unbilled
                    > 0;

                return bill_ip || bill_cu_su || bill_nu;
            }
            _ => true,
        }
    }

    pub fn submit_signed_transaction_for_contract_billing(
        contract_id: u64,
    ) -> Result<(), Error<T>> {
        let signer = Signer::<T, <T as pallet::Config>::AuthorityId>::all_accounts();

        if !signer.can_sign() {
            return Err(<Error<T>>::OffchainSignedTxCannotSign);
        }

        let result =
            signer.send_signed_transaction(|_acct| Call::bill_contract_for_block { contract_id });

        if result.iter().any(|(_, res)| res.is_ok()) {
            return Ok(());
        }

        Err(<Error<T>>::OffchainSignedTxAlreadySent)
    }

    // Bills a contract (NodeContract, NameContract or RentContract)
    // Calculates how much TFT is due by the user and distributes the rewards
    pub fn bill_contract(contract_id: u64) -> DispatchResultWithPostInfo {
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

        // In case contract is not a name contract ensure the node, farm and farmer twin exists
        let (farmer_twin, node_certification) =
            if !matches!(contract.contract_type, types::ContractData::NameContract(_)) {
                let node =
                    pallet_tfgrid::Nodes::<T>::get(contract.get_node_id()).ok_or_else(|| {
                        log::error!("Node not exists for contract_id: {:?}", contract_id);
                        Error::<T>::NodeNotExists
                    })?;
                let farm = pallet_tfgrid::Farms::<T>::get(node.farm_id).ok_or_else(|| {
                    log::error!("Farm not exists for node_id: {:?}", node.farm_id);
                    Error::<T>::FarmNotExists
                })?;
                let farmer_twin =
                    pallet_tfgrid::Twins::<T>::get(farm.twin_id).ok_or_else(|| {
                        log::error!("Twin not exists for farm_id: {:?}", farm.twin_id);
                        Error::<T>::TwinNotExists
                    })?;
                (Some(farmer_twin), Some(node.certification))
            } else {
                (None, None)
            };

        let mut contract_payment_state = ContractPaymentState::<T>::get(contract.contract_id);
        log::trace!(
            "Contract payment state [before billing]: {:?}",
            contract_payment_state
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
            log::info!("Waiving rent for contract_id: {:?}", contract.contract_id);
            Self::deposit_event(Event::RentWaived {
                contract_id: contract.contract_id,
            });
            // Although no billing required here, in deleted state, we should continue the billing process for contracts to distribute rewards if any and clean up storage
            if matches!(contract.state, types::ContractState::Created) {
                // So for created rent contracts, if the node is in standby, don't expect the billing cycle to advance
                return Ok(().into());
            }
        }

        // Calculate the due amount
        let (standard_amount_due, discount_received) = if should_waive_payment {
            (BalanceOf::<T>::zero(), types::DiscountLevel::None)
        } else {
            contract
                .calculate_contract_cost_tft(
                    total_usable_balance,
                    seconds_elapsed,
                    node_certification,
                )
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
        log::debug!(
            "Seconds elapsed since last bill {:?}, standard amount due: {:?}, additional amount due: {:?}, total amount due: {:?}, Discount received: {:?}",
            seconds_elapsed,
            standard_amount_due,
            additional_amount_due,
            total_amount_due,
            discount_received
        );

        // If the amount due is zero and the contract is not in deleted state, don't bill the contract (mostly node contract on a rented node)
        if total_amount_due.is_zero() && !matches!(contract.state, types::ContractState::Deleted(_))
        {
            log::info!(
                "Amount to be billed is 0 and contract state is {:?}, nothing to do with contract_id: {:?}",
                contract.state,
                contract.contract_id
            );
            return Ok(().into());
        }

        // Calculate the amount needed to be reserved from the user's balance
        // Should be the total amount due for current cycle + any overdraft from previous cycles
        let standard_amount_to_reserve =
            standard_amount_due.defensive_saturating_add(contract_payment_state.standard_overdraft);
        let additional_amount_to_reserve = additional_amount_due
            .defensive_saturating_add(contract_payment_state.additional_overdraft);
        let total_amount_to_reserve =
            standard_amount_to_reserve.defensive_saturating_add(additional_amount_to_reserve);

        let has_sufficient_fund =
            <T as Config>::Currency::can_reserve(&src_twin.account_id, total_amount_to_reserve);
        let current_block = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();

        _ = Self::manage_contract_state(&mut contract, has_sufficient_fund, current_block);

        if has_sufficient_fund {
            log::info!("Billing contract_id: {:?}, Contract state: {:?}, This cycle amount due: {:?}, Total (include previous overdraft) {:?}",
                contract.contract_id,
                contract.state,
                total_amount_due,
                total_amount_to_reserve
            );
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
                "Contract payment overdrawn for contract_id: {:?}, Contract state: {:?}, This cycle overdue: {:?}, Twin have: {:?}, Previous overdraft: {:?}",
                contract.contract_id,
                contract.state,
                total_amount_due,
                twin_usable_balance,
                contract_payment_state.get_overdraft()
            );
            Self::overdraft_funds(
                &mut contract_payment_state,
                standard_amount_due,
                additional_amount_due,
                &src_twin,
                &contract,
                now,
                twin_usable_balance,
            )?;
        }

        // Distribute rewards
        Self::remit_funds(
            &contract,
            &mut contract_payment_state,
            &src_twin,
            farmer_twin,
            &pricing_policy,
        )?;

        // Housekeeping for contracts in deleted state
        if matches!(contract.state, types::ContractState::Deleted(_)) {
            log::info!(
                "contract id {:?} in deleted state. clean up storage.",
                contract.contract_id
            );
            return Self::remove_contract(contract.contract_id);
        }

        // Reset NU amount if the contract is a node contract
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
        log::trace!(
            "Contract payment state [after billing]: {:?}",
            contract_payment_state
        );
        ContractPaymentState::<T>::insert(contract.contract_id, &contract_payment_state);

        Ok(().into())
    }

    // Handles the transition between different contract states based on the fund availability
    // May emits one of ContractGracePeriodStarted, ContractGracePeriodEnded, ContractGracePeriodElapsed events
    fn manage_contract_state(
        contract: &mut types::Contract<T>,
        has_sufficient_fund: bool,
        current_block: u64,
    ) -> DispatchResultWithPostInfo {
        match contract.state {
            types::ContractState::GracePeriod(_) if has_sufficient_fund => {
                // Manage transition from GracePeriod to Created
                log::info!("Contract {:?} is in grace period, but balance is recharged, moving to created state at block {:?}", contract.contract_id, current_block);
                Self::update_contract_state(contract, &types::ContractState::Created)?;
                Self::deposit_event(Event::ContractGracePeriodEnded {
                    contract_id: contract.contract_id,
                    node_id: contract.get_node_id(),
                    twin_id: contract.twin_id,
                });
                Self::synchronize_associated_node_contract_states(
                    contract,
                    types::ContractState::Created,
                )?;
            }
            types::ContractState::GracePeriod(grace_start) => {
                // Manage transition from GracePeriod to Deleted
                let diff = current_block.defensive_saturating_sub(grace_start);
                if diff >= T::GracePeriod::get() {
                    log::info!("Contract {:?} state changed to deleted at block {:?} due to an expired grace period. Elapsed blocks: {:?}", contract.contract_id, current_block, diff);
                    Self::deposit_event(Event::ContractGracePeriodElapsed {
                        contract_id: contract.contract_id,
                        grace_period: diff,
                    });
                    Self::update_contract_state(
                        contract,
                        &types::ContractState::Deleted(types::Cause::OutOfFunds),
                    )?;
                }
            }
            types::ContractState::Created if !has_sufficient_fund => {
                // Manage transition from Created to GracePeriod
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
                Self::synchronize_associated_node_contract_states(
                    contract,
                    types::ContractState::GracePeriod(current_block),
                )?;
            }
            _ => (),
        }
        Ok(().into())
    }

    // Holding funds from a user's account to guarantee that they are available later.
    // Emits ContractBilled event
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
                // should never happen as we called can_reserve first to check if the funds are available
                log::error!("Error while reserving amount due: {:?}", e);
                e
            },
        )?;
        contract_payment_state.settle_overdraft();
        contract_payment_state.reserve_standard_amount(standard_amount_due);
        contract_payment_state.reserve_additional_amount(additional_amount_due);
        let contract_bill = types::ContractBill {
            contract_id: contract.contract_id,
            timestamp: now,
            discount_level: discount_received.clone(),
            amount_billed: total_amount_to_reserve.saturated_into::<u128>(),
        };
        log::debug!("Contract billed: {:?}", contract_bill);
        Self::deposit_event(Event::ContractBilled(contract_bill));
        Ok(().into())
    }

    // Increasing the overdraft in the user's account
    // Emits ContractPaymentOverdrawn event
    fn overdraft_funds(
        contract_payment_state: &mut types::ContractPaymentState<BalanceOf<T>>,
        standard_amount_due: BalanceOf<T>,
        additional_amount_due: BalanceOf<T>,
        src_twin: &pallet_tfgrid::types::Twin<T::AccountId>,
        contract: &types::Contract<T>,
        now: u64,
        reservable: BalanceOf<T>,
    ) -> DispatchResultWithPostInfo {
        contract_payment_state.overdraft_standard_amount(standard_amount_due);
        contract_payment_state.overdraft_additional_amount(additional_amount_due);
        // Reserve as much as possible from the user's account to cover part of the amount overdue
        // We defensivly check if the available funds are greater than the amount overdue to avoid unintended reservation
        let over_due = standard_amount_due.saturating_add(additional_amount_due);
        let reserved = if reservable > over_due {
            log::error!("Logic error: handling overdraft while the reservable amount is greater than the amount overdue!");
            BalanceOf::<T>::zero()
        } else {
            <T as Config>::Currency::reserve(&src_twin.account_id, reservable).map_err(|e| {
                log::error!("Error while reserving partial amount due: {:?}", e);
                e
            })?;
            contract_payment_state.settle_partial_overdraft(reservable);
            reservable
        };
        let overdrawn = over_due.saturating_sub(reserved);
        log::info!("Partial amount reserved: {:?}", reserved);
        log::info!("Overdrawn: {:?}", overdrawn);
        Self::deposit_event(Event::ContractPaymentOverdrawn {
            contract_id: contract.contract_id,
            timestamp: now,
            // This is the partial amount successfully reserved from the user's account in this billing cycle
            partially_billed_amount: reserved,
            // This is the overdraft caused by insufficient funds for the contract payment in this billing cycle
            overdraft: overdrawn,
        });
        Ok(().into())
    }

    // Orchestrate the distribution of rewards
    // Emits RewardDistributed event
    // No-Op if contract neither in deleted state nor the distribution frequency is reached
    fn remit_funds(
        contract: &types::Contract<T>,
        contract_payment_state: &mut types::ContractPaymentState<BalanceOf<T>>,
        src_twin: &pallet_tfgrid::types::Twin<T::AccountId>,
        farmer_twin: Option<pallet_tfgrid::types::Twin<T::AccountId>>,
        pricing_policy: &pallet_tfgrid::types::PricingPolicy<T::AccountId>,
    ) -> DispatchResult {
        contract_payment_state.cycles.defensive_saturating_inc();
        let is_deleted = matches!(contract.state, types::ContractState::Deleted(_));
        let should_distribute_rewards =
            contract_payment_state.cycles >= T::DistributionFrequency::get() || is_deleted;
        if should_distribute_rewards && contract_payment_state.has_reserve() {
            let standard_rewards = contract_payment_state.standard_reserve;
            let additional_rewards = contract_payment_state.additional_reserve;
            // distribute additional rewards to the farm twin

            if let types::ContractData::RentContract(_) = &contract.contract_type {
                log::info!(
                    "Distributing additional rewards from twin {:?} with amount {:?}",
                    src_twin.id,
                    additional_rewards,
                );
                match Self::distribute_additional_rewards(farmer_twin, src_twin, additional_rewards)
                {
                    Ok(_) => (),
                    Err(e) => {
                        log::error!("Error while distributing additional rewards: {:?}", e);
                        if !is_deleted {
                            return Err(e);
                        }
                    }
                }
                contract_payment_state.reset_additional_reserve();
            }

            log::info!(
                "Distributing standard rewards from twin {:?} with amount {:?}",
                src_twin.id,
                standard_rewards,
            );
            // distribute standard rewards
            match Self::distribute_standard_rewards(
                &src_twin,
                contract.contract_id,
                contract.solution_provider_id,
                &pricing_policy,
                standard_rewards,
            ) {
                Ok(_) => (),
                Err(e) => {
                    log::error!("Error while distributing standard rewards: {:?}", e);
                    if !is_deleted {
                        return Err(e);
                    }
                }
            };

            contract_payment_state.reset_standard_reserve();
            contract_payment_state.reset_cycles();

            log::info!(
                "Rewards distributed for contract_id: {:?}",
                contract.contract_id
            );
            Self::deposit_event(Event::RewardDistributed {
                contract_id: contract.contract_id,
                standard_rewards,
                additional_rewards,
            });
        } else {
            log::debug!(
                "Not distributing rewards for contract_id: {:?}, cycles: {:?}, reserved amount: {:?}",
                contract.contract_id,
                contract_payment_state.cycles,
                contract_payment_state.get_reserve()
            );
        }
        Ok(().into())
    }

    fn distribute_additional_rewards(
        farmer_twin: Option<pallet_tfgrid::types::Twin<T::AccountId>>,
        src_twin: &pallet_tfgrid::types::Twin<T::AccountId>,
        additional_rewards: BalanceOf<T>,
    ) -> DispatchResult {
        if additional_rewards.is_zero() {
            return Ok(().into());
        }

        let dst_twin = farmer_twin.ok_or(Error::<T>::TwinNotExists)?;
        Self::transfer_reserved(
            &src_twin.account_id,
            &dst_twin.account_id,
            additional_rewards,
        )?;
        Ok(().into())
    }

    // Transferring the held or reserved funds from the user's account to the beneficiaries (foundation, staking pool, solution providers, sales account) and burning the remainder
    fn distribute_standard_rewards(
        src_twin: &pallet_tfgrid::types::Twin<T::AccountId>,
        contract_id: u64,
        solution_provider_id: Option<u64>,
        pricing_policy: &pallet_tfgrid::types::PricingPolicy<T::AccountId>,
        amount: BalanceOf<T>,
    ) -> DispatchResult {
        if amount.is_zero() {
            return Ok(().into());
        }

        // Calculate foundation share (10%)
        let foundation_share = Perbill::from_percent(10) * amount;
        log::debug!(
            "Transferring: {:?} (10%) from twin {:?} to foundation account {:?}",
            &foundation_share,
            &src_twin.id,
            &pricing_policy.foundation_account
        );
        Self::transfer_reserved(
            &src_twin.account_id,
            &pricing_policy.foundation_account,
            foundation_share,
        )?;

        // Calculate staking pool share (5%)
        let staking_pool_share = Perbill::from_percent(5) * amount;
        let staking_pool_account = T::StakingPoolAccount::get();
        log::debug!(
            "Transferring: {:?} (5%) from twin {:?} to staking pool account {:?}",
            &staking_pool_share,
            &src_twin.id,
            &staking_pool_account,
        );
        Self::transfer_reserved(
            &src_twin.account_id,
            &staking_pool_account,
            staking_pool_share,
        )?;
        // Calculate the sales share and solution provider share if any. Both combined should be 50%
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
                        "Transferring: {:?} ({:?}%) from twin {:?} to provider account {:?}",
                        &share,
                        &provider.take,
                        &src_twin.id,
                        &provider.who
                    );
                    Self::transfer_reserved(&src_twin.account_id, &provider.who, share)?;

                    total_provider_share.defensive_saturating_accrue(share);
                }
            }
        }

        let sales_share = if sales_percentage > 0 {
            let share = Perbill::from_percent(sales_percentage.into()) * amount;
            log::debug!(
                "Transferring: {:?} ({:?}%) from twin {:?} to sales account {:?}",
                &share,
                &sales_percentage,
                &src_twin.id,
                &pricing_policy.certified_sales_account
            );
            Self::transfer_reserved(
                &src_twin.account_id,
                &pricing_policy.certified_sales_account,
                share,
            )?;
            share
        } else {
            BalanceOf::<T>::zero()
        };

        let total_distributed =
            foundation_share + staking_pool_share + total_provider_share + sales_share;

        // Calculate the amount to burn, which is the remainder after distributing the rewards to the beneficiaries.
        // This should be 35% of the total amount, but we calculate it by subtract all previously send amounts with the initial to avoid accumulating rounding errors.
        let amount_to_burn = amount.defensive_saturating_sub(total_distributed);
        let (to_burn, remainder) =
            T::Currency::slash_reserved(&src_twin.account_id, amount_to_burn);

        log::debug!(
            "Burning: {:?} from twin {:?}",
            amount_to_burn - remainder,
            &src_twin.id
        );
        T::Burn::on_unbalanced(to_burn);
        Self::deposit_event(Event::TokensBurned {
            contract_id,
            amount: amount_to_burn - remainder,
        });
        Ok(().into())
    }

    // Transfer reserved funds to a beneficiary
    fn transfer_reserved(
        src_account: &T::AccountId,
        dst_account: &T::AccountId,
        amount: BalanceOf<T>,
    ) -> DispatchResult {
        if amount.is_zero() {
            return Ok(().into());
        }

        let result = if Self::account_exists(&dst_account) {
            // Default case: repatriate_reserved is efficient but requires the beneficiary account to exist
            <T as Config>::Currency::repatriate_reserved(
                &src_account,
                &dst_account,
                amount,
                BalanceStatus::Free,
            )
        } else {
            // Defensive measure: less efficient than repatriate_reserved, but works for non-existent accounts
            log::info!("Beneficiary account does not exist. The account will be created");
            let (slashed, remainder) =
                <T as Config>::Currency::slash_reserved(&src_account, amount);
            if remainder.is_zero() {
                <T as Config>::Currency::resolve_creating(dst_account, slashed);
            }
            Ok(remainder)
        };

        match result {
            // No remainder: the full amount was successfully deducted from the reserve and transferred to the beneficiary
            Ok(remainder) if remainder.is_zero() => Ok(().into()),
            // Partial remainder: A portion of the amount wasn't successfully deducted from the reserve.
            // This should only occur if on-chain logic has introduced a liquid restriction on the source account, or if there's a bug
            Ok(remainder) => {
                log::error!(
                    "Failed to transfer the whole amount: wanted {:?}, remainder {:?}",
                    amount,
                    remainder
                );
                Err(Error::<T>::RewardDistributionError.into())
            }
            // This should only occur if the destination account is unable to receive the funds
            Err(e) => {
                log::error!(
                    "Failed to transfer reserved balance: error: {:?}. source: {:?}, destination: {:?}",
                    e, src_account, dst_account
                );
                Err(e)
            }
        }
    }

    // Managing the states of node contracts associated with a rent contract to transition them to the appropriate state (either Created or GracePeriod).
    fn synchronize_associated_node_contract_states(
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
    // This way, we don't need to re-insert the contract every time it gets billed
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

    fn get_stash_balance(twin_id: u32) -> BalanceOf<T> {
        let account_id = pallet_tfgrid::TwinBoundedAccountID::<T>::get(twin_id);
        match account_id {
            Some(account) => Self::get_usable_balance(&account),
            None => BalanceOf::<T>::zero(),
        }
    }

    // Retrieve the liquid balance (amount that is neither reserved nor frozen).
    // The check can_reserve(get_reservable_balance(acc)) should always return true.
    // Returns free - max (ED, Frozen)
    pub fn get_usable_balance(account_id: &T::AccountId) -> BalanceOf<T> {
        let account = T::AccountStore::get(account_id);
        let free = account.free;
        let frozen = account.frozen;
        let reserved = account.reserved;
        let minimum_balance =
            <<T as Config>::Currency as Currency<T::AccountId>>::minimum_balance()
                .saturated_into::<u128>();
        // Get the reservable balance
        let reservable = free.saturating_sub(max(
            <T as pallet_balances::Config>::Balance::saturated_from(minimum_balance),
            frozen,
        ));
        log::debug!("Free balance: {:?} Reserved balance: {:?} Locked balance: {:?} Reservable balance: {:?}", free, reserved, frozen, reservable);
        let b = reservable.saturated_into::<u128>();
        BalanceOf::<T>::saturated_from(b)
    }

    pub fn get_current_timestamp_in_secs() -> u64 {
        <pallet_timestamp::Pallet<T>>::get().saturated_into::<u64>() / 1000
    }

    pub fn is_validator(account_id: T::AccountId) -> bool {
        let validators = pallet_session::Pallet::<T>::validators();

        <T as pallet_session::Config>::ValidatorIdOf::convert(account_id.clone())
            .map_or(false, |validator_id| validators.contains(&validator_id))
    }

    fn account_exists(account_id: &T::AccountId) -> bool {
        <frame_system::Pallet<T>>::account_exists(&account_id.clone().into())
    }
}
