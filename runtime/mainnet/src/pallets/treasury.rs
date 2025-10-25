#[cfg(feature = "runtime-benchmarks")]
use core::marker::PhantomData;

use crate::weights;
use allfeat_primitives::{AccountId, Balance};
use frame_support::{
    PalletId, parameter_types,
    traits::tokens::{PayFromAccount, UnityAssetBalanceConversion},
};
use frame_system::{EnsureRoot, EnsureRootWithSuccess};
use sp_core::ConstU32;
use sp_runtime::traits::IdentityLookup;

#[cfg(feature = "runtime-benchmarks")]
use frame_support::traits::fungible::{Inspect, Mutate};
#[cfg(feature = "runtime-benchmarks")]
use pallet_treasury::ArgumentsFactory;
#[cfg(feature = "runtime-benchmarks")]
use sp_core::crypto::FromEntropy;

use crate::{Balances, BlockNumber, DAYS, Runtime, RuntimeEvent, System, Treasury};

parameter_types! {
    pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
    pub const SpendPeriod: BlockNumber = 6 * DAYS;
    pub const PayoutSpendPeriod: BlockNumber = 30 * DAYS;
    pub const MaxBalance: Balance = Balance::MAX;

    pub TreasuryAccount: AccountId = Treasury::account_id();
}

#[cfg(feature = "runtime-benchmarks")]
pub struct PalletTreasuryArguments<T>(PhantomData<T>);
#[cfg(feature = "runtime-benchmarks")]
impl<T> ArgumentsFactory<(), AccountId> for PalletTreasuryArguments<T>
where
    T: Mutate<AccountId> + Inspect<AccountId>,
{
    fn create_asset_kind(_seed: u32) -> () {
        ()
    }
    fn create_beneficiary(seed: [u8; 32]) -> AccountId {
        let account = AccountId::from_entropy(&mut seed.as_slice()).unwrap();
        <T as Mutate<_>>::mint_into(&account, <T as Inspect<_>>::minimum_balance()).unwrap();
        account
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
    type WeightInfo = weights::treasury::AllfeatWeight<Runtime>;
    type SpendFunds = ();
    type SpendOrigin = EnsureRootWithSuccess<Self::AccountId, MaxBalance>;
    type AssetKind = ();
    type Beneficiary = Self::AccountId;
    type BeneficiaryLookup = IdentityLookup<Self::Beneficiary>;
    type Paymaster = PayFromAccount<Balances, TreasuryAccount>;
    type BalanceConverter = UnityAssetBalanceConversion;
    type PayoutPeriod = PayoutSpendPeriod;
    type BlockNumberProvider = System;
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = PalletTreasuryArguments<Balances>;
}
