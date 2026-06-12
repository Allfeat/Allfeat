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

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

pub mod apis;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod configs;
mod genesis_config_presets;
pub mod migrations;
mod weights;

extern crate alloc;

use alloc::vec::Vec;
#[cfg(feature = "std")]
use polkadot_sdk::sp_version::NativeVersion;
use polkadot_sdk::{
	cumulus_pallet_aura_ext, cumulus_pallet_weight_reclaim,
	cumulus_pallet_weight_reclaim::StorageWeightReclaim,
	cumulus_pallet_xcm, cumulus_pallet_xcmp_queue, frame_executive, frame_metadata_hash_extension,
	frame_support::{
		self,
		weights::{WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial},
	},
	frame_system, pallet_aura, pallet_authorship, pallet_balances, pallet_collator_selection,
	pallet_message_queue, pallet_meta_tx, pallet_multisig, pallet_preimage, pallet_proxy,
	pallet_safe_mode, pallet_scheduler, pallet_session, pallet_sudo, pallet_timestamp,
	pallet_transaction_payment, pallet_utility, pallet_verify_signature, pallet_xcm,
	sp_runtime::{Cow, Perbill, generic, impl_opaque_keys},
	sp_version::{self, RuntimeVersion},
	staging_parachain_info as parachain_info,
};
use smallvec::smallvec;

#[cfg(any(feature = "std", test))]
pub use polkadot_sdk::sp_runtime::BuildStorage;

pub use allfeat_runtime_common::{
	AVERAGE_ON_INITIALIZE_RATIO, AccountId, Address, BLOCK_PROCESSING_VELOCITY, Balance,
	BlockNumber, Hash, Header, MAXIMUM_BLOCK_WEIGHT, NORMAL_DISPATCH_RATIO, Nonce,
	RELAY_CHAIN_SLOT_DURATION_MILLIS, Signature, UNINCLUDED_SEGMENT_CAPACITY,
	currency::{CENTIUNIT, EXISTENTIAL_DEPOSIT, MICROUNIT, MILLIUNIT, UNIT},
	opaque,
	time::{DAYS, HOURS, MILLISECS_PER_BLOCK, MINUTES, SLOT_DURATION},
};

use weights::ExtrinsicBaseWeight;

pub type Block = generic::Block<Header, UncheckedExtrinsic>;

pub type SignedBlock = generic::SignedBlock<Block>;

pub type BlockId = generic::BlockId<Block>;

#[docify::export(template_signed_extra)]
pub type TxExtension = StorageWeightReclaim<
	Runtime,
	(
		frame_system::CheckNonZeroSender<Runtime>,
		frame_system::CheckSpecVersion<Runtime>,
		frame_system::CheckTxVersion<Runtime>,
		frame_system::CheckGenesis<Runtime>,
		frame_system::CheckEra<Runtime>,
		frame_system::CheckNonce<Runtime>,
		frame_system::CheckWeight<Runtime>,
		pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
		frame_metadata_hash_extension::CheckMetadataHash<Runtime>,
	),
>;

pub type UncheckedExtrinsic =
	generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, TxExtension>;

/// Bare meta-transaction extension: the signed part of a meta-transaction,
/// independent of the runtime's outer `TxExtension`.
pub type MetaTxBareExtension = (
	pallet_meta_tx::MetaTxMarker<Runtime>,
	frame_system::CheckNonZeroSender<Runtime>,
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckMortality<Runtime>,
	frame_system::CheckNonce<Runtime>,
);

#[cfg(feature = "runtime-benchmarks")]
pub type MetaTxExtension = pallet_meta_tx::WeightlessExtension<Runtime>;

/// `VerifySignature` validates the meta-tx signature here; it is intentionally
/// absent from the outer `TxExtension`.
#[cfg(not(feature = "runtime-benchmarks"))]
pub type MetaTxExtension = (pallet_verify_signature::VerifySignature<Runtime>, MetaTxBareExtension);

pub type Migrations = migrations::Migrations;

pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
	Migrations,
>;

pub struct WeightToFee;
impl WeightToFeePolynomial for WeightToFee {
	type Balance = Balance;
	fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
		// Fee parity with the live solo chain: one base-extrinsic worth of
		// weight maps to 10 MILLIUNIT.
		let p = 10 * MILLIUNIT;
		let q = Balance::from(ExtrinsicBaseWeight::get().ref_time());
		smallvec![WeightToFeeCoefficient {
			degree: 1,
			negative: false,
			coeff_frac: Perbill::from_rational(p % q, q),
			coeff_integer: p / q,
		}]
	}
}

impl_opaque_keys! {
	pub struct SessionKeys {
		pub aura: Aura,
	}
}

// This runtime CONTINUES the live `allfeat-melodie-3` solo chain in place
// (solo→para handover): `spec_name` MUST stay `allfeat-melodie-3`.
// `spec_version` 301 opens the parachain-era band (205..=299 stay free for
// last solo-side releases; 300 was tagged pre-cutover without the benched
// weights and never deployed — do not reuse it); `transaction_version` bumps
// to 4 (call surface changed, `TxExtension` gained `StorageWeightReclaim`).
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: Cow::Borrowed("allfeat-melodie-3"),
	impl_name: Cow::Borrowed("allfeatlabs-melodie-3"),
	authoring_version: 1,
	spec_version: 301,
	impl_version: 0,
	apis: apis::RUNTIME_API_VERSIONS,
	transaction_version: 4,
	system_version: 1,
};

/// Para ID the chain runs under on the relay: the ID reserved on Paseo.
/// Consumed by the genesis presets and seeded on the continued chain by
/// `TransitionToParachain`, so it has to match the registered ID.
pub const PARA_ID: u32 = 5206;

type ConsensusHook = cumulus_pallet_aura_ext::FixedVelocityConsensusHook<
	Runtime,
	RELAY_CHAIN_SLOT_DURATION_MILLIS,
	BLOCK_PROCESSING_VELOCITY,
	UNINCLUDED_SEGMENT_CAPACITY,
>;

#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

#[frame_support::runtime]
mod runtime {
	#[runtime::runtime]
	#[runtime::derive(
		RuntimeCall,
		RuntimeEvent,
		RuntimeError,
		RuntimeOrigin,
		RuntimeFreezeReason,
		RuntimeHoldReason,
		RuntimeSlashReason,
		RuntimeLockId,
		RuntimeTask,
		RuntimeViewFunction
	)]
	pub struct Runtime;

	// Pallet indices are pinned to the Melodie solo chain so its state decodes
	// unchanged (RuntimeCall/Origin/HoldReason discriminants). Declaration
	// ORDER (not index) drives hook execution and keeps the template's order.
	// 7/9/13 reuse slots freed by Validators/Grandpa/Historical; 105-108 are
	// the ATS/MIDDS indices.
	#[runtime::pallet_index(0)]
	pub type System = frame_system::Pallet<Runtime>;
	#[runtime::pallet_index(11)]
	pub type ParachainSystem = cumulus_pallet_parachain_system::Pallet<Runtime>;
	#[runtime::pallet_index(3)]
	pub type Timestamp = pallet_timestamp::Pallet<Runtime>;
	#[runtime::pallet_index(12)]
	pub type ParachainInfo = parachain_info::Pallet<Runtime>;
	#[runtime::pallet_index(13)]
	pub type WeightReclaim = cumulus_pallet_weight_reclaim::Pallet<Runtime>;

	#[runtime::pallet_index(5)]
	pub type Balances = pallet_balances::Pallet<Runtime>;
	#[runtime::pallet_index(6)]
	pub type TransactionPayment = pallet_transaction_payment::Pallet<Runtime>;

	#[runtime::pallet_index(10)]
	pub type Sudo = pallet_sudo::Pallet<Runtime>;

	#[runtime::pallet_index(4)]
	pub type Authorship = pallet_authorship::Pallet<Runtime>;
	// The order of these 4 is important and shall not change.
	#[runtime::pallet_index(7)]
	pub type CollatorSelection = pallet_collator_selection::Pallet<Runtime>;
	#[runtime::pallet_index(8)]
	pub type Session = pallet_session::Pallet<Runtime>;
	#[runtime::pallet_index(2)]
	pub type Aura = pallet_aura::Pallet<Runtime>;
	#[runtime::pallet_index(9)]
	pub type AuraExt = cumulus_pallet_aura_ext::Pallet<Runtime>;

	#[runtime::pallet_index(30)]
	pub type XcmpQueue = cumulus_pallet_xcmp_queue::Pallet<Runtime>;
	#[runtime::pallet_index(31)]
	pub type PolkadotXcm = pallet_xcm::Pallet<Runtime>;
	#[runtime::pallet_index(32)]
	pub type CumulusXcm = cumulus_pallet_xcm::Pallet<Runtime>;
	#[runtime::pallet_index(33)]
	pub type MessageQueue = pallet_message_queue::Pallet<Runtime>;

	#[runtime::pallet_index(1)]
	pub type Utility = pallet_utility::Pallet<Runtime>;
	#[runtime::pallet_index(17)]
	pub type Multisig = pallet_multisig::Pallet<Runtime>;
	#[runtime::pallet_index(16)]
	pub type Proxy = pallet_proxy::Pallet<Runtime>;
	#[runtime::pallet_index(14)]
	pub type Scheduler = pallet_scheduler::Pallet<Runtime>;
	#[runtime::pallet_index(15)]
	pub type Preimage = pallet_preimage::Pallet<Runtime>;
	#[runtime::pallet_index(18)]
	pub type SafeMode = pallet_safe_mode::Pallet<Runtime>;
	#[runtime::pallet_index(20)]
	pub type MetaTx = pallet_meta_tx::Pallet<Runtime>;
	#[runtime::pallet_index(21)]
	pub type VerifySignature = pallet_verify_signature::Pallet<Runtime>;

	#[runtime::pallet_index(105)]
	pub type Ats = pallet_ats::Pallet<Runtime>;

	#[runtime::pallet_index(106)]
	pub type MusicalWorks = pallet_midds::Pallet<Runtime, Instance1>;
	#[runtime::pallet_index(107)]
	pub type Recordings = pallet_midds::Pallet<Runtime, Instance2>;
	#[runtime::pallet_index(108)]
	pub type Releases = pallet_midds::Pallet<Runtime, Instance3>;
}

cumulus_pallet_parachain_system::register_validate_block! {
	Runtime = Runtime,
	BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
}
