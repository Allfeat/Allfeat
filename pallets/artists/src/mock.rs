use super::*;
use crate as pallet_artists;

use tests::ALICE;
use sp_core::H256;
use frame_support::{
	construct_runtime,
	traits::{ConstU32, ConstU64, ConstU8, GenesisBuild},
};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = ConstU64<250>;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<2>;
}

impl pallet_balances::Config for Test {
    type Balance = u64;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ConstU64<1>;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
}

impl pallet_assets::Config for Test {
    type Event = Event;
	type Balance = u64;
	type AssetId = u32;
	type Currency = Balances;
	type ForceOrigin = frame_system::EnsureRoot<u64>;
	type AssetDeposit = ConstU64<1>;
	type AssetAccountDeposit = ConstU64<10>;
	type MetadataDepositBase = ConstU64<1>;
	type MetadataDepositPerByte = ConstU64<1>;
	type ApprovalDeposit = ConstU64<1>;
	type StringLimit = ConstU32<50>;
	type Freezer = ();
	type Extra = ();
	type WeightInfo = ();
}

impl Config for Test {
    type Event = Event;
	type Balance = <Test as pallet_assets::Config>::Balance;
	type Currency = Balances;
	type ArtistId = u32;
	type AssetId = <Test as pallet_assets::Config>::AssetId;
	type Assets = Assets;
	type StringLimit = ConstU32<100>;
    type DefaultSupply = ConstU64<1_000_000_000_000>;
    type MinBalance = ConstU64<1_000_000>;
	type Decimals = ConstU8<10>;
	type WeightInfo = pallet_artists::weights::SubstrateWeight<Test>;
}

construct_runtime!(
    pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        Balances: pallet_balances,
        Assets: pallet_assets,
        Artists: pallet_artists,
    }
);

pub(crate) fn new_test_ext(empty_genesis: bool) -> sp_io::TestExternalities {
    let mut storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	let config: pallet_balances::GenesisConfig<Test> = pallet_balances::GenesisConfig {
		balances: vec![(ALICE.into(), 1_000_000_000_000)]
	};
	let mut artists_config: pallet_artists::GenesisConfig<Test> = pallet_artists::GenesisConfig::default();
	if !empty_genesis {
		artists_config = pallet_artists::GenesisConfig {
			artists: vec![
				(0, ALICE, "Genesis Artist".into(), "Genesis Artist Asset".into(), "GAA".into())
			]
		}
	}

	config.assimilate_storage(&mut storage).unwrap();
	artists_config.assimilate_storage(&mut storage).unwrap();

    let mut ext: sp_io::TestExternalities = storage.into();
    ext.execute_with(|| System::set_block_number(1));
    ext
}
