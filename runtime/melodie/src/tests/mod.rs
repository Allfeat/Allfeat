use crate::Runtime;
use sp_runtime::BuildStorage;

pub mod fee_report;

pub fn new_test_ext() -> sp_io::TestExternalities {
    let t = frame_system::GenesisConfig::<Runtime>::default()
        .build_storage()
        .unwrap();

    sp_io::TestExternalities::new(t)
}
