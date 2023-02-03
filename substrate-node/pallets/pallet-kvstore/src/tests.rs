use super::Event as KvStoreEvent;
use crate::{self as pallet_kvstore, Config};
use frame_support::{assert_ok, construct_runtime, parameter_types, traits::ConstU32};
use frame_system::{EventRecord, Phase};
use sp_core::H256;
use sp_io::TestExternalities;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};
use sp_std::convert::{TryFrom, TryInto};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<TestRuntime>;
type Block = frame_system::mocking::MockBlock<TestRuntime>;

construct_runtime!(
    pub enum TestRuntime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        TFKVStoreModule: pallet_kvstore::{Pallet, Call, Storage, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for TestRuntime {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type RuntimeOrigin = RuntimeOrigin;
    type Index = u64;
    type RuntimeCall = RuntimeCall;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = BlockHashCount;
    type DbWeight = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

impl Config for TestRuntime {
    type RuntimeEvent = RuntimeEvent;
}

struct ExternalityBuilder;

impl ExternalityBuilder {
    pub fn build() -> TestExternalities {
        let storage = frame_system::GenesisConfig::default()
            .build_storage::<TestRuntime>()
            .unwrap();
        let mut ext = TestExternalities::from(storage);
        ext.execute_with(|| System::set_block_number(1));
        ext
    }
}

#[test]
fn test_set_and_get() {
    ExternalityBuilder::build().execute_with(|| {
        let key = "name";
        let value = "nametest";
        // make sure Entry does not exists
        assert_ok!(TFKVStoreModule::set(
            RuntimeOrigin::signed(1),
            key.as_bytes().to_vec(),
            value.as_bytes().to_vec()
        ));

        let our_events = System::events();

        assert_eq!(
            our_events.contains(&record(RuntimeEvent::TFKVStoreModule(KvStoreEvent::<
                TestRuntime,
            >::EntrySet(
                1,
                key.as_bytes().to_vec(),
                value.as_bytes().to_vec(),
            )))),
            true
        );

        let entry_value = TFKVStoreModule::key_value_store(1, key.as_bytes().to_vec());
        assert_eq!(entry_value, value.as_bytes().to_vec());
    })
}

#[test]
fn test_delete() {
    ExternalityBuilder::build().execute_with(|| {
        let key = "Address";
        let value = "Cairo";
        assert_ok!(TFKVStoreModule::set(
            RuntimeOrigin::signed(1),
            key.as_bytes().to_vec(),
            value.as_bytes().to_vec()
        ));

        let our_events = System::events();

        assert_eq!(
            our_events.contains(&record(RuntimeEvent::TFKVStoreModule(KvStoreEvent::<
                TestRuntime,
            >::EntrySet(
                1,
                key.as_bytes().to_vec(),
                value.as_bytes().to_vec(),
            )))),
            true
        );

        assert_ok!(TFKVStoreModule::delete(
            RuntimeOrigin::signed(1),
            key.as_bytes().to_vec()
        ));

        let our_events = System::events();

        assert_eq!(
            our_events.contains(&record(RuntimeEvent::TFKVStoreModule(KvStoreEvent::<
                TestRuntime,
            >::EntryTaken(
                1,
                key.as_bytes().to_vec(),
                value.as_bytes().to_vec(),
            )))),
            true
        );

        // check if value get deleted
        let entry_value = TFKVStoreModule::key_value_store(1, key.as_bytes().to_vec());
        let expected_value = "".as_bytes().to_vec();
        assert_eq!(entry_value, expected_value);
    })
}

fn record(event: RuntimeEvent) -> EventRecord<RuntimeEvent, H256> {
    EventRecord {
        phase: Phase::Initialization,
        event,
        topics: vec![],
    }
}
