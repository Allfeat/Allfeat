use frame_system::pallet_prelude::BlockNumberFor;
use pallet_evm_precompile_nfts_types::solidity::{
	CollectionConfig, CollectionSettings, MintSettings,
};
use sp_core::H160;
use sp_runtime::BuildStorage;

type AccountIdOf<R> = <R as frame_system::Config>::AccountId;

type BalanceOf<R> = <R as pallet_balances::Config>::Balance;

pub const ALICE: H160 = H160::repeat_byte(0xAA);
pub const BOB: H160 = H160::repeat_byte(0xBB);
pub const CHARLIE: H160 = H160::repeat_byte(0xCC);

pub fn mock_collection_config() -> CollectionConfig {
	CollectionConfig {
		settings: CollectionSettings::all_enabled(),
		max_supply: Default::default(),
		mint_settings: MintSettings::item_settings_all_enabled(),
	}
}

pub struct ExtBuilder<R>
where
	R: pallet_balances::Config,
{
	// endowed accounts with balances
	balances: Vec<(AccountIdOf<R>, BalanceOf<R>)>,
}

impl<R> Default for ExtBuilder<R>
where
	R: pallet_balances::Config,
{
	fn default() -> ExtBuilder<R> {
		ExtBuilder { balances: vec![] }
	}
}

impl<R> ExtBuilder<R>
where
	R: pallet_balances::Config,
	BlockNumberFor<R>: From<u32>,
{
	pub fn with_balances(mut self, balances: Vec<(AccountIdOf<R>, BalanceOf<R>)>) -> Self {
		self.balances = balances;
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::<R>::default()
			.build_storage()
			.expect("Frame system builds valid default genesis config");

		pallet_balances::GenesisConfig::<R> { balances: self.balances }
			.assimilate_storage(&mut t)
			.expect("Pallet balances storage can be assimilated");

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| frame_system::Pallet::<R>::set_block_number(1u32.into()));
		ext
	}
}
