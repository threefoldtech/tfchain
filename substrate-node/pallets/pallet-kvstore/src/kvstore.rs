use sp_std::prelude::{Vec};
use frame_support::pallet_prelude::{DispatchResultWithPostInfo, ensure};
use super::pallet::{Config, Pallet, TFKVStore, Event, Error};

impl<T: Config> Pallet<T> {
    pub fn _set(
        user: T::AccountId,
        key: Vec<u8>,
        value: Vec<u8>,
    ) -> DispatchResultWithPostInfo {
        ensure!(key.len() <= 512, Error::<T>::KeyIsTooLarge);
        ensure!(value.len() <= 2048, Error::<T>::ValueIsTooLarge);
            
        <TFKVStore<T>>::insert(&user, &key, &value);
        Self::deposit_event(Event::EntrySet(user, key, value));
        Ok(().into())
    }

    pub fn _delete(
        user: T::AccountId,
        key: Vec<u8>,
    ) -> DispatchResultWithPostInfo {

        ensure!(
            <TFKVStore<T>>::contains_key(&user, &key),
            Error::<T>::NoValueStored
        );
        let value = <TFKVStore<T>>::take(&user, &key);
        Self::deposit_event(Event::EntryTaken(user, key, value));
        Ok(().into())
    }
}
