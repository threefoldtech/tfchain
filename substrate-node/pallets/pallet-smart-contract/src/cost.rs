use crate::pallet;
use crate::pallet::BalanceOf;
use crate::pallet::Error;
use crate::types;
use crate::types::{Contract, ContractBillingInformation};
use crate::Config;
use frame_support::dispatch::DispatchErrorWithPostInfo;
use pallet_tfgrid::types as pallet_tfgrid_types;
use sp_runtime::Percent;
use sp_runtime::SaturatedConversion;
use substrate_fixed::types::U64F64;
use tfchain_support::{resources::Resources, types::NodeCertification};

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
                BalanceOf::<T>::saturated_from(0 as u128),
                types::DiscountLevel::None,
            ));
        }

        let total_cost_tft_64 = calculate_cost_in_tft_from_musd::<T>(total_cost)?;

        // Calculate the amount due and discount received based on the total_cost amount due
        let (amount_due, discount_received) =
            calculate_discount::<T>(total_cost_tft_64, balance, certification_type);

        return Ok((amount_due, discount_received));
    }

    pub fn calculate_contract_cost(
        &self,
        pricing_policy: &pallet_tfgrid_types::PricingPolicy<T::AccountId>,
        seconds_elapsed: u64,
    ) -> Result<u64, DispatchErrorWithPostInfo> {
        let total_cost = match &self.contract_type {
            types::ContractData::DeploymentContract(_) => {
                // cost is calculated in capacity reservation contracts
                0
            }
            types::ContractData::CapacityReservationContract(capacity_reservation_contract) => {
                // Get the contract billing info to view the amount unbilled for NRU (network resource units)
                let contract_billing_info = self.get_billing_info();
                // Get the node
                let node = pallet_tfgrid::Nodes::<T>::get(capacity_reservation_contract.node_id)
                    .ok_or(Error::<T>::NodeNotExists)?;

                let contract_cost = calculate_resources_cost::<T>(
                    capacity_reservation_contract.resources.total_resources,
                    capacity_reservation_contract.public_ips,
                    seconds_elapsed,
                    &pricing_policy,
                );
                if node.resources.total_resources
                    == capacity_reservation_contract.resources.total_resources
                {
                    Percent::from_percent(pricing_policy.discount_for_dedication_nodes)
                        * contract_cost
                        + contract_billing_info.amount_unbilled
                } else {
                    contract_cost + contract_billing_info.amount_unbilled
                }
            }
            types::ContractData::NameContract(_) => {
                // bill user for name usage for number of seconds elapsed
                let total_cost_u64f64 = (U64F64::from_num(pricing_policy.unique_name.value) / 3600)
                    * U64F64::from_num(seconds_elapsed);
                total_cost_u64f64.to_num::<u64>()
            }
        };

        Ok(total_cost)
    }
}

// Calculates the total cost of a node contract.
pub fn calculate_resources_cost<T: Config>(
    resources: Resources,
    ipu: u32,
    seconds_elapsed: u64,
    pricing_policy: &pallet_tfgrid_types::PricingPolicy<T::AccountId>,
) -> u64 {
    let hru = U64F64::from_num(resources.hru) / pricing_policy.su.factor_base_1024();
    let sru = U64F64::from_num(resources.sru) / pricing_policy.su.factor_base_1024();
    let mru = U64F64::from_num(resources.mru) / pricing_policy.cu.factor_base_1024();
    let cru = U64F64::from_num(resources.cru);

    let su_used = hru / 1200 + sru / 200;
    // the pricing policy su cost value is expressed in 1 hours or 3600 seconds.
    // we bill every 3600 seconds but here we need to calculate the cost per second and multiply it by the seconds elapsed.
    let su_cost = (U64F64::from_num(pricing_policy.su.value) / 3600)
        * U64F64::from_num(seconds_elapsed)
        * su_used;
    log::debug!("su cost: {:?}", su_cost);

    let cu = calculate_cu(cru, mru);

    let cu_cost =
        (U64F64::from_num(pricing_policy.cu.value) / 3600) * U64F64::from_num(seconds_elapsed) * cu;
    log::debug!("cu cost: {:?}", cu_cost);
    let mut total_cost = su_cost + cu_cost;

    if ipu > 0 {
        let total_ip_cost = U64F64::from_num(ipu)
            * (U64F64::from_num(pricing_policy.ipu.value) / 3600)
            * U64F64::from_num(seconds_elapsed);
        log::debug!("ip cost: {:?}", total_ip_cost);
        total_cost += total_ip_cost;
    }

    return total_cost.ceil().to_num::<u64>();
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
    balance: BalanceOf<T>,
    certification_type: NodeCertification,
) -> (BalanceOf<T>, types::DiscountLevel) {
    if amount_due == 0 {
        return (
            BalanceOf::<T>::saturated_from(0 as u128),
            types::DiscountLevel::None,
        );
    }

    let balance_as_u128: u128 = balance.saturated_into::<u128>();

    // calculate amount due on a monthly basis
    // we bill every one hour so we can infer the amount due monthly (30 days ish)
    let amount_due_monthly = amount_due * 24 * 30;

    // see how many months a user can pay for this deployment given his balance
    let discount_level = U64F64::from_num(balance_as_u128) / U64F64::from_num(amount_due_monthly);

    // predefined discount levels
    // https://wiki.threefold.io/#/threefold__grid_pricing
    let discount_received = match discount_level.floor().to_num::<u64>() {
        d if d >= 3 && d < 6 => types::DiscountLevel::Default,
        d if d >= 6 && d < 12 => types::DiscountLevel::Bronze,
        d if d >= 12 && d < 36 => types::DiscountLevel::Silver,
        d if d >= 36 => types::DiscountLevel::Gold,
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

    // Guaranty tft price will never be lower than min tft price
    let min_tft_price = pallet_tft_price::MinTftPrice::<T>::get();
    let mut tft_price = avg_tft_price.max(min_tft_price);

    // Guaranty tft price will never be higher than max tft price
    let max_tft_price = pallet_tft_price::MaxTftPrice::<T>::get();
    tft_price = tft_price.min(max_tft_price);

    // TFT Price is in musd
    let tft_price_musd = U64F64::from_num(tft_price);

    // Cost is expressed in units USD, divide by 10000 to get the price in musd
    let total_cost_musd = U64F64::from_num(total_cost) / 10000;

    // Now we have the price in musd and cost in musd, make the conversion to the amount of TFT's and multiply by the chain precision (7 decimals)
    let total_cost_tft = (total_cost_musd / tft_price_musd) * U64F64::from_num(1e7 as u64);
    let total_cost_tft_64: u64 = U64F64::to_num(total_cost_tft);
    Ok(total_cost_tft_64)
}
