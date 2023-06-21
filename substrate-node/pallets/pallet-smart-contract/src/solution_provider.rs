use crate::*;
use frame_support::{
    dispatch::{DispatchErrorWithPostInfo, DispatchResultWithPostInfo},
    ensure,
};

impl<T: Config> Pallet<T> {
    pub fn _create_solution_provider(
        description: Vec<u8>,
        link: Vec<u8>,
        providers: Vec<types::Provider<T::AccountId>>,
    ) -> DispatchResultWithPostInfo {
        let total_take: u8 = providers.iter().map(|provider| provider.take).sum();
        ensure!(total_take <= 50, Error::<T>::InvalidProviderConfiguration);

        let mut id = SolutionProviderID::<T>::get();
        id = id + 1;

        let solution_provider = types::SolutionProvider {
            solution_provider_id: id,
            providers,
            description,
            link,
            approved: false,
        };

        SolutionProviderID::<T>::put(id);
        SolutionProviders::<T>::insert(id, &solution_provider);

        Self::deposit_event(Event::SolutionProviderCreated(solution_provider));

        Ok(().into())
    }

    pub fn _approve_solution_provider(
        solution_provider_id: u64,
        approve: bool,
    ) -> DispatchResultWithPostInfo {
        ensure!(
            SolutionProviders::<T>::contains_key(solution_provider_id),
            Error::<T>::NoSuchSolutionProvider
        );

        if let Some(mut solution_provider) = SolutionProviders::<T>::get(solution_provider_id) {
            solution_provider.approved = approve;
            SolutionProviders::<T>::insert(solution_provider_id, &solution_provider);
            Self::deposit_event(Event::SolutionProviderApproved(
                solution_provider_id,
                approve,
            ));
        }

        Ok(().into())
    }

    pub fn _attach_solution_provider_id(
        account_id: T::AccountId,
        contract_id: u64,
        solution_provider_id: u64,
    ) -> DispatchResultWithPostInfo {
        let solution_provider = SolutionProviders::<T>::get(solution_provider_id)
            .ok_or(Error::<T>::NoSuchSolutionProvider)?;
        ensure!(
            solution_provider.approved,
            Error::<T>::SolutionProviderNotApproved
        );

        let mut contract = Contracts::<T>::get(contract_id).ok_or(Error::<T>::ContractNotExists)?;

        let twin_id = pallet_tfgrid::TwinIdByAccountID::<T>::get(&account_id)
            .ok_or(Error::<T>::TwinNotExists)?;

        ensure!(
            contract.twin_id == twin_id,
            Error::<T>::UnauthorizedToChangeSolutionProviderId
        );

        match contract.solution_provider_id {
            Some(_) => {
                return Err(DispatchErrorWithPostInfo::from(
                    Error::<T>::UnauthorizedToChangeSolutionProviderId,
                ))
            }
            None => {
                contract.solution_provider_id = Some(solution_provider_id);
                Contracts::<T>::insert(contract_id, &contract);
                Self::deposit_event(Event::ContractUpdated(contract));
            }
        };

        Ok(().into())
    }

    pub fn validate_solution_provider(
        solution_provider_id: Option<u64>,
    ) -> DispatchResultWithPostInfo {
        if let Some(provider_id) = solution_provider_id {
            ensure!(
                SolutionProviders::<T>::contains_key(provider_id),
                Error::<T>::NoSuchSolutionProvider
            );

            if let Some(solution_provider) = SolutionProviders::<T>::get(provider_id) {
                ensure!(
                    solution_provider.approved,
                    Error::<T>::SolutionProviderNotApproved
                );
                return Ok(().into());
            }
        }
        Ok(().into())
    }
}
