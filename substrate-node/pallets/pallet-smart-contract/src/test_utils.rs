#![cfg(test)]

use crate::mock::{
    bob, Origin, PoolState, SmartContractModule, System, Timestamp,
};

use codec::alloc::sync::Arc;
use frame_support::traits::Hooks;
use parking_lot::RwLock;
//use sp_runtime::offchain::testing::PoolState;

pub fn run_to_block(n: u64, mut pool_state: Option<&mut Arc<RwLock<PoolState>>>) {
    Timestamp::set_timestamp((1628082000 * 1000) + (6000 * n));
    while System::block_number() < n {
        //System::offchain_worker(System::block_number());
        SmartContractModule::offchain_worker(System::block_number());
        SmartContractModule::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        if pool_state.is_some() {
            contracts_should_be_billed(*pool_state.as_mut().unwrap());
        }
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        SmartContractModule::on_initialize(System::block_number());
    }
}

fn contracts_should_be_billed(pool_state: &mut Arc<RwLock<PoolState>>) {
    //TODO inlinde doc
    if pool_state.read().calls_to_execute.len() == 0 {
        return;
    }

    for call_to_execute in pool_state.read().calls_to_execute.iter() {
        let result = match call_to_execute.0 {
            // matches bill_contract_for_block
            crate::Call::bill_contract_for_block {
                contract_id,
                block_number,
            } => SmartContractModule::bill_contract_for_block(
                    Origin::signed(bob()),
                    contract_id,
                    block_number,
                ),
            // did not match anything => unkown call => this means you should add a capture for that function here
            _ => panic!("Unknown call!"),
        };

        let result = match result {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        };

        assert_eq!(
            call_to_execute.1,
            result,
            "The result of call to {:?} was not as expected!",
            call_to_execute.0
        );
    }

    pool_state.write().calls_to_execute.clear();
}
