use crate::pallet;
use crate::pallet::BalanceOf;
use crate::pallet::Error;
use crate::types;
use crate::types::{Contract, ContractBillingInformation, ServiceContract, ServiceContractBill};
use crate::Config;
use crate::DedicatedNodesExtraFee;
use frame_support::{dispatch::DispatchErrorWithPostInfo, traits::Get};
use log;
use pallet_tfgrid::types as pallet_tfgrid_types;
use sp_runtime::{Percent, SaturatedConversion, traits::Zero};
use substrate_fixed::types::U64F64;
use tfchain_support::{
    constants::time::{SECS_PER_HOUR, SECS_PER_MONTH},
    resources::Resources,
    types::NodeCertification,
};

impl<T: Config> Contract<T> {
    pub fn get_billing_info(&self) -> ContractBillingInformation {
        pallet::ContractBillingInformationByID::<T>::get(self.contract_id)
    }

    pub fn calculate_contract_cost_tft(
        &self,
        balance: BalanceOf<T>,
        seconds_elapsed: u64,
    ) -> Result<(BalanceOf<T>, types::DiscountLevel), DispatchErrorWithPostInfo> {
        // Fetch the default pricing policy and certification type
        let pricing_policy = pallet_tfgrid::PricingPolicies::<T>::get(1).unwrap();
        let certification_type = NodeCertification::Diy;

        // Calculate the cost for a contract, can be any of:
        // - NodeContract
        // - RentContract
        // - NameContract
        let total_cost = self.calculate_contract_cost(&pricing_policy, seconds_elapsed)?;
        // If cost is 0, reinsert to be billed at next interval
        if total_cost == 0 {
            return Ok((
                BalanceOf::<T>::zero(),
                types::DiscountLevel::None,
            ));
        }

        let total_cost_tft_64 = calculate_cost_in_tft_from_musd::<T>(total_cost)?;

        // Calculate the amount due and discount received based on the total_cost amount due
        let (amount_due, discount_received) = calculate_discount::<T>(
            total_cost_tft_64,
            seconds_elapsed,
            balance,
            certification_type,
        );

        Ok((amount_due, discount_received))
    }

    pub fn calculate_contract_cost(
        &self,
        pricing_policy: &pallet_tfgrid_types::PricingPolicy<T::AccountId>,
        seconds_elapsed: u64,
    ) -> Result<u64, DispatchErrorWithPostInfo> {
        let total_cost = match &self.contract_type {
            // Calculate total cost for a node contract
            types::ContractData::NodeContract(node_contract) => {
                // Get the contract billing info to view the amount unbilled for NRU (network resource units)
                let contract_billing_info = self.get_billing_info();
                // Make sure the node exists
                if pallet_tfgrid::Nodes::<T>::get(node_contract.node_id).is_none() {
                    return Err(DispatchErrorWithPostInfo::from(Error::<T>::NodeNotExists));
                }

                // We know the contract is using resources, now calculate the cost for each used resource
                let node_contract_resources =
                    pallet::Pallet::<T>::node_contract_resources(self.contract_id);

                let mut bill_resources = true;
                // If this node contract is deployed on a node which has a rent contract
                // We can ignore billing for the resources used by this node contract
                if pallet::ActiveRentContractForNode::<T>::contains_key(node_contract.node_id) {
                    bill_resources = false
                }

                let contract_cost = calculate_resources_cost::<T>(
                    node_contract_resources.used,
                    node_contract.public_ips,
                    seconds_elapsed,
                    &pricing_policy,
                    bill_resources,
                );
                contract_cost + contract_billing_info.amount_unbilled
            }
            types::ContractData::RentContract(rent_contract) => {
                let node = pallet_tfgrid::Nodes::<T>::get(rent_contract.node_id)
                    .ok_or(Error::<T>::NodeNotExists)?;

                let contract_cost = calculate_resources_cost::<T>(
                    node.resources,
                    0,
                    seconds_elapsed,
                    &pricing_policy,
                    true,
                );
                Percent::from_percent(pricing_policy.discount_for_dedication_nodes) * contract_cost
            }
            // Calculate total cost for a name contract
            types::ContractData::NameContract(_) => {
                // bill user for name usage for number of seconds elapsed
                let total_cost_u64f64 = (U64F64::from_num(pricing_policy.unique_name.value)
                    / U64F64::from_num(T::BillingReferencePeriod::get()))
                    * U64F64::from_num(seconds_elapsed);
                total_cost_u64f64.to_num::<u64>()
            }
        };

        Ok(total_cost)
    }

    // Calculates the cost of extra fee for a dedicated node in TFT.
    pub fn calculate_extra_fee_cost_tft(
        &self,
        node_id: u32,
        seconds_elapsed: u64,
    ) -> Result<BalanceOf<T>, DispatchErrorWithPostInfo> {
        let cost = calculate_extra_fee_cost::<T>(node_id, seconds_elapsed);
        if cost == 0 {
            return Ok(BalanceOf::<T>::zero());
        }
        let cost_tft = calculate_cost_in_tft_from_musd::<T>(cost)?;
    
        Ok(BalanceOf::<T>::saturated_from(cost_tft))
    }
}

impl ServiceContract {
    pub fn calculate_bill_cost_tft<T: Config>(
        &self,
        service_bill: ServiceContractBill,
    ) -> Result<BalanceOf<T>, DispatchErrorWithPostInfo> {
        // Calculate the cost in mUSD for service contract bill
        let total_cost = self.calculate_bill_cost::<T>(service_bill);

        if total_cost == 0 {
            return Ok(BalanceOf::<T>::zero());
        }

        // Calculate the cost in TFT for service contract
        let total_cost_tft_64 = calculate_cost_in_tft_from_musd::<T>(total_cost)?;

        // convert to balance object
        let amount_due: BalanceOf<T> = BalanceOf::<T>::saturated_from(total_cost_tft_64);

        return Ok(amount_due);
    }

    pub fn calculate_bill_cost<T: Config>(&self, service_bill: ServiceContractBill) -> u64 {
        // bill user for service usage for elpased usage (window) in seconds
        let contract_cost = U64F64::from_num(self.base_fee) * U64F64::from_num(service_bill.window)
            / U64F64::from_num(T::BillingReferencePeriod::get())
            + U64F64::from_num(service_bill.variable_amount);
        contract_cost.round().to_num::<u64>()
    }
}

// Calculates the total cost of a node contract.
pub fn calculate_resources_cost<T: Config>(
    resources: Resources,
    ipu: u32,
    seconds_elapsed: u64,
    pricing_policy: &pallet_tfgrid_types::PricingPolicy<T::AccountId>,
    bill_resources: bool,
) -> u64 {
    let mut total_cost = U64F64::from_num(0);

    if bill_resources {
        let hru = U64F64::from_num(resources.hru) / pricing_policy.su.factor_base_1024();
        let sru = U64F64::from_num(resources.sru) / pricing_policy.su.factor_base_1024();
        let mru = U64F64::from_num(resources.mru) / pricing_policy.cu.factor_base_1024();
        let cru = U64F64::from_num(resources.cru);

        let su_used = hru / 1200 + sru / 200;
        // the pricing policy su cost value is expressed in 1 hours or 3600 seconds.
        // we bill every 3600 seconds but here we need to calculate the cost per second and multiply it by the seconds elapsed.
        let su_cost = (U64F64::from_num(pricing_policy.su.value)
            / U64F64::from_num(T::BillingReferencePeriod::get()))
            * U64F64::from_num(seconds_elapsed)
            * su_used;
        log::debug!("su cost: {:?}", su_cost);

        let cu = calculate_cu(cru, mru);

        let cu_cost = (U64F64::from_num(pricing_policy.cu.value)
            / U64F64::from_num(T::BillingReferencePeriod::get()))
            * U64F64::from_num(seconds_elapsed)
            * cu;
        log::debug!("cu cost: {:?}", cu_cost);
        total_cost = su_cost + cu_cost;
    }

    if ipu > 0 {
        let total_ip_cost = U64F64::from_num(ipu)
            * (U64F64::from_num(pricing_policy.ipu.value)
                / U64F64::from_num(T::BillingReferencePeriod::get()))
            * U64F64::from_num(seconds_elapsed);
        log::debug!("ip cost: {:?}", total_ip_cost);
        total_cost += total_ip_cost;
    }

    return total_cost.ceil().to_num::<u64>();
}

// Calculates the cost of extra fee for a dedicated node.
// TODO: check why this gives different results when giving it a low input (multiple billing cycles) and a high input (total seconds elapsed)
pub fn calculate_extra_fee_cost<T: Config>(node_id: u32, seconds_elapsed: u64) -> u64 {
    match DedicatedNodesExtraFee::<T>::get(node_id) {
        Some(fee_musd_per_month) => {
            // Convert fee from mUSD to units USD
            let fee_units_usd_per_month = fee_musd_per_month * 10000;
            (U64F64::from_num(fee_units_usd_per_month * seconds_elapsed)
                / U64F64::from_num(SECS_PER_MONTH))
            .to_num::<u64>()
        }
        None => 0,
    }
}

// cu1 = MAX(cru/2, mru/4)
// cu2 = MAX(cru, mru/8)
// cu3 = MAX(cru/4, mru/2)

// CU = MIN(cu1, cu2, cu3)
pub(crate) fn calculate_cu(cru: U64F64, mru: U64F64) -> U64F64 {
    let mru_used_1 = mru / 4;
    let cru_used_1 = cru / 2;
    let cu1 = if mru_used_1 > cru_used_1 {
        mru_used_1
    } else {
        cru_used_1
    };

    let mru_used_2 = mru / 8;
    let cru_used_2 = cru;
    let cu2 = if mru_used_2 > cru_used_2 {
        mru_used_2
    } else {
        cru_used_2
    };

    let mru_used_3 = mru / 2;
    let cru_used_3 = cru / 4;
    let cu3 = if mru_used_3 > cru_used_3 {
        mru_used_3
    } else {
        cru_used_3
    };

    let mut cu = if cu1 > cu2 { cu2 } else { cu1 };

    cu = if cu > cu3 { cu3 } else { cu };

    cu
}

// Calculates the discount that will be applied to the billing of the contract
// Returns an amount due as balance object and a static string indicating which kind of discount it received
// (default, bronze, silver, gold or none)
pub fn calculate_discount<T: Config>(
    amount_due: u64,
    seconds_elapsed: u64,
    balance: BalanceOf<T>,
    certification_type: NodeCertification,
) -> (BalanceOf<T>, types::DiscountLevel) {
    if amount_due == 0 {
        return (
            BalanceOf::<T>::zero(),
            types::DiscountLevel::None,
        );
    }

    // calculate amount due on a monthly basis
    // first get the normalized amount per hour
    let amount_due_hourly = U64F64::from_num(amount_due) * U64F64::from_num(seconds_elapsed)
        / U64F64::from_num(SECS_PER_HOUR);
    // then we can infer the amount due monthly (30 days ish)
    let amount_due_monthly = (amount_due_hourly * 24 * 30).round().to_num::<u64>();

    // see how many months a user can pay for this deployment given his balance
    let balance_as_u128: u128 = balance.saturated_into::<u128>();
    let discount_level = U64F64::from_num(balance_as_u128) / U64F64::from_num(amount_due_monthly);

    // predefined discount levels
    // https://library.threefold.me/info/manual/#/threefold__pricing?id=discount-levels
    let discount_received = match discount_level {
        d if d >= 1.5 && d < 3 => types::DiscountLevel::Default,
        d if d >= 3 && d < 6 => types::DiscountLevel::Bronze,
        d if d >= 6 && d < 18 => types::DiscountLevel::Silver,
        d if d >= 18 => types::DiscountLevel::Gold,
        _ => types::DiscountLevel::None,
    };

    // calculate the new amount due given the discount
    let mut amount_due = U64F64::from_num(amount_due) * discount_received.price_multiplier();

    // Certified capacity costs 25% more
    if certification_type == NodeCertification::Certified {
        amount_due = amount_due * U64F64::from_num(1.25);
    }

    // convert to balance object
    let amount_due: BalanceOf<T> =
        BalanceOf::<T>::saturated_from(amount_due.ceil().to_num::<u64>());

    (amount_due, discount_received)
}

pub fn calculate_cost_in_tft_from_musd<T: Config>(
    total_cost: u64,
) -> Result<u64, DispatchErrorWithPostInfo> {
    let avg_tft_price = pallet_tft_price::AverageTftPrice::<T>::get();

    // Guarantee tft price will never be lower than min tft price
    let min_tft_price = pallet_tft_price::MinTftPrice::<T>::get();
    let mut tft_price = avg_tft_price.max(min_tft_price);

    // Guarantee tft price will never be higher than max tft price
    let max_tft_price = pallet_tft_price::MaxTftPrice::<T>::get();
    tft_price = tft_price.min(max_tft_price);

    // TFT Price is in musd so lets convert to units usd
    let tft_price_units_usd = U64F64::from_num(tft_price) * 10000;

    // Now we have the price in musd and cost in musd, make the conversion to the amount of TFT's and multiply by the chain precision (7 decimals)
    let total_cost_tft = U64F64::from_num(total_cost) / tft_price_units_usd;

    Ok((total_cost_tft * U64F64::from_num(10u64.pow(7))).to_num::<u64>())
}
