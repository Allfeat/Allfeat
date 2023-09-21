// Copyright (c) 2022-2023 Allfeat labs
//
// Permission is hereby granted, free of charge, to any person obtaining
// a copy of this software and associated documentation files (the"Software"),
// to deal in the Software without restriction, including
// without limitation the rights to use, copy, modify, merge, publish,
// distribute, sublicense, and/or sell copies of the Software, and to
// permit persons to whom the Software is furnished to do so, subject to
// the following conditions:
//
// The above copyright notice and this permission notice shall be
// included in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
// NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE
// LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
// WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

use core::marker::PhantomData;
use frame_support::pallet_prelude::Get;
use pallet_artists::CandidateOf;
use pallet_contracts::chain_extension::{
	BufInBufOutState, ChainExtension, ChargedAmount, Environment, Ext, InitState, RetVal,
};
use parity_scale_codec::{Decode, Encode};
use sp_runtime::{DispatchError, ModuleError};

enum ArtistsFunc {
	// Constants
	CreationDepositAmount,
	NameMaxLength,
	// Chain State Queries
	Artists,
	Candidates,
}

impl TryFrom<u16> for ArtistsFunc {
	type Error = DispatchError;

	fn try_from(value: u16) -> Result<Self, Self::Error> {
		match value {
			1 => Ok(ArtistsFunc::CreationDepositAmount),
			2 => Ok(ArtistsFunc::NameMaxLength),
			51 => Ok(ArtistsFunc::Artists),
			52 => Ok(ArtistsFunc::Candidates),
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
			// Constants
			ArtistsFunc::CreationDepositAmount => {
				charge_weight_read(&mut env)?;
				let value = T::CreationDepositAmount::get();
				env.write(&value.encode(), false, None)?
			},
			ArtistsFunc::NameMaxLength => {
				charge_weight_read(&mut env)?;
				let value = T::NameMaxLength::get();
				env.write(&value.encode(), false, None)?
			},

			// Chain State Queries
			ArtistsFunc::Artists => {
				let account_id: T::AccountId = env.read_as()?;

				charge_weight_read(&mut env)?;
				let data = pallet_artists::Pallet::<T>::get_artist(account_id);
				env.write(&data.encode(), false, None)?
			},
			ArtistsFunc::Candidates => {
				let account_id: T::AccountId = env.read_as()?;

				charge_weight_read(&mut env)?;
				let data: Option<CandidateOf<T>> =
					pallet_artists::Pallet::<T>::get_candidate(account_id);
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

	// General errors:
	// ===============
	/// The caller doesn't have enough funds for the deposit
	NotEnoughFunds = 1,
	/// The given string is longer than `T::NameMaxLength`.
	NameTooLong = 2,

	// Candidate related errors:
	// =========================
	/// The account is already in the candidate list
	AlreadyACandidate = 3,
	/// The wanted candidate is not found in the Candidates Storage
	CandidateNotFound = 4,
	/// The caller isn't in the candidate list.
	NotACandidate = 5,

	// Artist related errors:
	// ======================
	/// This account already is a certificated artist account.
	AlreadyAnArtist = 6,
	/// The caller isn't a verified artist.
	NotAnArtist = 7,
	/// The wanted artist is not found in the Artists Storage
	ArtistNotFound = 8,

	/// Unknown error
	UnknownError = 99,
}

// TODO: macro to make the implement not verbose that much ?
impl TryFrom<DispatchError> for ArtistsError {
	type Error = DispatchError;

	fn try_from(value: DispatchError) -> Result<Self, Self::Error> {
		let error_text = match value {
			DispatchError::Module(ModuleError { message, .. }) => message,
			_ => Some("No module error Info"),
		};
		return match error_text {
			Some("NotEnoughFunds") => Ok(Self::NotEnoughFunds),
			Some("NameTooLong") => Ok(Self::NameTooLong),
			Some("AlreadyACandidate") => Ok(Self::AlreadyACandidate),
			Some("CandidateNotFound") => Ok(Self::CandidateNotFound),
			Some("NotACandidate") => Ok(Self::NotACandidate),
			Some("AlreadyAnArtist") => Ok(Self::AlreadyAnArtist),
			Some("NotAnArtist") => Ok(Self::NotAnArtist),
			Some("ArtistNotFound") => Ok(Self::ArtistNotFound),
			_ => Ok(ArtistsError::UnknownError),
		};
	}
}
