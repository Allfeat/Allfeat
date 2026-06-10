// This file is part of Allfeat.

// Copyright (C) 2022-2025 Allfeat.
// SPDX-License-Identifier: GPL-3.0-or-later

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

#[path = "xcm.rs"]
mod xcm_config;

#[cfg(feature = "runtime-benchmarks")]
use core::marker::PhantomData;
use cumulus_pallet_parachain_system::RelayNumberMonotonicallyIncreases;
use polkadot_sdk::{
	cumulus_pallet_aura_ext, cumulus_pallet_weight_reclaim, cumulus_pallet_xcmp_queue,
	cumulus_primitives_core::{AggregateMessageOrigin, ParaId},
	frame_support,
	frame_support::{
		PalletId, derive_impl,
		dispatch::DispatchClass,
		pallet_prelude::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen},
		parameter_types,
		traits::{
			ConstBool, ConstU8, ConstU16, ConstU32, ConstU64, EitherOfDiverse, EqualPrivilegeOnly,
			Imbalance, InstanceFilter, LinearStoragePrice, OnUnbalanced, TransformOrigin,
			VariantCountOf,
			fungible::{Balanced, Credit, HoldConsideration},
			tokens::{PayFromAccount, UnityAssetBalanceConversion},
		},
		weights::{ConstantMultiplier, Weight},
	},
	frame_system,
	frame_system::{EnsureRoot, EnsureRootWithSuccess, limits::BlockWeights},
	pallet_aura, pallet_authorship, pallet_balances, pallet_collator_selection,
	pallet_message_queue, pallet_meta_tx, pallet_multisig, pallet_preimage, pallet_proxy,
	pallet_scheduler, pallet_session, pallet_sudo, pallet_timestamp, pallet_transaction_payment,
	pallet_treasury, pallet_utility, pallet_verify_signature,
	pallet_xcm::{EnsureXcm, IsVoiceOfBody},
	parachains_common::message_queue::{NarrowOriginToSibling, ParaIdToSibling},
	polkadot_runtime_common::{BlockHashCount, xcm_sender::NoPriceForMessageDelivery},
	sp_consensus_aura::sr25519::AuthorityId as AuraId,
	sp_runtime,
	sp_runtime::{
		Perbill,
		traits::{BlakeTwo256, IdentityLookup, Verify},
	},
	sp_version::RuntimeVersion,
	staging_parachain_info as parachain_info,
	staging_xcm::latest::prelude::BodyId,
};
#[cfg(feature = "runtime-benchmarks")]
use polkadot_sdk::{
	frame_support::traits::fungible::{Inspect, Mutate},
	pallet_treasury::ArgumentsFactory,
	sp_core::crypto::FromEntropy,
};

pub use allfeat_runtime_common::{RuntimeBlockLength, SlowAdjustingFeeUpdate, currency::deposit};

use super::{
	AVERAGE_ON_INITIALIZE_RATIO, AccountId, Aura, Authorship, Balance, Balances, Block,
	BlockNumber, CollatorSelection, ConsensusHook, DAYS, EXISTENTIAL_DEPOSIT, HOURS, Hash,
	MAXIMUM_BLOCK_WEIGHT, MICROUNIT, MessageQueue, MetaTxExtension, NORMAL_DISPATCH_RATIO, Nonce,
	OriginCaller, PalletInfo, ParachainSystem, Preimage, Runtime, RuntimeCall, RuntimeEvent,
	RuntimeFreezeReason, RuntimeHoldReason, RuntimeOrigin, RuntimeTask, SLOT_DURATION, Session,
	SessionKeys, Signature, System, Treasury, UNIT, VERSION, WeightToFee, XcmpQueue,
	weights::{BlockExecutionWeight, ExtrinsicBaseWeight, ParityDbWeight},
};
use xcm_config::{RelayLocation, XcmOriginToTransactDispatchOrigin};

parameter_types! {
	pub const Version: RuntimeVersion = VERSION;

	pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
		.base_block(BlockExecutionWeight::get())
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = ExtrinsicBaseWeight::get();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();
}

#[derive_impl(frame_system::config_preludes::ParaChainDefaultConfig)]
impl frame_system::Config for Runtime {
	type AccountId = AccountId;
	type Nonce = Nonce;
	type Hash = Hash;
	type Block = Block;
	type BlockHashCount = BlockHashCount;
	type Version = Version;
	type AccountData = pallet_balances::AccountData<Balance>;
	/// Must match the collator's actual `--database` backend — switch back to
	/// `RocksDbWeight` if collators run RocksDB.
	type DbWeight = ParityDbWeight;
	type BlockWeights = RuntimeBlockWeights;
	type BlockLength = RuntimeBlockLength;
	/// `allfeat_network`, registered in the SS58 registry.
	type SS58Prefix = ConstU16<440>;
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
	type MaxConsumers = ConstU32<16>;
}

impl cumulus_pallet_weight_reclaim::Config for Runtime {
	type WeightInfo = ();
}

impl pallet_timestamp::Config for Runtime {
	type Moment = u64;
	type OnTimestampSet = Aura;
	type MinimumPeriod = ConstU64<0>;
	type WeightInfo = ();
}

impl pallet_authorship::Config for Runtime {
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
	type EventHandler = (CollatorSelection,);
}

parameter_types! {
	pub const ExistentialDeposit: Balance = EXISTENTIAL_DEPOSIT;
}

impl pallet_balances::Config for Runtime {
	type MaxLocks = ConstU32<50>;
	type Balance = Balance;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = [u8; 8];
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type FreezeIdentifier = RuntimeFreezeReason;
	type MaxFreezes = VariantCountOf<RuntimeFreezeReason>;
	type DoneSlashHandler = ();
}

parameter_types! {
	/// Historical mainnet value (10x the Melodie byte fee).
	pub const TransactionByteFee: Balance = 10 * MICROUNIT;
}

/// Routes 80% of transaction fees and tips to the block author and 20% to
/// the treasury (historical mainnet split).
pub struct DealWithFees;
impl OnUnbalanced<Credit<AccountId, Balances>> for DealWithFees {
	fn on_unbalanceds(mut fees_then_tips: impl Iterator<Item = Credit<AccountId, Balances>>) {
		if let Some(mut amount) = fees_then_tips.next() {
			if let Some(tips) = fees_then_tips.next() {
				tips.merge_into(&mut amount);
			}
			let treasury_amount = Perbill::from_percent(20) * amount.peek();
			let (treasury_part, author_part) = amount.split(treasury_amount);
			let _ = Balances::resolve(&Treasury::account_id(), treasury_part);
			if let Some(author) = Authorship::author() {
				let _ = Balances::resolve(&author, author_part);
			}
		}
	}
}

impl pallet_transaction_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnChargeTransaction = pallet_transaction_payment::FungibleAdapter<Balances, DealWithFees>;
	type WeightToFee = WeightToFee;
	type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
	type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
	type OperationalFeeMultiplier = ConstU8<5>;
	type WeightInfo = ();
}

impl pallet_sudo::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type WeightInfo = ();
}

parameter_types! {
	pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
	pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
	pub const RelayOrigin: AggregateMessageOrigin = AggregateMessageOrigin::Parent;
}

impl cumulus_pallet_parachain_system::Config for Runtime {
	type WeightInfo = ();
	type RuntimeEvent = RuntimeEvent;
	type OnSystemEvent = ();
	type SelfParaId = parachain_info::Pallet<Runtime>;
	type OutboundXcmpMessageSource = XcmpQueue;
	type DmpQueue = frame_support::traits::EnqueueWithOrigin<MessageQueue, RelayOrigin>;
	type ReservedDmpWeight = ReservedDmpWeight;
	type XcmpMessageHandler = XcmpQueue;
	type ReservedXcmpWeight = ReservedXcmpWeight;
	type CheckAssociatedRelayNumber = RelayNumberMonotonicallyIncreases;
	type ConsensusHook = ConsensusHook;
	type RelayParentOffset = ConstU32<0>;
}

impl parachain_info::Config for Runtime {}

parameter_types! {
	pub MessageQueueServiceWeight: Weight = Perbill::from_percent(35) * RuntimeBlockWeights::get().max_block;
}

impl pallet_message_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	#[cfg(feature = "runtime-benchmarks")]
	type MessageProcessor =
		pallet_message_queue::mock_helpers::NoopMessageProcessor<AggregateMessageOrigin>;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type MessageProcessor = polkadot_sdk::staging_xcm_builder::ProcessXcmMessage<
		AggregateMessageOrigin,
		polkadot_sdk::staging_xcm_executor::XcmExecutor<xcm_config::XcmConfig>,
		RuntimeCall,
	>;
	type Size = u32;
	type QueueChangeHandler = NarrowOriginToSibling<XcmpQueue>;
	type QueuePausedQuery = NarrowOriginToSibling<XcmpQueue>;
	type HeapSize = ConstU32<{ 103 * 1024 }>;
	type MaxStale = ConstU32<8>;
	type ServiceWeight = MessageQueueServiceWeight;
	type IdleMaxServiceWeight = ();
}

impl cumulus_pallet_aura_ext::Config for Runtime {}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ChannelInfo = ParachainSystem;
	type VersionWrapper = ();
	type XcmpQueue = TransformOrigin<MessageQueue, AggregateMessageOrigin, ParaId, ParaIdToSibling>;
	type MaxInboundSuspended = ConstU32<1_000>;
	type ControllerOrigin = EnsureRoot<AccountId>;
	type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
	type WeightInfo = ();
	type PriceForSiblingDelivery = NoPriceForMessageDelivery<ParaId>;
	type MaxActiveOutboundChannels = ConstU32<128>;
	type MaxPageSize = ConstU32<{ 1 << 16 }>;
}

parameter_types! {
	pub const Period: u32 = 6 * HOURS;
	pub const Offset: u32 = 0;
}

impl pallet_session::Config for Runtime {
	type DisablingStrategy = ();
	type RuntimeEvent = RuntimeEvent;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
	type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
	type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
	type SessionManager = CollatorSelection;
	type SessionHandler = <SessionKeys as sp_runtime::traits::OpaqueKeys>::KeyTypeIdProviders;
	type Keys = SessionKeys;
	type WeightInfo = ();
	type Currency = Balances;
	type KeyDeposit = ();
}

impl pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
	type DisabledValidators = ();
	type MaxAuthorities = ConstU32<100_000>;
	type AllowMultipleBlocksPerSlot = ConstBool<true>;
	type SlotDuration = ConstU64<SLOT_DURATION>;
}

parameter_types! {
	pub const PotId: PalletId = PalletId(*b"PotStake");
	pub const SessionLength: BlockNumber = 6 * HOURS;
	pub const StakingAdminBodyId: BodyId = BodyId::Defense;
}

pub type CollatorSelectionUpdateOrigin = EitherOfDiverse<
	EnsureRoot<AccountId>,
	EnsureXcm<IsVoiceOfBody<RelayLocation, StakingAdminBodyId>>,
>;

impl pallet_collator_selection::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type UpdateOrigin = CollatorSelectionUpdateOrigin;
	type PotId = PotId;
	type MaxCandidates = ConstU32<100>;
	type MinEligibleCollators = ConstU32<4>;
	type MaxInvulnerables = ConstU32<20>;
	// Should be a multiple of the session period or things will get inconsistent.
	type KickThreshold = Period;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
	type ValidatorRegistration = Session;
	type WeightInfo = ();
}

impl pallet_utility::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = ();
}

parameter_types! {
	pub MaximumSchedulerWeight: Weight =
		Perbill::from_percent(80) * RuntimeBlockWeights::get().max_block;
}

impl pallet_scheduler::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type PalletsOrigin = OriginCaller;
	type RuntimeCall = RuntimeCall;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = EnsureRoot<AccountId>;
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
	type MaxScheduledPerBlock = ConstU32<50>;
	type WeightInfo = ();
	type BlockNumberProvider = frame_system::Pallet<Runtime>;
	type Preimages = Preimage;
}

parameter_types! {
	pub const PreimageBaseDeposit: Balance = deposit(2, 64);
	pub const PreimageByteDeposit: Balance = deposit(0, 1);
	pub const PreimageHoldReason: RuntimeHoldReason =
		RuntimeHoldReason::Preimage(pallet_preimage::HoldReason::Preimage);
}

impl pallet_preimage::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type Currency = Balances;
	type ManagerOrigin = EnsureRoot<AccountId>;
	type Consideration = HoldConsideration<
		AccountId,
		Balances,
		PreimageHoldReason,
		LinearStoragePrice<PreimageBaseDeposit, PreimageByteDeposit, Balance>,
	>;
}

parameter_types! {
	pub const ProxyDepositBase: Balance = deposit(1, 8);
	pub const ProxyDepositFactor: Balance = deposit(0, 33);
	pub const AnnouncementDepositBase: Balance = deposit(1, 8);
	pub const AnnouncementDepositFactor: Balance = deposit(0, 66);
	pub const MaxProxies: u16 = 32;
	pub const MaxPending: u16 = 32;
}

#[derive(
	Debug,
	Default,
	Copy,
	Clone,
	Eq,
	PartialEq,
	Ord,
	PartialOrd,
	Encode,
	Decode,
	DecodeWithMemTracking,
	MaxEncodedLen,
	scale_info::TypeInfo,
)]
pub enum ProxyType {
	#[default]
	Any,
	NonTransfer,
}
impl InstanceFilter<RuntimeCall> for ProxyType {
	fn filter(&self, c: &RuntimeCall) -> bool {
		match self {
			ProxyType::Any => true,
			ProxyType::NonTransfer => !matches!(c, RuntimeCall::Balances(..)),
		}
	}
	fn is_superset(&self, o: &Self) -> bool {
		match (self, o) {
			(x, y) if x == y => true,
			(ProxyType::Any, _) => true,
			(_, ProxyType::Any) => false,
			(ProxyType::NonTransfer, _) => true,
		}
	}
}

impl pallet_proxy::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type ProxyType = ProxyType;
	type ProxyDepositBase = ProxyDepositBase;
	type ProxyDepositFactor = ProxyDepositFactor;
	type MaxProxies = MaxProxies;
	type WeightInfo = ();
	type MaxPending = MaxPending;
	type CallHasher = BlakeTwo256;
	type AnnouncementDepositBase = AnnouncementDepositBase;
	type AnnouncementDepositFactor = AnnouncementDepositFactor;
	type BlockNumberProvider = frame_system::Pallet<Runtime>;
}

parameter_types! {
	pub const DepositBase: Balance = deposit(1, 88);
	pub const DepositFactor: Balance = deposit(0, 32);
	pub const MaxSignatories: u32 = 100;
}

impl pallet_multisig::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type DepositBase = DepositBase;
	type DepositFactor = DepositFactor;
	type MaxSignatories = MaxSignatories;
	type BlockNumberProvider = frame_system::Pallet<Runtime>;
	type WeightInfo = ();
}

impl pallet_meta_tx::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Extension = MetaTxExtension;
	type WeightInfo = ();
}

impl pallet_verify_signature::Config for Runtime {
	type Signature = Signature;
	type AccountIdentifier = <Signature as Verify>::Signer;
	type WeightInfo = ();
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}

// Treasury (fed by 20% of the transaction fees and the tokenomics
// envelopes whose unique beneficiary is the treasury account).

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
	fn create_asset_kind(_seed: u32) {}
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
	type WeightInfo = ();
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

// Token allocation (genesis tokenomics: upfront, cliff and linear vesting
// per envelope).

parameter_types! {
	pub const TokenAllocPalletId: PalletId = PalletId(*b"m/tknalc");
	pub const EpochDuration: BlockNumber = DAYS;
	pub const MaxPayoutsPerBlock: u32 = 256;
}

impl pallet_token_allocation::Config for Runtime {
	type Currency = Balances;
	type AdminOrigin = EnsureRoot<Self::AccountId>;
	type PalletId = TokenAllocPalletId;
	type EpochDuration = EpochDuration;
	type MaxPayoutsPerBlock = MaxPayoutsPerBlock;
	type RuntimeHoldReason = RuntimeHoldReason;
	type WeightInfo = pallet_token_allocation::weights::AllfeatWeight<Runtime>;
}

// ATS (Allfeat Timestamp Service).

parameter_types! {
	pub const AtsBaseDeposit: Balance = 5 * UNIT;
	pub const AtsVersionDeposit: Balance = UNIT;
	pub const MaxVersionsPerAts: u32 = 100;
	pub const MaxAtsPerAccount: u32 = 1000;
}

impl pallet_ats::Config for Runtime {
	type RuntimeHoldReason = RuntimeHoldReason;
	type Currency = Balances;
	type BaseDeposit = AtsBaseDeposit;
	type VersionDeposit = AtsVersionDeposit;
	type MaxVersionsPerAts = MaxVersionsPerAts;
	type MaxAtsPerAccount = MaxAtsPerAccount;
	type OffchainSignature = sp_runtime::MultiSignature;
	type Signer = sp_runtime::MultiSigner;
	type WeightInfo = crate::weights::ats::AllfeatWeight<Runtime>;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}
