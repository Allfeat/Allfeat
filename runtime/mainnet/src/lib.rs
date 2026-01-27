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

//! The Melodie runtime.

#![cfg_attr(not(feature = "std"), no_std)]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

extern crate alloc;
use alloc::vec::Vec;

// allfeat
pub use allfeat_primitives::{AccountId, Address, Balance, BlockNumber, Moment, Nonce, Signature};

use apis::RUNTIME_API_VERSIONS;
use pallet_meta_tx::MetaTxMarker;
use sp_runtime::{
    generic::{self, ExtensionVersion},
    traits::NumberFor,
};
use sp_version::{RuntimeVersion, runtime_version};

#[cfg(any(feature = "std", test))]
pub use frame_system::Call as SystemCall;
#[cfg(any(feature = "std", test))]
pub use pallet_balances::Call as BalancesCall;

#[cfg(feature = "std")]
use sp_version::NativeVersion;

pub mod apis;
pub use apis::RuntimeApi;

/// Constant values used within the runtime.
pub mod constants;
pub use constants::time::*;

mod pallets;
pub use pallets::*;
mod genesis;

mod weights;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;

/// Runtime version.
#[runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: alloc::borrow::Cow::Borrowed("allfeat"),
    impl_name: alloc::borrow::Cow::Borrowed("allfeat-allfeat"),
    authoring_version: 1,
    spec_version: 100,
    impl_version: 0,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
    system_version: 1,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

/// Block header type as expected by this runtime.
pub type Header = allfeat_primitives::Header;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;

/// The `TransactionExtension` to the basic transaction logic.
pub type TxBareExtension = (
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

pub const META_EXTENSION_VERSION: ExtensionVersion = 0;

#[cfg(feature = "runtime-benchmarks")]
pub type MetaTxExtension = pallet_meta_tx::WeightlessExtension<Runtime>;

#[cfg(not(feature = "runtime-benchmarks"))]
/// Meta transaction extension.
pub type MetaTxExtension = (
    pallet_verify_signature::VerifySignature<Runtime>,
    MetaTxBareExtension,
);

/// Meta transaction extension without signature information.
///
/// Helper type used to decode the part of the extension which should be signed.
pub type MetaTxBareExtension = (
    MetaTxMarker<Runtime>,
    frame_system::CheckNonZeroSender<Runtime>,
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckMortality<Runtime>,
    frame_system::CheckNonce<Runtime>,
);

/// Transaction extension for signed transactions (v4 format, compatible with Polkadot-JS).
/// Note: VerifySignature is NOT included here because the signature is verified at the
/// extrinsic level for Signed (v4) transactions.
pub type TxExtension = TxBareExtension;

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
    generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, TxExtension>;

/// Executive: handles dispatch to the various modules.
pub type RuntimeExecutive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPalletsWithSystem,
>;

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
    pub type Utility = pallet_utility;

    #[runtime::pallet_index(2)]
    pub type Aura = pallet_aura;

    #[runtime::pallet_index(3)]
    pub type Timestamp = pallet_timestamp;

    #[runtime::pallet_index(4)]
    pub type Authorship = pallet_authorship;

    #[runtime::pallet_index(5)]
    pub type Balances = pallet_balances;

    #[runtime::pallet_index(6)]
    pub type TransactionPayment = pallet_transaction_payment;

    #[runtime::pallet_index(7)]
    pub type Validators = pallet_validators;

    #[runtime::pallet_index(8)]
    pub type Session = pallet_session;

    #[runtime::pallet_index(9)]
    pub type Grandpa = pallet_grandpa;

    #[runtime::pallet_index(10)]
    pub type Sudo = pallet_sudo;

    #[runtime::pallet_index(13)]
    pub type Historical = pallet_session::historical;

    #[runtime::pallet_index(14)]
    pub type Scheduler = pallet_scheduler;

    #[runtime::pallet_index(15)]
    pub type Preimage = pallet_preimage;

    #[runtime::pallet_index(16)]
    pub type Proxy = pallet_proxy;

    #[runtime::pallet_index(17)]
    pub type Multisig = pallet_multisig;

    #[runtime::pallet_index(18)]
    pub type TokenAllocation = pallet_token_allocation;

    #[runtime::pallet_index(19)]
    pub type Treasury = pallet_treasury;

    #[runtime::pallet_index(20)]
    pub type MetaTx = pallet_meta_tx;

    #[runtime::pallet_index(21)]
    pub type VerifySignature = pallet_verify_signature;
}
