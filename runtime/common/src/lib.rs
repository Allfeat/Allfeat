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

//! Common code shared by the Allfeat parachain runtimes (Melodie testnet and
//! Allfeat mainnet): primitive types, monetary/time constants and the fee
//! machinery. Anything consensus-critical that the two networks are NOT
//! meant to share by design (existential deposit aside, e.g. byte fee, fee
//! routing, SS58 prefix, pallet sets) stays in each runtime.

#![cfg_attr(not(feature = "std"), no_std)]

pub mod currency;
pub mod time;

use polkadot_sdk::{
	cumulus_primitives_core,
	frame_support::{
		dispatch::DispatchClass,
		parameter_types,
		weights::{Weight, constants::WEIGHT_REF_TIME_PER_SECOND},
	},
	frame_system::limits::BlockLength,
	pallet_transaction_payment::{Multiplier, TargetedFeeAdjustment},
	sp_core,
	sp_runtime::{
		FixedPointNumber, MultiAddress, MultiSignature, Perbill, Perquintill, generic,
		traits::{BlakeTwo256, Bounded, IdentifyAccount, Verify},
	},
};

/// Alias to 512-bit hash when used in the context of a transaction signature
/// on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it
/// equivalent to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Nonce = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// An index to a block.
pub type BlockNumber = u32;

/// The address format for describing accounts.
pub type Address = MultiAddress<AccountId, ()>;

/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

/// Opaque types. These are used by the CLI to instantiate machinery that
/// don't need to know the specifics of the runtime. They can then be made to
/// be agnostic over specific formats of data like extrinsics, allowing for
/// them to continue syncing the network through upgrades to even the core
/// data structures.
pub mod opaque {
	use super::BlockNumber;
	use polkadot_sdk::sp_runtime::{
		generic,
		traits::{BlakeTwo256, Hash as HashT},
	};

	pub use polkadot_sdk::sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	pub type BlockId = generic::BlockId<Block>;
	pub type Hash = <BlakeTwo256 as HashT>::Output;
}

/// We assume that ~5% of the block weight is consumed by `on_initialize`
/// handlers. This is used to limit the maximal weight of a single extrinsic.
pub const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(5);

/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can
/// be used by `Operational` extrinsics.
pub const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

/// We allow for 2 seconds of compute with a 6 second average block time,
/// bounded by the relay PoV size limit.
pub const MAXIMUM_BLOCK_WEIGHT: Weight = Weight::from_parts(
	WEIGHT_REF_TIME_PER_SECOND.saturating_mul(2),
	cumulus_primitives_core::relay_chain::MAX_POV_SIZE as u64,
);

/// Maximum number of blocks simultaneously accepted by the runtime, not yet
/// included into the relay chain.
pub const UNINCLUDED_SEGMENT_CAPACITY: u32 = 3;

/// How many parachain blocks are processed by the relay chain per parent.
/// Limits the number of blocks authored per slot.
pub const BLOCK_PROCESSING_VELOCITY: u32 = 1;

/// Relay chain slot duration, in milliseconds.
pub const RELAY_CHAIN_SLOT_DURATION_MILLIS: u32 = 6000;

parameter_types! {
	/// Maximum length of a block, 5 MiB.
	pub RuntimeBlockLength: BlockLength = BlockLength::builder()
		.max_length(5 * 1024 * 1024)
		.modify_max_length_for_class(DispatchClass::Normal, |m| *m = NORMAL_DISPATCH_RATIO * *m)
		.build();
}

parameter_types! {
	/// The portion of `NORMAL_DISPATCH_RATIO` that we adjust the fees with.
	/// Blocks filled less than this will decrease the weight and more will
	/// increase.
	pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
	/// The adjustment variable of the runtime. Higher values will cause
	/// `TargetBlockFullness` to change the fees more rapidly.
	pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(75, 1_000_000);
	/// Minimum amount of the multiplier (0.5, live-chain parity). This value
	/// cannot be too low: combined with `AdjustmentVariable` it must allow
	/// recovering from the minimum.
	pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(5, 10u128);
	/// The maximum amount of the multiplier.
	pub MaximumMultiplier: Multiplier = Bounded::max_value();
}

/// Parameterized slow adjusting fee updated based on
/// <https://research.web3.foundation/Polkadot/overview/token-economics#2-slow-adjusting-mechanism>
pub type SlowAdjustingFeeUpdate<R> = TargetedFeeAdjustment<
	R,
	TargetBlockFullness,
	AdjustmentVariable,
	MinimumMultiplier,
	MaximumMultiplier,
>;
