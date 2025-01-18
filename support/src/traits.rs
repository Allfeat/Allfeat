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

use frame_support::{pallet_prelude::DispatchResult, sp_runtime::traits::Hash as HashT, Parameter};
use parity_scale_codec::Encode;

/// Base definition of a MIDDS (Music Industry Decentralized Data Structure)
pub trait Midds
where
	Self: Encode,
{
	type Hash: HashT;
	type EditableFields: Parameter + Default;

	/// Return true if the MIDDS is judged as complete, all fields are filled.
	/// (e.g in case of Option fields, they all should be `Some`)
	fn is_complete(&self) -> bool;
	/// Perform any required check that ensure MIDDS data are all valid
	fn is_valid(&self) -> bool;
	/// Hash combined fields of the MIDDS to output a general MIDDS Hash.
	fn hash(&self) -> <Self::Hash as HashT>::Output;
	fn total_bytes(&self) -> u32 {
		self.encoded_size() as u32
	}
	fn update_field(&mut self, data: Self::EditableFields) -> DispatchResult;

	/// Create a basic instance of the midds.
	#[cfg(feature = "runtime-benchmarks")]
	fn create_midds() -> Self;
}
