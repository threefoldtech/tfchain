use crate::*;
use frame_support::{
    dispatch::{DispatchErrorWithPostInfo, DispatchResultWithPostInfo},
    ensure,
    sp_runtime::SaturatedConversion,
    traits::Get,
    BoundedVec, RuntimeDebug,
};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_std::{marker::PhantomData, vec, vec::Vec};
use tfchain_support::{
    traits::ChangeNode,
    types::{Farm, FarmCertification, FarmingPolicyLimit, PublicIP, IP4},
};

impl<T: Config> Pallet<T> {
    pub fn _create_farm(
        account_id: T::AccountId,
        name: FarmNameInput<T>,
        public_ips: PublicIpListInput<T>,
    ) -> DispatchResultWithPostInfo {
        let mut id = FarmID::<T>::get();
        id = id + 1;

        let twin_id = TwinIdByAccountID::<T>::get(&account_id).ok_or(Error::<T>::TwinNotExists)?;
        let twin = Twins::<T>::get(twin_id).ok_or(Error::<T>::TwinNotExists)?;
        ensure!(
            twin.account_id == account_id,
            Error::<T>::CannotCreateFarmWrongTwin
        );

        ensure!(
            !FarmIdByName::<T>::contains_key(name.clone()),
            Error::<T>::FarmExists
        );
        let farm_name = Self::get_farm_name(name.clone())?;

        let public_ips_list = Self::get_public_ips(public_ips)?;

        let new_farm = Farm {
            version: TFGRID_FARM_VERSION,
            id,
            twin_id,
            name: farm_name,
            pricing_policy_id: 1,
            certification: FarmCertification::NotCertified,
            public_ips: public_ips_list,
            dedicated_farm: false,
            farming_policy_limits: None,
        };

        Farms::<T>::insert(id, &new_farm);
        FarmIdByName::<T>::insert(name.to_vec(), id);
        FarmID::<T>::put(id);

        Self::deposit_event(Event::FarmStored(new_farm));

        Ok(().into())
    }

    pub fn _update_farm(
        account_id: T::AccountId,
        id: u32,
        name: FarmNameInput<T>,
    ) -> DispatchResultWithPostInfo {
        let new_farm_name = Self::get_farm_name(name.clone())?;

        let twin_id = TwinIdByAccountID::<T>::get(&account_id).ok_or(Error::<T>::TwinNotExists)?;

        let mut farm = Farms::<T>::get(id).ok_or(Error::<T>::FarmNotExists)?;

        ensure!(
            farm.twin_id == twin_id,
            Error::<T>::CannotUpdateFarmWrongTwin
        );

        if FarmIdByName::<T>::contains_key(name.clone()) {
            let farm_id_by_new_name = FarmIdByName::<T>::get(name.clone());
            // if the user picks a new name but it is taken by another farmer, don't allow it
            if farm_id_by_new_name != id {
                return Err(Error::<T>::InvalidFarmName.into());
            }
        }

        let name: Vec<u8> = farm.name.into();
        // Remove stored farm by name and insert new one
        FarmIdByName::<T>::remove(name.clone());

        farm.name = new_farm_name;

        Farms::<T>::insert(id, &farm);
        FarmIdByName::<T>::insert(name, farm.id);

        Self::deposit_event(Event::FarmUpdated(farm));

        Ok(().into())
    }

    pub fn _add_stellar_payout_v2address(
        account_id: T::AccountId,
        farm_id: u32,
        stellar_address: Vec<u8>,
    ) -> DispatchResultWithPostInfo {
        let twin_id = TwinIdByAccountID::<T>::get(&account_id).ok_or(Error::<T>::TwinNotExists)?;

        let farm = Farms::<T>::get(farm_id).ok_or(Error::<T>::FarmNotExists)?;

        ensure!(
            farm.twin_id == twin_id,
            Error::<T>::CannotUpdateFarmWrongTwin
        );

        FarmPayoutV2AddressByFarmID::<T>::insert(&farm_id, &stellar_address);

        Self::deposit_event(Event::FarmPayoutV2AddressRegistered(
            farm_id,
            stellar_address,
        ));

        Ok(().into())
    }

    pub fn _set_farm_certification(
        farm_id: u32,
        certification: FarmCertification,
    ) -> DispatchResultWithPostInfo {
        let mut farm = Farms::<T>::get(farm_id).ok_or(Error::<T>::FarmNotExists)?;

        farm.certification = certification;

        Farms::<T>::insert(farm_id, &farm);

        Self::deposit_event(Event::FarmCertificationSet(farm_id, certification));

        Ok(().into())
    }

    pub fn _add_farm_ip(
        account_id: T::AccountId,
        farm_id: u32,
        ip: Ip4Input,
        gw: Gw4Input,
    ) -> DispatchResultWithPostInfo {
        let mut farm = Farms::<T>::get(farm_id).ok_or(Error::<T>::FarmNotExists)?;

        let twin = Twins::<T>::get(farm.twin_id).ok_or(Error::<T>::TwinNotExists)?;
        ensure!(
            twin.account_id == account_id,
            Error::<T>::CannotUpdateFarmWrongTwin
        );

        // Check if it's a valid IP4
        let ip4 = IP4 { ip, gw };
        ip4.is_valid().map_err(|_| Error::<T>::InvalidPublicIP)?;

        let new_ip = PublicIP {
            ip: ip4.ip,
            gateway: ip4.gw,
            contract_id: 0,
        };

        match farm
            .public_ips
            .iter()
            .position(|public_ip| public_ip.ip == new_ip.ip)
        {
            Some(_) => return Err(Error::<T>::IpExists.into()),
            None => {
                farm.public_ips
                    .try_push(new_ip)
                    .map_err(|_| Error::<T>::InvalidPublicIP)?;
                Farms::<T>::insert(farm.id, &farm);
                Self::deposit_event(Event::FarmUpdated(farm));
                return Ok(().into());
            }
        };
    }

    pub fn _remove_farm_ip(
        account_id: T::AccountId,
        id: u32,
        ip: Ip4Input,
    ) -> DispatchResultWithPostInfo {
        let mut farm = Farms::<T>::get(id).ok_or(Error::<T>::FarmNotExists)?;

        let twin = Twins::<T>::get(farm.twin_id).ok_or(Error::<T>::TwinNotExists)?;
        ensure!(
            twin.account_id == account_id,
            Error::<T>::CannotUpdateFarmWrongTwin
        );

        match farm
            .public_ips
            .iter()
            .position(|pubip| pubip.ip == ip && pubip.contract_id == 0)
        {
            Some(index) => {
                farm.public_ips.remove(index);
                Farms::<T>::insert(farm.id, &farm);
                Self::deposit_event(Event::FarmUpdated(farm));
                Ok(().into())
            }
            None => Err(Error::<T>::IpNotExists.into()),
        }
    }

    pub fn _delete_node_farm(account_id: T::AccountId, node_id: u32) -> DispatchResultWithPostInfo {
        // check if the farmer twin is authorized
        let farm_twin_id =
            TwinIdByAccountID::<T>::get(&account_id).ok_or(Error::<T>::TwinNotExists)?;
        // check if the ndode belong to said farm
        let node = Nodes::<T>::get(&node_id).ok_or(Error::<T>::NodeNotExists)?;
        let farm = Farms::<T>::get(node.farm_id).ok_or(Error::<T>::FarmNotExists)?;

        ensure!(
            Twins::<T>::contains_key(&farm.twin_id),
            Error::<T>::TwinNotExists
        );
        let farm_twin = Twins::<T>::get(farm.twin_id).ok_or(Error::<T>::TwinNotExists)?;
        ensure!(
            farm_twin_id == farm_twin.id,
            Error::<T>::FarmerNotAuthorized
        );

        let mut nodes_by_farm = NodesByFarmID::<T>::get(node.farm_id);
        let location = nodes_by_farm
            .binary_search(&node_id)
            .or(Err(Error::<T>::NodeNotExists))?;
        nodes_by_farm.remove(location);
        NodesByFarmID::<T>::insert(node.farm_id, nodes_by_farm);

        // Call node deleted
        T::NodeChanged::node_deleted(&node);

        Nodes::<T>::remove(node_id);
        NodeIdByTwinID::<T>::remove(node.twin_id);

        Self::deposit_event(Event::NodeDeleted(node_id));

        Ok(().into())
    }

    pub fn _set_farm_dedicated(farm_id: u32, dedicated: bool) -> DispatchResultWithPostInfo {
        let mut farm = Farms::<T>::get(farm_id).ok_or(Error::<T>::FarmNotExists)?;
        farm.dedicated_farm = dedicated;
        Farms::<T>::insert(farm_id, &farm);

        Self::deposit_event(Event::FarmUpdated(farm));

        Ok(().into())
    }

    pub fn _force_reset_farm_ip(farm_id: u32, ip: Ip4Input) -> DispatchResultWithPostInfo {
        ensure!(Farms::<T>::contains_key(farm_id), Error::<T>::FarmNotExists);
        let mut stored_farm = Farms::<T>::get(farm_id).ok_or(Error::<T>::FarmNotExists)?;

        match stored_farm
            .public_ips
            .iter_mut()
            .find(|pubip| pubip.ip == ip)
        {
            Some(ip) => {
                ip.contract_id = 0;
            }
            None => return Err(Error::<T>::IpNotExists.into()),
        };

        Farms::<T>::insert(stored_farm.id, &stored_farm);

        Self::deposit_event(Event::FarmUpdated(stored_farm));

        Ok(().into())
    }

    pub fn _attach_policy_to_farm(
        farm_id: u32,
        limits: Option<FarmingPolicyLimit>,
    ) -> DispatchResultWithPostInfo {
        if let Some(policy_limits) = limits {
            let farming_policy = FarmingPoliciesMap::<T>::get(policy_limits.farming_policy_id);
            let now = frame_system::Pallet::<T>::block_number();

            // Policy end is expressed in number of blocks
            if farming_policy.policy_end != T::BlockNumber::from(0 as u32)
                && now >= farming_policy.policy_created + farming_policy.policy_end
            {
                return Err(DispatchErrorWithPostInfo::from(
                    Error::<T>::FarmingPolicyExpired,
                ));
            }

            let mut farm = Farms::<T>::get(farm_id).ok_or(Error::<T>::FarmNotExists)?;
            // Save the policy limits and farm certification on the Farm object
            farm.farming_policy_limits = Some(policy_limits.clone());
            farm.certification = farming_policy.farm_certification;
            Farms::<T>::insert(farm_id, &farm);
            Self::deposit_event(Event::FarmUpdated(farm));

            // Give all the nodes in this farm the policy that is attached
            for node_id in NodesByFarmID::<T>::get(farm_id) {
                match Nodes::<T>::get(node_id) {
                    Some(mut node) => {
                        let current_node_policy =
                            FarmingPoliciesMap::<T>::get(node.farming_policy_id);
                        // If the current policy attached to the node is default one, assign it the newly created policy
                        // because we wouldn't wanna override any existing non-default policies
                        if current_node_policy.default {
                            let policy = Self::get_farming_policy(&node)?;
                            // Save the new policy ID and certification on the Node object
                            node.farming_policy_id = policy.id;
                            node.certification = policy.node_certification;
                            Nodes::<T>::insert(node_id, &node);
                            Self::deposit_event(Event::NodeUpdated(node))
                        }
                    }
                    None => continue,
                }
            }

            Self::deposit_event(Event::FarmingPolicySet(farm_id, Some(policy_limits)));
        }

        Ok(().into())
    }

    fn get_farm_name(name: FarmNameInput<T>) -> Result<FarmNameOf<T>, DispatchErrorWithPostInfo> {
        let name_parsed = <T as Config>::FarmName::try_from(name)?;
        Ok(name_parsed)
    }

    fn get_public_ips(
        public_ips: PublicIpListInput<T>,
    ) -> Result<PublicIpListOf, DispatchErrorWithPostInfo> {
        let mut public_ips_list: PublicIpListOf =
            vec![].try_into().map_err(|_| Error::<T>::InvalidPublicIP)?;

        for ip in public_ips {
            let pub_ip = PublicIP {
                ip: ip.ip,
                gateway: ip.gw,
                contract_id: 0,
            };

            if public_ips_list.contains(&pub_ip) {
                return Err(DispatchErrorWithPostInfo::from(Error::<T>::IpExists));
            }

            public_ips_list
                .try_push(pub_ip)
                .map_err(|_| Error::<T>::InvalidPublicIP)?;
        }

        Ok(public_ips_list)
    }
}

impl<T: Config> tfchain_support::traits::Tfgrid<T::AccountId, T::FarmName> for Pallet<T> {
    fn is_farm_owner(farm_id: u32, who: T::AccountId) -> bool {
        let farm = Farms::<T>::get(farm_id);
        if let Some(f) = farm {
            match Twins::<T>::get(f.twin_id) {
                Some(twin) => twin.account_id == who,
                None => false,
            }
        } else {
            false
        }
    }
}

/// A Farm name (ASCI Characters).
///
/// It is bounded in size (inclusive range [MinLength, MaxLength]) and must be a valid ipv6
#[derive(Encode, Decode, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct FarmName<T: Config>(
    pub(crate) BoundedVec<u8, T::MaxFarmNameLength>,
    PhantomData<(T, T::MaxFarmNameLength)>,
);

pub const MIN_FARM_NAME_LENGTH: u32 = 3;

impl<T: Config> TryFrom<FarmNameInput<T>> for FarmName<T> {
    type Error = Error<T>;

    /// Fallible initialization from a provided byte vector if it is below the
    /// minimum or exceeds the maximum allowed length or contains invalid ASCII
    /// characters.
    fn try_from(value: FarmNameInput<T>) -> Result<Self, Self::Error> {
        ensure!(
            value.len() >= MIN_FARM_NAME_LENGTH.saturated_into(),
            Self::Error::FarmNameTooShort
        );
        ensure!(
            value.len() <= T::MaxFarmNameLength::get() as usize,
            Self::Error::FarmNameTooLong
        );
        ensure!(validate_farm_name(&value), Self::Error::InvalidFarmName);
        Ok(Self(value, PhantomData))
    }
}

impl<T: Config> From<FarmName<T>> for Vec<u8> {
    fn from(value: FarmName<T>) -> Self {
        value.0.to_vec()
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> PartialEq for FarmName<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> Clone for FarmName<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1)
    }
}

pub fn replace_farm_name_invalid_characters(input: &[u8]) -> Vec<u8> {
    input
        .iter()
        .map(|c| match c {
            b' ' => b'_',
            b'\'' => b'-',
            b';' => b'_',
            _ => *c,
        })
        .collect()
}

fn validate_farm_name(input: &[u8]) -> bool {
    input
        .iter()
        .all(|c| matches!(c, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_'))
}
