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

use cumulus_pallet_parachain_system::RelayNumberMonotonicallyIncreases;
use polkadot_sdk::{
	cumulus_pallet_aura_ext, cumulus_pallet_weight_reclaim, cumulus_pallet_xcmp_queue,
	cumulus_primitives_core::{AggregateMessageOrigin, ParaId},
	frame_support,
	frame_support::{
		derive_impl,
		dispatch::DispatchClass,
		pallet_prelude::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen},
		parameter_types,
		traits::{
			fungible::{Balanced, Credit, HoldConsideration},
			ConstBool, ConstU32, ConstU64, ConstU8, Contains, EitherOfDiverse,
			EqualPrivilegeOnly, Imbalance, InstanceFilter, LinearStoragePrice, OnUnbalanced,
			TransformOrigin, VariantCountOf,
		},
		weights::{ConstantMultiplier, Weight},
		PalletId,
	},
	frame_system,
	frame_system::{
		limits::{BlockLength, BlockWeights},
		EnsureRoot, EnsureRootWithSuccess, EnsureSigned,
	},
	pallet_aura, pallet_authorship, pallet_balances, pallet_collator_selection,
	pallet_message_queue, pallet_meta_tx, pallet_multisig, pallet_preimage, pallet_proxy,
	pallet_safe_mode, pallet_scheduler, pallet_session, pallet_sudo, pallet_timestamp,
	pallet_transaction_payment, pallet_utility, pallet_verify_signature,
	pallet_xcm::{EnsureXcm, IsVoiceOfBody},
	parachains_common::message_queue::{NarrowOriginToSibling, ParaIdToSibling},
	polkadot_runtime_common::{xcm_sender::NoPriceForMessageDelivery, BlockHashCount},
	sp_consensus_aura::sr25519::AuthorityId as AuraId,
	sp_runtime,
	sp_runtime::{
		traits::{AccountIdConversion, BlakeTwo256, Bounded, Verify},
		FixedPointNumber, FixedU128, Perbill, Perquintill,
	},
	sp_version::RuntimeVersion,
	staging_parachain_info as parachain_info,
	staging_xcm::latest::prelude::BodyId,
};
use polkadot_sdk::pallet_transaction_payment::{Multiplier, TargetedFeeAdjustment};

use super::{
	weights::{BlockExecutionWeight, ExtrinsicBaseWeight, ParityDbWeight},
	AccountId, Authorship, Aura, Balance, Balances, Block, BlockNumber, CollatorSelection,
	ConsensusHook, Hash, MessageQueue, MetaTxExtension, Nonce, OriginCaller, PalletInfo,
	ParachainSystem, Preimage, Runtime, RuntimeCall, RuntimeEvent, RuntimeFreezeReason,
	RuntimeHoldReason, RuntimeOrigin, RuntimeTask, Session, SessionKeys, Signature, System,
	WeightToFee, XcmpQueue, AVERAGE_ON_INITIALIZE_RATIO, DAYS, EXISTENTIAL_DEPOSIT, HOURS,
	MAXIMUM_BLOCK_WEIGHT, MICROUNIT, MILLIUNIT, NORMAL_DISPATCH_RATIO, SLOT_DURATION, UNIT,
	VERSION,
};
use xcm_config::{RelayLocation, XcmOriginToTransactDispatchOrigin};

parameter_types! {
	pub const Version: RuntimeVersion = VERSION;

	pub RuntimeBlockLength: BlockLength = BlockLength::builder()
		.max_length(5 * 1024 * 1024)
		.modify_max_length_for_class(DispatchClass::Normal, |m| *m = NORMAL_DISPATCH_RATIO * *m)
		.build();
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
	pub const SS58Prefix: u16 = 42;
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
	type SS58Prefix = SS58Prefix;
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
	/// Live-chain parity: deliberately 1/10 of the template default so the
	/// length fee does not dwarf the MIDDS bonds.
	pub const TransactionByteFee: Balance = MICROUNIT;
}

parameter_types! {
	pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
	pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(75, 1_000_000);
	pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(5, 10u128);
	pub MaximumMultiplier: Multiplier = Bounded::max_value();
}

/// `polkadot_runtime_common::SlowAdjustingFeeUpdate` with a 0.5 multiplier
/// floor (live-chain parity).
pub type SlowAdjustingFeeUpdate<R> = TargetedFeeAdjustment<
	R,
	TargetBlockFullness,
	AdjustmentVariable,
	MinimumMultiplier,
	MaximumMultiplier,
>;

/// Routes 100% of transaction fees and tips to the block author.
pub struct DealWithFees;
impl OnUnbalanced<Credit<AccountId, Balances>> for DealWithFees {
	fn on_unbalanceds(mut fees_then_tips: impl Iterator<Item = Credit<AccountId, Balances>>) {
		if let Some(mut amount) = fees_then_tips.next() {
			if let Some(tips) = fees_then_tips.next() {
				tips.merge_into(&mut amount);
			}
			if let Some(author) = Authorship::author() {
				match Balances::resolve(&author, amount) {
					Ok(_) => (),
					Err(_drop) => (),
				}
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

pub const fn deposit(items: u32, bytes: u32) -> Balance {
	items as Balance * 10 * UNIT + (bytes as Balance) * 100 * MICROUNIT
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
	Any,
	NonTransfer,
}
impl Default for ProxyType {
	fn default() -> Self {
		Self::Any
	}
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

pub struct SafeModeWhitelistedCalls;
impl Contains<RuntimeCall> for SafeModeWhitelistedCalls {
	fn contains(call: &RuntimeCall) -> bool {
		matches!(call, RuntimeCall::System(_) | RuntimeCall::SafeMode(_))
	}
}

parameter_types! {
	pub const EnterDuration: BlockNumber = 4 * HOURS;
	pub const EnterDepositAmount: Option<Balance> = None;
	pub const ExtendDuration: BlockNumber = 2 * HOURS;
	pub const ExtendDepositAmount: Option<Balance> = None;
	pub const ReleaseDelay: u32 = 2 * DAYS;
}

impl pallet_safe_mode::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type RuntimeHoldReason = RuntimeHoldReason;
	type WhitelistedCalls = SafeModeWhitelistedCalls;
	type EnterDuration = EnterDuration;
	type ExtendDuration = ExtendDuration;
	type EnterDepositAmount = EnterDepositAmount;
	type ExtendDepositAmount = ExtendDepositAmount;
	type ForceEnterOrigin = EnsureRootWithSuccess<AccountId, ConstU32<9>>;
	type ForceExtendOrigin = EnsureRootWithSuccess<AccountId, ConstU32<11>>;
	type ForceExitOrigin = EnsureRoot<AccountId>;
	type ForceDepositOrigin = EnsureRoot<AccountId>;
	type Notify = ();
	type ReleaseDelay = ReleaseDelay;
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

// MIDDS — one `pallet_midds` instance per type: MusicalWork (Instance1),
// Recording (Instance2), Release (Instance3). The bond floor below is
// runtime-mutable storage seeded at genesis (and re-seeded by the migration);
// governance can recalibrate via `force_set_deposit_*`.

parameter_types! {
	pub const MiddsDepositBase: Balance = 100 * MILLIUNIT;
	pub const MiddsDepositPerByte: Balance = 250 * MICROUNIT;

	pub const MiddsCommitmentWindow: BlockNumber = 7 * DAYS;
	pub const MiddsMaxFinalizationsPerBlock: u32 = 100;
	// Must stay ≥ the benchmark sweep range (`Linear<1, 64>`).
	pub const MiddsMaxRemovalsPerCall: u32 = 100;
	pub const MiddsBlocksPerDay: BlockNumber = DAYS;

	// M_fast — anti-DoS, per-block reactivity.
	pub const MiddsFastTargetPerBlock: u32 = 100;
	pub MiddsFastAdjustmentRate: FixedU128 = FixedU128::from_rational(125, 1_000);
	pub MiddsFastMultiplierMin: FixedU128 = FixedU128::from_rational(1, 10);
	pub MiddsFastMultiplierMax: FixedU128 = FixedU128::from_u32(20);

	// M_slow — anti-flood, 7-day rolling window.
	pub const MiddsSlowTargetPerWindow: u32 = 200_000;
	pub MiddsSlowAdjustmentRate: FixedU128 = FixedU128::from_rational(5, 100);
	pub MiddsSlowMultiplierMin: FixedU128 = FixedU128::from_rational(1, 10);
	pub MiddsSlowMultiplierMax: FixedU128 = FixedU128::from_u32(50);

	pub const MiddsTreasuryPalletId: PalletId = PalletId(*b"af/midds");
	pub MiddsTreasuryAccount: AccountId =
		MiddsTreasuryPalletId::get().into_account_truncating();
}

impl pallet_midds::Config<pallet_midds::Instance1> for Runtime {
	type Currency = Balances;
	type RuntimeHoldReason = RuntimeHoldReason;
	type Midds = midds_types::MusicalWork;
	type ProviderOrigin = EnsureSigned<AccountId>;
	type ForceOrigin = EnsureRoot<AccountId>;
	type OffchainSignature = Signature;
	type Signer = sp_runtime::MultiSigner;
	type TreasuryAccount = MiddsTreasuryAccount;
	type CommitmentWindow = MiddsCommitmentWindow;
	type MaxFinalizationsPerBlock = MiddsMaxFinalizationsPerBlock;
	type MaxRemovalsPerCall = MiddsMaxRemovalsPerCall;
	type BlocksPerDay = MiddsBlocksPerDay;
	type FastTargetPerBlock = MiddsFastTargetPerBlock;
	type FastAdjustmentRate = MiddsFastAdjustmentRate;
	type FastMultiplierMin = MiddsFastMultiplierMin;
	type FastMultiplierMax = MiddsFastMultiplierMax;
	type SlowTargetPerWindow = MiddsSlowTargetPerWindow;
	type SlowAdjustmentRate = MiddsSlowAdjustmentRate;
	type SlowMultiplierMin = MiddsSlowMultiplierMin;
	type SlowMultiplierMax = MiddsSlowMultiplierMax;
	type WeightInfo = crate::weights::midds_musical_works::AllfeatWeight<Runtime>;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = MusicalWorksBenchmarkHelper;
}

#[cfg(feature = "runtime-benchmarks")]
pub struct MusicalWorksBenchmarkHelper;

#[cfg(feature = "runtime-benchmarks")]
impl pallet_midds::BenchmarkHelper<midds_types::MusicalWork, Signature, AccountId>
	for MusicalWorksBenchmarkHelper
{
	fn bench_instance(size: u32) -> midds_types::MusicalWork {
		use frame_support::BoundedVec;
		use midds_types::{Creator, CreatorRole, CreatorRoles, MusicalWorkV1, PartyId, WorkType};

		// The identifier stays constant across calls (`IdentifierImmutable`
		// guard); per-iteration uniqueness is carried by the title length.
		let title_len = ((size as usize) + 1).min(midds_types::TITLE_MAX_LEN as usize);
		let title = BoundedVec::try_from(alloc::vec![b'a'; title_len])
			.expect("title clamped to TITLE_MAX_LEN");
		let iswc = bench_iswc();
		let ipi = BoundedVec::try_from(b"123456789".to_vec()).expect("9-byte IPI literal");
		let mut roles = CreatorRoles::new();
		roles
			.try_insert(CreatorRole::Composer)
			.expect("single role fits CREATOR_ROLES_MAX");
		let creators = BoundedVec::try_from(alloc::vec![Creator {
			roles,
			party: PartyId::Ipi(ipi),
		}])
		.expect("single creator fits CREATORS_MAX");

		midds_types::MusicalWork::V1(MusicalWorkV1 {
			iswc,
			title,
			creation_year: Some(2025),
			instrumental: false,
			language: None,
			explicit_lyrics: false,
			bpm: None,
			key: None,
			work_type: WorkType::Original,
			samples: Default::default(),
			creators,
			classical_info: None,
			offchain_extension: None,
		})
	}

	fn create_signature(entropy: &[u8], msg: &[u8]) -> (Signature, AccountId) {
		bench_create_signature(entropy, msg)
	}
}

/// Constant, structurally-valid ISWC literal.
#[cfg(feature = "runtime-benchmarks")]
fn bench_iswc() -> midds_traits::Iswc {
	use frame_support::BoundedVec;
	BoundedVec::try_from(b"T0000000001".to_vec()).expect("11-byte literal fits ISWC bound")
}

/// Deterministic `(MultiSignature, AccountId)` pair valid for `msg`. Uses the
/// benchmark keystore (`sp_io::crypto`) so the runtime build stays
/// `no_std`-clean (no `sp-core/full_crypto`).
#[cfg(feature = "runtime-benchmarks")]
fn bench_create_signature(entropy: &[u8], msg: &[u8]) -> (Signature, AccountId) {
	use sp_runtime::traits::IdentifyAccount as _;
	let path = core::str::from_utf8(entropy).unwrap_or("bench");
	let uri = alloc::format!("//{path}");
	let public =
		polkadot_sdk::sp_io::crypto::sr25519_generate(0.into(), Some(uri.into_bytes()));
	let account: AccountId = sp_runtime::MultiSigner::Sr25519(public).into_account();
	let sig = polkadot_sdk::sp_io::crypto::sr25519_sign(0.into(), &public, msg)
		.expect("keystore available in benchmark context; qed");
	(Signature::Sr25519(sig), account)
}

// Instance2 — Recording (ISRC-keyed). Same calibration as Instance1.
impl pallet_midds::Config<pallet_midds::Instance2> for Runtime {
	type Currency = Balances;
	type RuntimeHoldReason = RuntimeHoldReason;
	type Midds = midds_types::Recording;
	type ProviderOrigin = EnsureSigned<AccountId>;
	type ForceOrigin = EnsureRoot<AccountId>;
	type OffchainSignature = Signature;
	type Signer = sp_runtime::MultiSigner;
	type TreasuryAccount = MiddsTreasuryAccount;
	type CommitmentWindow = MiddsCommitmentWindow;
	type MaxFinalizationsPerBlock = MiddsMaxFinalizationsPerBlock;
	type MaxRemovalsPerCall = MiddsMaxRemovalsPerCall;
	type BlocksPerDay = MiddsBlocksPerDay;
	type FastTargetPerBlock = MiddsFastTargetPerBlock;
	type FastAdjustmentRate = MiddsFastAdjustmentRate;
	type FastMultiplierMin = MiddsFastMultiplierMin;
	type FastMultiplierMax = MiddsFastMultiplierMax;
	type SlowTargetPerWindow = MiddsSlowTargetPerWindow;
	type SlowAdjustmentRate = MiddsSlowAdjustmentRate;
	type SlowMultiplierMin = MiddsSlowMultiplierMin;
	type SlowMultiplierMax = MiddsSlowMultiplierMax;
	type WeightInfo = crate::weights::midds_recordings::AllfeatWeight<Runtime>;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = RecordingsBenchmarkHelper;
}

#[cfg(feature = "runtime-benchmarks")]
pub struct RecordingsBenchmarkHelper;

#[cfg(feature = "runtime-benchmarks")]
impl pallet_midds::BenchmarkHelper<midds_types::Recording, Signature, AccountId>
	for RecordingsBenchmarkHelper
{
	fn bench_instance(size: u32) -> midds_types::Recording {
		use frame_support::BoundedVec;
		use midds_types::{PartyId, RecordingV1, WorkRef};

		let title_len = ((size as usize) + 1).min(midds_types::TITLE_MAX_LEN as usize);
		let title = BoundedVec::try_from(alloc::vec![b'a'; title_len])
			.expect("title clamped to TITLE_MAX_LEN");
		let isrc = bench_isrc();
		let ipi = BoundedVec::try_from(b"123456789".to_vec()).expect("9-byte IPI literal");

		midds_types::Recording::V1(RecordingV1 {
			isrc,
			title,
			title_aliases: Default::default(),
			artist: PartyId::Ipi(ipi),
			featuring: Default::default(),
			work: WorkRef::Midds(0),
			genre: None,
			sub_genre: None,
			record_year: None,
			version_type: None,
			performers: Default::default(),
			producers: Default::default(),
			duration: None,
			bpm: None,
			key: None,
			places: None,
			contributors: Default::default(),
			offchain_extension: None,
		})
	}

	fn create_signature(entropy: &[u8], msg: &[u8]) -> (Signature, AccountId) {
		bench_create_signature(entropy, msg)
	}
}

/// Constant, structurally-valid ISRC literal.
#[cfg(feature = "runtime-benchmarks")]
fn bench_isrc() -> midds_traits::Isrc {
	use frame_support::BoundedVec;
	BoundedVec::try_from(b"USAAA2500001".to_vec()).expect("12-byte literal fits ISRC bound")
}

// Instance3 — Release (UPC/EAN-keyed). Same calibration as Instance1/2.
impl pallet_midds::Config<pallet_midds::Instance3> for Runtime {
	type Currency = Balances;
	type RuntimeHoldReason = RuntimeHoldReason;
	type Midds = midds_types::Release;
	type ProviderOrigin = EnsureSigned<AccountId>;
	type ForceOrigin = EnsureRoot<AccountId>;
	type OffchainSignature = Signature;
	type Signer = sp_runtime::MultiSigner;
	type TreasuryAccount = MiddsTreasuryAccount;
	type CommitmentWindow = MiddsCommitmentWindow;
	type MaxFinalizationsPerBlock = MiddsMaxFinalizationsPerBlock;
	type MaxRemovalsPerCall = MiddsMaxRemovalsPerCall;
	type BlocksPerDay = MiddsBlocksPerDay;
	type FastTargetPerBlock = MiddsFastTargetPerBlock;
	type FastAdjustmentRate = MiddsFastAdjustmentRate;
	type FastMultiplierMin = MiddsFastMultiplierMin;
	type FastMultiplierMax = MiddsFastMultiplierMax;
	type SlowTargetPerWindow = MiddsSlowTargetPerWindow;
	type SlowAdjustmentRate = MiddsSlowAdjustmentRate;
	type SlowMultiplierMin = MiddsSlowMultiplierMin;
	type SlowMultiplierMax = MiddsSlowMultiplierMax;
	type WeightInfo = crate::weights::midds_releases::AllfeatWeight<Runtime>;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ReleasesBenchmarkHelper;
}

#[cfg(feature = "runtime-benchmarks")]
pub struct ReleasesBenchmarkHelper;

#[cfg(feature = "runtime-benchmarks")]
impl pallet_midds::BenchmarkHelper<midds_types::Release, Signature, AccountId>
	for ReleasesBenchmarkHelper
{
	fn bench_instance(size: u32) -> midds_types::Release {
		use frame_support::BoundedVec;
		use midds_types::{
			Country, PartyId, RecordingRef, ReleaseDate, ReleaseFormat, ReleasePackaging,
			ReleaseStatus, ReleaseType, ReleaseV1, Track,
		};

		let title_len = ((size as usize) + 1).min(midds_types::TITLE_MAX_LEN as usize);
		let title = BoundedVec::try_from(alloc::vec![b'a'; title_len])
			.expect("title clamped to TITLE_MAX_LEN");
		let upc = bench_upc();
		let ipi = BoundedVec::try_from(b"123456789".to_vec()).expect("9-byte IPI literal");
		let tracks = BoundedVec::try_from(alloc::vec![Track {
			number: 1,
			recording: RecordingRef::Midds(0),
		}])
		.expect("single track fits TRACKS_MAX");

		midds_types::Release::V1(ReleaseV1 {
			upc,
			title,
			title_aliases: Default::default(),
			artist: PartyId::Ipi(ipi),
			featuring: Default::default(),
			tracks,
			producers: Default::default(),
			status: ReleaseStatus::Official,
			release_date: ReleaseDate {
				year: 2024,
				month: 1,
				day: 1,
			},
			country: Country::Fr,
			distributor_name: BoundedVec::try_from(b"Believe".to_vec())
				.expect("non-empty distributor name"),
			release_type: ReleaseType::Album,
			format: ReleaseFormat::Cd,
			packaging: ReleasePackaging::None,
			cover_contributors: Default::default(),
			offchain_extension: None,
		})
	}

	fn create_signature(entropy: &[u8], msg: &[u8]) -> (Signature, AccountId) {
		bench_create_signature(entropy, msg)
	}
}

/// Constant, structurally-valid EAN-13 literal.
#[cfg(feature = "runtime-benchmarks")]
fn bench_upc() -> midds_traits::Upc {
	use frame_support::BoundedVec;
	BoundedVec::try_from(b"0000000000001".to_vec()).expect("13-byte literal fits UPC bound")
}
