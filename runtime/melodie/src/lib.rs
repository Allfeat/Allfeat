// This file is part of Allfeat.

// Copyright (C) 2022-2024 Allfeat.
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

//! The Melodie runtime.

#![cfg_attr(not(feature = "std"), no_std)]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

// allfeat
pub use allfeat_primitives::*;

// substrate use frame_support::pallet_prelude::*;
use sp_runtime::create_runtime_str;
use sp_std::prelude::*;

#[cfg(any(feature = "std", test))]
pub use frame_system::Call as SystemCall;
#[cfg(any(feature = "std", test))]
pub use pallet_balances::Call as BalancesCall;

/// Constant values used within the runtime.
pub mod constants;
pub use constants::time::*;

pub mod apis;
mod pallets;
pub use pallets::*;
mod genesis;
mod midds;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;

/// Runtime version.
#[sp_version::runtime_version]
pub const VERSION: sp_version::RuntimeVersion = sp_version::RuntimeVersion {
	spec_name: create_runtime_str!("allfeat-testnet"),
	impl_name: create_runtime_str!("allfeatlabs-melodie"),
	authoring_version: 1,
	// Per convention: if the runtime behavior changes, increment spec_version
	// and set impl_version to 0. If only runtime
	// implementation changes and behavior does not, then leave spec_version as
	// is and increment impl_version.
	spec_version: 100,
	impl_version: 1,
	apis: crate::apis::RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 1,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> sp_version::NativeVersion {
	sp_version::NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

/// Block header type as expected by this runtime.
pub type Header = sp_runtime::generic::Header<BlockNumber, Hashing>;
/// Block type as expected by this runtime.
pub type Block = sp_runtime::generic::Block<Header, UncheckedExtrinsic>;

//// The `TransactionExtension` to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckNonZeroSender<Runtime>,
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
	frame_metadata_hash_extension::CheckMetadataHash<Runtime>,
);

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
	sp_runtime::generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;

/// The payload being signed in transactions.
pub type SignedPayload = sp_runtime::generic::SignedPayload<RuntimeCall, SignedExtra>;

/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
	Migrations,
>;

/// All migrations of the runtime, aside from the ones declared in the pallets.
///
/// This can be a tuple of types, each implementing `OnRuntimeUpgrade`.
#[allow(unused_parens)]
type Migrations = ();

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
		RuntimeTask
	)]
	pub struct Runtime;

	#[runtime::pallet_index(0)]
	pub type System = frame_system;

	#[runtime::pallet_index(1)]
	pub type Balances = pallet_balances;

	#[runtime::pallet_index(2)]
	pub type Babe = pallet_babe;

	#[runtime::pallet_index(3)]
	pub type Timestamp = pallet_timestamp;

	#[runtime::pallet_index(26)]
	pub type TransactionPayment = pallet_transaction_payment;

	#[runtime::pallet_index(4)]
	pub type ImOnline = pallet_im_online;

	#[runtime::pallet_index(5)]
	pub type Authorship = pallet_authorship;

	#[runtime::pallet_index(201)]
	pub type Mmr = pallet_mmr;

	#[runtime::pallet_index(8)]
	pub type ValidatorSet = pallet_validator_set;

	#[runtime::pallet_index(27)]
	pub type Historical = pallet_session::historical;

	#[runtime::pallet_index(9)]
	pub type Session = pallet_session;

	#[runtime::pallet_index(10)]
	pub type Grandpa = pallet_grandpa;

	#[runtime::pallet_index(12)]
	pub type AuthorityDiscovery = pallet_authority_discovery;

	#[runtime::pallet_index(16)]
	pub type Utility = pallet_utility;

	#[runtime::pallet_index(17)]
	pub type Identity = pallet_identity;

	#[runtime::pallet_index(20)]
	pub type Scheduler = pallet_scheduler;

	#[runtime::pallet_index(28)]
	pub type Preimage = pallet_preimage;

	#[runtime::pallet_index(21)]
	pub type Sudo = pallet_sudo;

	#[runtime::pallet_index(22)]
	pub type Proxy = pallet_proxy;

	#[runtime::pallet_index(23)]
	pub type Multisig = pallet_multisig;

	// Allfeat related
	//	#[runtime::pallet_index(100)] Old artists pallet

	#[runtime::pallet_index(101)]
	pub type Stakeholders = pallet_midds<Instance1>;

	#[runtime::pallet_index(102)]
	pub type MusicalWorks = pallet_midds<Instance2>;
}
