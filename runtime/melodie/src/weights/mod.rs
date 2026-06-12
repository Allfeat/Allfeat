// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Expose the auto generated weight files.

pub mod ats;
pub mod balances;
pub mod block_weights;
pub mod collator_selection;
pub mod cumulus_pallet_parachain_system;
pub mod cumulus_pallet_weight_reclaim;
pub mod cumulus_pallet_xcmp_queue;
pub mod extrinsic_weights;
pub mod message_queue;
pub mod meta_tx;
pub mod midds_musical_works;
pub mod midds_recordings;
pub mod midds_releases;
pub mod multisig;
pub mod paritydb_weights;
pub mod preimage;
pub mod proxy;
pub mod rocksdb_weights;
pub mod safe_mode;
pub mod scheduler;
pub mod session;
pub mod sudo;
pub mod system;
pub mod timestamp;
pub mod utility;
pub mod verify_signature;

pub use block_weights::constants::BlockExecutionWeight;
pub use extrinsic_weights::constants::ExtrinsicBaseWeight;
pub use paritydb_weights::constants::ParityDbWeight;
// `RocksDbWeight` stays available for collators running RocksDB (see `DbWeight`).
