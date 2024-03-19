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

use core::marker::PhantomData;
use frame_support::pallet_prelude::Get;
use pallet_contracts::chain_extension::{
	BufInBufOutState, ChainExtension, ChargedAmount, Environment, Ext, InitState, RetVal,
};
use parity_scale_codec::{Decode, Encode};
use sp_runtime::{DispatchError, ModuleError};

enum ArtistsFunc {
	// Chain State Queries
	Artist,
}

impl TryFrom<u16> for ArtistsFunc {
	type Error = DispatchError;

	fn try_from(value: u16) -> Result<Self, Self::Error> {
		match value {
			51 => Ok(ArtistsFunc::Artist),
			_ => Err(DispatchError::Other("PalletArtistsExtension: Unimplemented func_id")),
		}
	}
}

/// Pallet Artists chain extension.
pub struct ArtistsExtension<T>(PhantomData<T>);
impl<T> Default for ArtistsExtension<T> {
	fn default() -> Self {
		ArtistsExtension(PhantomData)
	}
}

impl<T> ChainExtension<T> for ArtistsExtension<T>
where
	T: pallet_contracts::Config + pallet_artists::Config,
{
	fn call<E: Ext<T = T>>(
		&mut self,
		env: Environment<E, InitState>,
	) -> pallet_contracts::chain_extension::Result<RetVal> {
		let func_id: ArtistsFunc = env.func_id().try_into()?;
		let mut env = env.buf_in_buf_out();

		match func_id {
			// Chain State Queries
			ArtistsFunc::Artist => {
				let account_id: T::AccountId = env.read_as()?;

				charge_weight_read(&mut env)?;
				let data = pallet_artists::Pallet::<T>::get_artist_by_id(account_id);
				env.write(&data.encode(), false, None)?
			},
		};

		Ok(RetVal::Converging(ArtistsError::Success as u32))
	}
}

fn charge_weight_read<E, T>(
	env: &mut Environment<E, BufInBufOutState>,
) -> Result<ChargedAmount, DispatchError>
where
	E: Ext<T = T>,
	T: pallet_contracts::Config + pallet_artists::Config,
{
	let base_weight = <T as frame_system::Config>::DbWeight::get().reads(1);
	env.charge_weight(base_weight)
}

#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode, Debug)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum ArtistsError {
	/// Success
	Success = 0,
	/// Unknown error
	UnknownError = 99,
}

impl TryFrom<DispatchError> for ArtistsError {
	type Error = DispatchError;

	fn try_from(value: DispatchError) -> Result<Self, Self::Error> {
		let error_text = match value {
			DispatchError::Module(ModuleError { message, .. }) => message,
			_ => Some("No module error Info"),
		};
		return match error_text {
			_ => Ok(ArtistsError::UnknownError),
		};
	}
}
