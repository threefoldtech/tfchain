use crate::*;
use frame_support::{traits::Get, traits::OnRuntimeUpgrade, weights::Weight};
use log::{debug, info};
use scale_info::prelude::string::String;
use sp_core::ConstU32;
use sp_runtime::{BoundedVec, Saturating};
use sp_std::marker::PhantomData;
use tfchain_support::types::{PublicIP, IP4};

#[cfg(feature = "try-runtime")]
use parity_scale_codec::{Decode, Encode};
#[cfg(feature = "try-runtime")]
use sp_std::vec::Vec;

pub struct FixFarmPublicIps<T: Config>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for FixFarmPublicIps<T> {
    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
        info!("current pallet version: {:?}", PalletVersion::<T>::get());
        assert!(PalletVersion::<T>::get() >= types::StorageVersion::V16Struct);

        let farms_count: u64 = Farms::<T>::iter().count() as u64;
        info!(
            "🔎 FixFarmPublicIps pre migration: Number of existing farms {:?}",
            farms_count
        );

        info!("👥  TFGrid pallet to V17 passes PRE migrate checks ✅",);
        Ok(farms_count.encode())
    }

    fn on_runtime_upgrade() -> Weight {
        if PalletVersion::<T>::get() == types::StorageVersion::V16Struct {
            fix_farm_public_ips::<T>()
        } else {
            info!(" >>> Unused TFGrid pallet V17 migration");
            Weight::zero()
        }
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(pre_farms_count: Vec<u8>) -> Result<(), &'static str> {
        info!("current pallet version: {:?}", PalletVersion::<T>::get());
        assert!(PalletVersion::<T>::get() >= types::StorageVersion::V17Struct);

        // Check number of farms against pre-check result
        let pre_farms_count: u64 = Decode::decode(&mut pre_farms_count.as_slice())
            .expect("the state parameter should be something that was generated by pre_upgrade");
        assert_eq!(
            Farms::<T>::iter().count() as u64,
            pre_farms_count,
            "Number of farms migrated does not match"
        );

        info!(
            "👥  TFGrid pallet migration to {:?} passes POST migrate checks ✅",
            Pallet::<T>::pallet_version()
        );

        Ok(())
    }
}

pub fn fix_farm_public_ips<T: Config>() -> frame_support::weights::Weight {
    info!(" >>> Migrating farms storage...");

    let mut r = 0u64;
    let mut w = 0u64;

    let farms = Farms::<T>::iter().collect::<Vec<_>>();
    r.saturating_accrue(farms.len() as u64);

    for (_, mut farm) in Farms::<T>::iter() {
        r.saturating_inc();
        let size = farm.public_ips.len();

        farm.public_ips.retain(|pubip| {
            let ip4 = IP4 {
                ip: pubip.ip.clone(),
                gw: pubip.gateway.clone(),
            };
            ip4.is_valid().is_ok()
        });

        // Update farm only if some invalid IP was found
        if farm.public_ips.len() < size {
            debug!("Farm #{:?}: invalid IP found", farm.id);
            debug!(
                " public ips were: {:?}",
                public_ips_to_string(Farms::<T>::get(farm.id).unwrap().public_ips)
            );

            Farms::<T>::insert(farm.id, &farm);

            debug!(
                " public ips now: {:?}",
                public_ips_to_string(Farms::<T>::get(farm.id).unwrap().public_ips)
            );
            w.saturating_inc();
        }
    }

    info!(" <<< Farms storage updated! Migrated {} Farms ✅", w);

    // Update pallet storage version
    PalletVersion::<T>::set(types::StorageVersion::V17Struct);
    w.saturating_inc();
    info!(" <<< Storage version upgraded");

    // Return the weight consumed by the migration.
    T::DbWeight::get().reads_writes(r, w)
}

fn public_ips_to_string(public_ips: BoundedVec<PublicIP, ConstU32<256>>) -> String {
    let mut s = String::new();
    for pub_ip in public_ips {
        s.push_str("{ ip: ");
        s.push_str(&String::from_utf8_lossy(&pub_ip.ip));
        s.push_str(", gw: ");
        s.push_str(&String::from_utf8_lossy(&pub_ip.gateway));
        s.push_str("} ");
    }
    s
}
