use super::{types::*, *};
use frame_support::{
    dispatch::DispatchErrorWithPostInfo,
    ensure, log,
    pallet_prelude::DispatchResultWithPostInfo,
    traits::{Currency, ExistenceRequirement, OnUnbalanced, WithdrawReasons},
};
use frame_system as system;
use sp_runtime::SaturatedConversion;
use sp_std::prelude::*;
use substrate_stellar_sdk as stellar;

impl<T: Config> Pallet<T> {
    pub fn mint_tft(
        tx_id: Vec<u8>,
        mut tx: MintTransaction<T::AccountId, T::BlockNumber>,
    ) -> DispatchResultWithPostInfo {
        let deposit_fee = DepositFee::<T>::get();
        ensure!(
            tx.amount > deposit_fee,
            Error::<T>::AmountIsLessThanDepositFee
        );

        // caculate amount - deposit fee
        let new_amount = tx.amount - deposit_fee;

        // transfer new amount to target
        let amount_as_balance = BalanceOf::<T>::saturated_from(new_amount);
        T::Currency::deposit_creating(&tx.target, amount_as_balance);
        // transfer deposit fee to fee wallet
        let deposit_fee_b = BalanceOf::<T>::saturated_from(deposit_fee);

        if let Some(fee_account) = FeeAccount::<T>::get() {
            T::Currency::deposit_creating(&fee_account, deposit_fee_b);
        }

        // Remove tx from storage
        MintTransactions::<T>::remove(tx_id.clone());
        // Insert into executed transactions
        let now = <system::Pallet<T>>::block_number();
        tx.block = now;
        ExecutedMintTransactions::<T>::insert(tx_id.clone(), &tx);

        Self::deposit_event(Event::MintCompleted(tx, tx_id));

        Ok(().into())
    }

    pub fn burn_tft(
        source: T::AccountId,
        target_stellar_address: Vec<u8>,
        amount: BalanceOf<T>,
    ) -> DispatchResultWithPostInfo {
        let _ = stellar::PublicKey::from_encoding(target_stellar_address.clone())
            .map_err(|_| <Error<T>>::InvalidStellarPublicKey)?;

        let withdraw_fee = WithdrawFee::<T>::get();
        let withdraw_fee_b = BalanceOf::<T>::saturated_from(withdraw_fee);
        // Make sure the user wants to swap more than the burn fee
        log::debug!("withdraw_fee {:?}", withdraw_fee_b);
        ensure!(
            amount > withdraw_fee_b,
            Error::<T>::AmountIsLessThanWithdrawFee
        );

        let usable_balance = Self::get_usable_balance(&source);
        // Make sure the user has enough usable balance to swap the amount
        ensure!(amount <= usable_balance, Error::<T>::NotEnoughBalanceToSwap);

        // transfer amount - fee to target account
        let value = T::Currency::withdraw(
            &source,
            amount,
            WithdrawReasons::TRANSFER,
            ExistenceRequirement::KeepAlive,
        )?;
        T::Burn::on_unbalanced(value);

        // transfer withdraw fee to fee wallet
        if let Some(fee_account) = FeeAccount::<T>::get() {
            T::Currency::deposit_creating(&fee_account, withdraw_fee_b);
        }

        // increment burn transaction id
        let mut burn_id = BurnTransactionID::<T>::get();
        burn_id += 1;
        BurnTransactionID::<T>::put(burn_id);

        let burn_amount_as_u64 = amount.saturated_into::<u64>() - withdraw_fee;
        Self::deposit_event(Event::BurnTransactionCreated(
            burn_id,
            source,
            target_stellar_address.clone(),
            burn_amount_as_u64,
        ));

        // Create transaction with empty signatures
        let now = <frame_system::Pallet<T>>::block_number();
        let tx = BurnTransaction {
            block: now,
            amount: burn_amount_as_u64,
            target: target_stellar_address,
            signatures: Vec::new(),
            sequence_number: 0,
        };
        BurnTransactions::<T>::insert(burn_id, &tx);

        Ok(().into())
    }

    pub fn create_stellar_refund_transaction_or_add_sig(
        validator: T::AccountId,
        tx_hash: Vec<u8>,
        target: Vec<u8>,
        amount: u64,
        signature: Vec<u8>,
        stellar_pub_key: Vec<u8>,
        sequence_number: u64,
    ) -> DispatchResultWithPostInfo {
        Self::check_if_validator_exists(validator.clone())?;

        // make sure we don't duplicate the transaction
        // ensure!(!MintTransactions::<T>::contains_key(tx_id.clone()), Error::<T>::MintTransactionExists);
        if RefundTransactions::<T>::contains_key(tx_hash.clone()) {
            return Self::add_stellar_sig_refund_transaction(
                tx_hash.clone(),
                signature,
                stellar_pub_key,
                sequence_number,
            );
        }

        let now = <frame_system::Pallet<T>>::block_number();
        let tx = RefundTransaction {
            block: now,
            target: target.clone(),
            amount,
            tx_hash: tx_hash.clone(),
            signatures: Vec::new(),
            sequence_number,
        };
        RefundTransactions::<T>::insert(tx_hash.clone(), &tx);

        Self::add_stellar_sig_refund_transaction(
            tx_hash.clone(),
            signature,
            stellar_pub_key,
            sequence_number,
        )?;

        Self::deposit_event(Event::RefundTransactionCreated(
            tx_hash.clone(),
            target,
            amount,
        ));

        Ok(().into())
    }

    pub fn propose_or_vote_stellar_mint_transaction(
        validator: T::AccountId,
        tx_id: Vec<u8>,
        target: T::AccountId,
        amount: u64,
    ) -> DispatchResultWithPostInfo {
        Self::check_if_validator_exists(validator.clone())?;
        // check if it already has been executed in the past
        ensure!(
            !ExecutedMintTransactions::<T>::contains_key(tx_id.clone()),
            Error::<T>::MintTransactionAlreadyExecuted
        );
        // make sure we don't duplicate the transaction
        // ensure!(!MintTransactions::<T>::contains_key(tx_id.clone()), Error::<T>::MintTransactionExists);
        if MintTransactions::<T>::contains_key(tx_id.clone()) {
            return Self::vote_stellar_mint_transaction(tx_id);
        }

        let now = <frame_system::Pallet<T>>::block_number();
        let tx = MintTransaction {
            amount,
            target: target.clone(),
            block: now,
            votes: 0,
        };
        MintTransactions::<T>::insert(&tx_id, &tx);

        Self::deposit_event(Event::MintTransactionProposed(
            tx_id.clone(),
            target,
            amount,
        ));

        // Vote already
        Self::vote_stellar_mint_transaction(tx_id)?;

        Ok(().into())
    }

    pub fn vote_stellar_mint_transaction(tx_id: Vec<u8>) -> DispatchResultWithPostInfo {
        let mint_transaction = MintTransactions::<T>::get(&tx_id);
        match mint_transaction {
            Some(mut tx) => {
                // increment amount of votes
                tx.votes += 1;

                // deposit voted event
                Self::deposit_event(Event::MintTransactionVoted(tx_id.clone()));

                // update the transaction
                MintTransactions::<T>::insert(&tx_id, &tx);

                let validators = Validators::<T>::get();
                // If majority aggrees on the transaction, mint tokens to target address
                if tx.votes as usize >= (validators.len() / 2) + 1 {
                    log::info!("enough votes, minting transaction...");
                    Self::mint_tft(tx_id.clone(), tx)?;
                }
            }
            None => (),
        }

        Ok(().into())
    }

    pub fn propose_stellar_burn_transaction_or_add_sig(
        validator: T::AccountId,
        tx_id: u64,
        target: Vec<u8>,
        amount: u64,
        signature: Vec<u8>,
        stellar_pub_key: Vec<u8>,
        sequence_number: u64,
    ) -> DispatchResultWithPostInfo {
        Self::check_if_validator_exists(validator.clone())?;

        // check if it already has been executed in the past
        ensure!(
            !ExecutedBurnTransactions::<T>::contains_key(tx_id),
            Error::<T>::BurnTransactionAlreadyExecuted
        );

        let mut burn_tx = BurnTransactions::<T>::get(tx_id);
        ensure!(
            BurnTransactions::<T>::contains_key(tx_id),
            Error::<T>::BurnTransactionNotExists
        );

        ensure!(
            burn_tx.amount == amount,
            Error::<T>::WrongParametersProvided
        );
        ensure!(
            burn_tx.target == target,
            Error::<T>::WrongParametersProvided
        );

        if BurnTransactions::<T>::contains_key(tx_id) {
            return Self::add_stellar_sig_burn_transaction(
                tx_id,
                signature,
                stellar_pub_key,
                sequence_number,
            );
        }

        let now = <frame_system::Pallet<T>>::block_number();

        burn_tx.block = now;
        burn_tx.sequence_number = sequence_number;
        BurnTransactions::<T>::insert(tx_id.clone(), &burn_tx);

        Self::add_stellar_sig_burn_transaction(tx_id, signature, stellar_pub_key, sequence_number)?;

        Self::deposit_event(Event::BurnTransactionProposed(tx_id, target, amount));

        Ok(().into())
    }

    pub fn add_stellar_sig_burn_transaction(
        tx_id: u64,
        signature: Vec<u8>,
        stellar_pub_key: Vec<u8>,
        sequence_number: u64,
    ) -> DispatchResultWithPostInfo {
        let mut tx = BurnTransactions::<T>::get(&tx_id);

        let validators = Validators::<T>::get();
        if tx.signatures.len() == (validators.len() / 2) + 1 {
            return Err(DispatchErrorWithPostInfo::from(
                Error::<T>::EnoughBurnSignaturesPresent,
            ));
        }

        // check if the signature already exists
        ensure!(
            !tx.signatures
                .iter()
                .any(|sig| sig.stellar_pub_key == stellar_pub_key),
            Error::<T>::BurnSignatureExists
        );
        ensure!(
            !tx.signatures.iter().any(|sig| sig.signature == signature),
            Error::<T>::BurnSignatureExists
        );

        // add the signature
        let stellar_signature = StellarSignature {
            signature,
            stellar_pub_key,
        };

        tx.sequence_number = sequence_number;
        tx.signatures.push(stellar_signature.clone());
        BurnTransactions::<T>::insert(tx_id, &tx);
        Self::deposit_event(Event::BurnTransactionSignatureAdded(
            tx_id,
            stellar_signature,
        ));

        if tx.signatures.len() >= (validators.len() / 2) + 1 {
            Self::deposit_event(Event::BurnTransactionReady(tx_id));
            BurnTransactions::<T>::insert(tx_id, tx);
        }

        Ok(().into())
    }

    pub fn set_stellar_burn_transaction_executed(
        validator: T::AccountId,
        tx_id: u64,
    ) -> DispatchResultWithPostInfo {
        Self::check_if_validator_exists(validator)?;

        ensure!(
            !ExecutedBurnTransactions::<T>::contains_key(tx_id),
            Error::<T>::BurnTransactionAlreadyExecuted
        );
        ensure!(
            BurnTransactions::<T>::contains_key(tx_id),
            Error::<T>::BurnTransactionNotExists
        );

        let tx = BurnTransactions::<T>::get(tx_id);

        BurnTransactions::<T>::remove(tx_id);
        ExecutedBurnTransactions::<T>::insert(tx_id, &tx);

        Self::deposit_event(Event::BurnTransactionProcessed(tx));

        Ok(().into())
    }

    pub fn add_stellar_sig_refund_transaction(
        tx_hash: Vec<u8>,
        signature: Vec<u8>,
        stellar_pub_key: Vec<u8>,
        sequence_number: u64,
    ) -> DispatchResultWithPostInfo {
        let mut tx = RefundTransactions::<T>::get(&tx_hash);

        let validators = Validators::<T>::get();
        if tx.signatures.len() == (validators.len() / 2) + 1 {
            return Err(DispatchErrorWithPostInfo::from(
                Error::<T>::EnoughRefundSignaturesPresent,
            ));
        }

        // check if the signature already exists
        ensure!(
            !tx.signatures
                .iter()
                .any(|sig| sig.stellar_pub_key == stellar_pub_key),
            Error::<T>::RefundSignatureExists
        );
        ensure!(
            !tx.signatures.iter().any(|sig| sig.signature == signature),
            Error::<T>::RefundSignatureExists
        );

        // add the signature
        let stellar_signature = StellarSignature {
            signature,
            stellar_pub_key,
        };

        tx.sequence_number = sequence_number;
        tx.signatures.push(stellar_signature.clone());
        RefundTransactions::<T>::insert(&tx_hash, &tx);
        Self::deposit_event(Event::RefundTransactionsignatureAdded(
            tx_hash.clone(),
            stellar_signature,
        ));
        // if more then then the half of all validators
        // submitted their signature we can emit an event that a transaction
        // is ready to be submitted to the stellar network
        if tx.signatures.len() >= (validators.len() / 2) + 1 {
            Self::deposit_event(Event::RefundTransactionReady(tx_hash.clone()));
            RefundTransactions::<T>::insert(tx_hash, tx);
        }

        Ok(().into())
    }

    pub fn set_stellar_refund_transaction_executed(
        validator: T::AccountId,
        tx_id: Vec<u8>,
    ) -> DispatchResultWithPostInfo {
        Self::check_if_validator_exists(validator)?;

        ensure!(
            !ExecutedRefundTransactions::<T>::contains_key(&tx_id),
            Error::<T>::RefundTransactionAlreadyExecuted
        );
        ensure!(
            RefundTransactions::<T>::contains_key(&tx_id),
            Error::<T>::RefundTransactionNotExists
        );

        let tx = RefundTransactions::<T>::get(&tx_id);

        RefundTransactions::<T>::remove(&tx_id);
        ExecutedRefundTransactions::<T>::insert(tx_id.clone(), &tx);

        Self::deposit_event(Event::RefundTransactionProcessed(tx));

        Ok(().into())
    }

    pub fn add_validator_account(target: T::AccountId) -> DispatchResultWithPostInfo {
        let mut validators = Validators::<T>::get();

        match validators.binary_search(&target) {
            Ok(_) => Err(Error::<T>::ValidatorExists.into()),
            // If the search fails, the caller is not a member and we learned the index where
            // they should be inserted
            Err(index) => {
                validators.insert(index, target.clone());
                Validators::<T>::put(validators);
                Ok(().into())
            }
        }
    }

    pub fn remove_validator_account(target: T::AccountId) -> DispatchResultWithPostInfo {
        let mut validators = Validators::<T>::get();

        match validators.binary_search(&target) {
            Ok(index) => {
                validators.remove(index);
                Validators::<T>::put(validators);
                Ok(().into())
            }
            Err(_) => Err(Error::<T>::ValidatorNotExists.into()),
        }
    }

    pub(crate) fn check_if_validator_exists(validator: T::AccountId) -> DispatchResultWithPostInfo {
        let validators = Validators::<T>::get();
        match validators.binary_search(&validator) {
            Ok(_) => Ok(().into()),
            Err(_) => Err(Error::<T>::ValidatorNotExists.into()),
        }
    }

    pub(crate) fn get_usable_balance(account_id: &T::AccountId) -> BalanceOf<T> {
        let balance = pallet_balances::pallet::Pallet::<T>::usable_balance(account_id);
        let b = balance.saturated_into::<u128>();
        BalanceOf::<T>::saturated_from(b)
    }
}
