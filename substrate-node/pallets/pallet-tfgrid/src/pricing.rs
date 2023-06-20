use crate::*;
use frame_support::{
    dispatch::{DispatchErrorWithPostInfo, DispatchResultWithPostInfo},
    ensure,
    sp_runtime::SaturatedConversion,
};
use sp_std::vec::Vec;
use tfchain_support::types::{FarmCertification, NodeCertification};

impl<T: Config> Pallet<T> {
    pub fn _create_pricing_policy(
        name: Vec<u8>,
        su: types::Policy,
        cu: types::Policy,
        nu: types::Policy,
        ipu: types::Policy,
        unique_name: types::Policy,
        domain_name: types::Policy,
        foundation_account: T::AccountId,
        certified_sales_account: T::AccountId,
        discount_for_dedication_nodes: u8,
    ) -> DispatchResultWithPostInfo {
        ensure!(
            !PricingPolicyIdByName::<T>::contains_key(&name),
            Error::<T>::PricingPolicyExists
        );

        let mut id = PricingPolicyID::<T>::get();
        id = id + 1;

        let new_policy = types::PricingPolicy {
            version: TFGRID_PRICING_POLICY_VERSION,
            id,
            name,
            su,
            cu,
            nu,
            ipu,
            unique_name,
            domain_name,
            foundation_account,
            certified_sales_account,
            discount_for_dedication_nodes,
        };

        PricingPolicies::<T>::insert(&id, &new_policy);
        PricingPolicyIdByName::<T>::insert(&new_policy.name, &id);
        PricingPolicyID::<T>::put(id);

        Self::deposit_event(Event::PricingPolicyStored(new_policy));
        Ok(().into())
    }

    pub fn _update_pricing_policy(
        pricing_policy_id: u32,
        name: Vec<u8>,
        su: types::Policy,
        cu: types::Policy,
        nu: types::Policy,
        ipu: types::Policy,
        unique_name: types::Policy,
        domain_name: types::Policy,
        foundation_account: T::AccountId,
        certified_sales_account: T::AccountId,
        discount_for_dedication_nodes: u8,
    ) -> DispatchResultWithPostInfo {
        // Ensure pricing policy with same id already exists
        let mut pricing_policy = PricingPolicies::<T>::get(pricing_policy_id)
            .ok_or(Error::<T>::PricingPolicyNotExists)?;

        // if name exists ensure that it belongs to the same policy id
        if PricingPolicyIdByName::<T>::contains_key(&name) {
            let stored_id = PricingPolicyIdByName::<T>::get(&name);
            ensure!(
                stored_id == pricing_policy_id,
                Error::<T>::PricingPolicyWithDifferentIdExists
            );
        }

        if name != pricing_policy.name {
            PricingPolicyIdByName::<T>::remove(&pricing_policy.name);
        }

        pricing_policy.name = name;
        pricing_policy.su = su;
        pricing_policy.cu = cu;
        pricing_policy.nu = nu;
        pricing_policy.ipu = ipu;
        pricing_policy.unique_name = unique_name;
        pricing_policy.domain_name = domain_name;
        pricing_policy.foundation_account = foundation_account;
        pricing_policy.certified_sales_account = certified_sales_account;
        pricing_policy.discount_for_dedication_nodes = discount_for_dedication_nodes;

        PricingPolicies::<T>::insert(&pricing_policy_id, &pricing_policy);
        PricingPolicyIdByName::<T>::insert(&pricing_policy.name, &pricing_policy_id);
        PricingPolicyID::<T>::put(pricing_policy_id);

        Self::deposit_event(Event::PricingPolicyStored(pricing_policy));

        Ok(().into())
    }

    pub fn _create_farming_policy(
        name: Vec<u8>,
        su: u32,
        cu: u32,
        nu: u32,
        ipv4: u32,
        minimal_uptime: u16,
        policy_end: T::BlockNumber,
        immutable: bool,
        default: bool,
        node_certification: NodeCertification,
        farm_certification: FarmCertification,
    ) -> DispatchResultWithPostInfo {
        let mut id = FarmingPolicyID::<T>::get();
        id = id + 1;

        let now_block = frame_system::Pallet::<T>::block_number();

        let new_policy = types::FarmingPolicy {
            version: TFGRID_FARMING_POLICY_VERSION,
            id,
            name,
            su,
            cu,
            nu,
            ipv4,
            minimal_uptime,
            policy_created: now_block,
            policy_end,
            immutable,
            default,
            node_certification,
            farm_certification,
        };

        FarmingPoliciesMap::<T>::insert(id, &new_policy);
        FarmingPolicyID::<T>::put(id);

        Self::deposit_event(Event::FarmingPolicyStored(new_policy));

        Ok(().into())
    }

    pub fn _update_farming_policy(
        farming_policy_id: u32,
        name: Vec<u8>,
        su: u32,
        cu: u32,
        nu: u32,
        ipv4: u32,
        minimal_uptime: u16,
        policy_end: T::BlockNumber,
        default: bool,
        node_certification: NodeCertification,
        farm_certification: FarmCertification,
    ) -> DispatchResultWithPostInfo {
        ensure!(
            FarmingPoliciesMap::<T>::contains_key(farming_policy_id),
            Error::<T>::FarmingPolicyNotExists
        );

        let mut farming_policy = FarmingPoliciesMap::<T>::get(farming_policy_id);

        farming_policy.name = name;
        farming_policy.su = su;
        farming_policy.cu = cu;
        farming_policy.nu = nu;
        farming_policy.ipv4 = ipv4;
        farming_policy.minimal_uptime = minimal_uptime;
        farming_policy.policy_end = policy_end;
        farming_policy.default = default;
        farming_policy.node_certification = node_certification;
        farming_policy.farm_certification = farm_certification;

        FarmingPoliciesMap::<T>::insert(farming_policy_id, &farming_policy);

        Self::deposit_event(Event::FarmingPolicyUpdated(farming_policy));

        Ok(().into())
    }

    pub fn _set_connection_price(price: u32) -> DispatchResultWithPostInfo {
        ConnectionPrice::<T>::set(price);
        Self::deposit_event(Event::ConnectionPriceSet(price));
        Ok(().into())
    }

    pub fn get_farming_policy(
        node: &TfgridNode<T>,
    ) -> Result<types::FarmingPolicy<T::BlockNumber>, DispatchErrorWithPostInfo> {
        let mut farm = Farms::<T>::get(node.farm_id).ok_or(Error::<T>::FarmNotExists)?;

        // If there is a farming policy defined on the
        // farm policy limits, use that one
        match farm.farming_policy_limits {
            Some(mut limits) => {
                ensure!(
                    FarmingPoliciesMap::<T>::contains_key(limits.farming_policy_id),
                    Error::<T>::FarmingPolicyNotExists
                );
                match limits.end {
                    Some(end_timestamp) => {
                        let now =
                            <pallet_timestamp::Pallet<T>>::get().saturated_into::<u64>() / 1000;
                        if now > end_timestamp {
                            return Self::get_default_farming_policy();
                        }
                    }
                    None => (),
                };

                match limits.cu {
                    Some(cu_limit) => {
                        let cu = node.resources.get_cu();
                        if cu > cu_limit {
                            return Self::get_default_farming_policy();
                        }
                        limits.cu = Some(cu_limit - cu);
                    }
                    None => (),
                };

                match limits.su {
                    Some(su_limit) => {
                        let su = node.resources.get_su();
                        if su > su_limit {
                            return Self::get_default_farming_policy();
                        }
                        limits.su = Some(su_limit - su);
                    }
                    None => (),
                };

                match limits.node_count {
                    Some(node_count) => {
                        if node_count == 0 {
                            return Self::get_default_farming_policy();
                        }
                        limits.node_count = Some(node_count - 1);
                    }
                    None => (),
                };

                // Save limits when decrement is done
                farm.farming_policy_limits = Some(limits.clone());
                // Update farm in farms map
                Farms::<T>::insert(node.farm_id, &farm);
                Self::deposit_event(Event::FarmUpdated(farm));

                let farming_policy = FarmingPoliciesMap::<T>::get(limits.farming_policy_id);
                return Ok(farming_policy);
            }
            None => (),
        };

        // Set the farming policy as the last stored farming
        // policy which certifications best fit the current
        // node and farm certifications, considering that in all
        // cases a default policy would be preferable
        let mut policies: Vec<types::FarmingPolicy<T::BlockNumber>> =
            FarmingPoliciesMap::<T>::iter().map(|p| p.1).collect();

        policies.sort();
        // by reversing sorted policies we place default policies first
        // and then rank them from more certified to less certified
        policies.reverse();

        let possible_policy = policies
            .into_iter()
            .filter(|policy| {
                policy.node_certification <= node.certification
                    && policy.farm_certification <= farm.certification
            })
            .take(1)
            .next();

        match possible_policy {
            Some(policy) => Ok(policy),
            None => {
                return Err(DispatchErrorWithPostInfo::from(
                    Error::<T>::FarmingPolicyNotExists,
                ))
            }
        }
    }

    // Set the default farming policy as the last best certified stored default farming policy
    fn get_default_farming_policy(
    ) -> Result<types::FarmingPolicy<T::BlockNumber>, DispatchErrorWithPostInfo> {
        let mut policies: Vec<types::FarmingPolicy<T::BlockNumber>> =
            FarmingPoliciesMap::<T>::iter().map(|p| p.1).collect();

        policies.sort();
        // by reversing sorted policies we place default policies first
        // and then rank them from more certified to less certified
        policies.reverse();

        let possible_policy = policies
            .into_iter()
            .filter(|policy| policy.default)
            .take(1)
            .next();

        match possible_policy {
            Some(policy) => Ok(policy),
            None => {
                return Err(DispatchErrorWithPostInfo::from(
                    Error::<T>::FarmingPolicyNotExists,
                ))
            }
        }
    }
}
