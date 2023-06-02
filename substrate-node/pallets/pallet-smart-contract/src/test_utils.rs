use crate::mock::{PoolState, SmartContractModule, System, Timestamp};

use frame_support::traits::Hooks;
use parity_scale_codec::alloc::sync::Arc;
use parking_lot::RwLock;

pub fn run_to_block(n: u64, pool_state: Option<&mut Arc<RwLock<PoolState>>>) {
    Timestamp::set_timestamp((1628082000 * 1000) + (6000 * n));
    while System::block_number() <= n {
        SmartContractModule::offchain_worker(System::block_number());
        SmartContractModule::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        if pool_state.is_some() {
            pool_state
                .as_ref()
                .unwrap()
                .write()
                .execute_calls_and_check_results(System::block_number() as u64);
        }
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        SmartContractModule::on_initialize(System::block_number());
    }
}
