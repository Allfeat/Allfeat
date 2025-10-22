use allfeat_primitives::{AccountId, Balance};
use frame_support::{
    PalletId, parameter_types,
    traits::tokens::{PayFromAccount, UnityAssetBalanceConversion},
};
use frame_system::{EnsureRoot, EnsureWithSuccess};
use sp_core::ConstU32;
use sp_runtime::traits::IdentityLookup;

#[cfg(feature = "runtime-benchmarks")]
use pallet_treasury::ArgumentsFactory;

use crate::{Balances, BlockNumber, DAYS, Runtime, RuntimeEvent, System, Treasury};

parameter_types! {
    pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
    pub const SpendPeriod: BlockNumber = 6 * DAYS;
    pub const PayoutSpendPeriod: BlockNumber = 30 * DAYS;
    pub const MaxBalance: Balance = Balance::MAX;

    pub TreasuryAccount: AccountId = Treasury::account_id();
}

#[cfg(feature = "runtime-benchmarks")]
pub struct PalletTreasuryArguments;
#[cfg(feature = "runtime-benchmarks")]
impl ArgumentsFactory<(), AccountId> for PalletTreasuryArguments {
    fn create_asset_kind(seed: u32) -> () {
        ()
    }

    fn create_beneficiary(seed: [u8; 32]) -> AccountId {
        AccountId::from_entropy(&mut seed.as_slice()).unwrap()
    }
}

impl pallet_treasury::Config for Runtime {
    type PalletId = TreasuryPalletId;
    type Currency = Balances;
    type RejectOrigin = EnsureRoot<Self::AccountId>;
    type RuntimeEvent = RuntimeEvent;
    type SpendPeriod = SpendPeriod;
    type Burn = ();
    type BurnDestination = ();
    type MaxApprovals = ConstU32<100>;
    type WeightInfo = pallet_treasury::weights::SubstrateWeight<Runtime>;
    type SpendFunds = ();
    type SpendOrigin = EnsureWithSuccess<EnsureRoot<Self::AccountId>, Self::AccountId, MaxBalance>;
    type AssetKind = ();
    type Beneficiary = Self::AccountId;
    type BeneficiaryLookup = IdentityLookup<Self::Beneficiary>;
    type Paymaster = PayFromAccount<Balances, TreasuryAccount>;
    type BalanceConverter = UnityAssetBalanceConversion;
    type PayoutPeriod = PayoutSpendPeriod;
    type BlockNumberProvider = System;
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = ();
}
