use crate::*;
use frame_support::parameter_types;
use frame_system::EnsureRoot;

parameter_types! {
	pub const MinAuthorities: u32 = 2;
}

impl pallet_validator_set::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AddRemoveOrigin = EnsureRoot<AccountId>;
	type MinAuthorities = MinAuthorities;
	type WeightInfo = pallet_validator_set::weights::SubstrateWeight<Runtime>;
}
