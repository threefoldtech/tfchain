#![cfg(test)]

use crate::mock::{Call, Extrinsic, SmartContractModule, Origin, bob, Timestamp, System};

use codec::alloc::sync::Arc;
use frame_support::assert_ok;
use frame_support::traits::Hooks;
use parking_lot::RwLock;
use sp_core::Decode;
use sp_runtime::offchain::testing::PoolState;


pub fn run_to_block(n: u64) {
    Timestamp::set_timestamp((1628082000 * 1000) + (6000 * n));
    while System::block_number() < n {
        System::offchain_worker(System::block_number());
        SmartContractModule::offchain_worker(System::block_number());
        SmartContractModule::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        SmartContractModule::on_initialize(System::block_number());
    }
}

// TODO check for option
pub fn run_to_block_and_check_extrinsics(n: u64, pool_state: &mut Arc<RwLock<PoolState>>) {
    Timestamp::set_timestamp((1628082000 * 1000) + (6000 * n));
    while System::block_number() < n {
        System::offchain_worker(System::block_number());
        SmartContractModule::offchain_worker(System::block_number());
        SmartContractModule::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        contracts_should_be_billed(pool_state);
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        SmartContractModule::on_initialize(System::block_number());
    }
}

fn contracts_should_be_billed(pool_state: &mut Arc<RwLock<PoolState>>) {
    let contracts_to_bill: Vec<u64> = SmartContractModule::contract_to_bill_at_block(System::block_number());
    assert_eq!(pool_state.read().transactions.len(), contracts_to_bill.len());
    for i in 0..contracts_to_bill.len() {
        let encoded = pool_state.read().transactions[i].clone();
        let extrinsic: Extrinsic = Decode::decode(&mut &*encoded).unwrap();
               
        // the extrinsic call should be bill_contract_for_block with the arguments contract_id and block_number
        assert_eq!(extrinsic.call, 
            Call::SmartContractModule(
                crate::Call::bill_contract_for_block {
                    contract_id : contracts_to_bill[i], 
                    block_number: System::block_number()
                }));
        
        // now execute the call so that we can check the events
        assert_ok!(SmartContractModule::bill_contract_for_block(
            Origin::signed(bob()),
            contracts_to_bill[i],
            System::block_number()
        ));
    }
    pool_state.write().transactions.clear();
}