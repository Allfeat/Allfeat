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
//
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
use allfeat_support::{traits::Midds, types::IPINameNumber};
#[cfg(feature = "runtime-benchmarks")]
use alloc::vec;
use alloc::vec::Vec;
use core::marker::PhantomData;
use frame_support::{
	sp_runtime::{traits::Hash as HashT, DispatchError, RuntimeDebug},
	traits::ConstU32,
	BoundedVec,
};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

pub type NameLike = BoundedVec<u8, ConstU32<256>>;

#[derive(Encode, Default, MaxEncodedLen, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct Stakeholder<Hash>
where
	Hash: HashT,
{
	pub ipi: Option<IPINameNumber>,
	pub first_name: Option<NameLike>,
	pub last_name: Option<NameLike>,
	pub nickname: Option<NameLike>,
	_phantom_data: PhantomData<Hash>,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum EditableStakeholderField {
	IPIBase(Option<IPINameNumber>),
	FirstName(Option<NameLike>),
	LastName(Option<NameLike>),
	Nickname(Option<NameLike>),
}

impl Default for EditableStakeholderField {
	fn default() -> Self {
		Self::IPIBase(Default::default())
	}
}

impl<Hash> Midds for Stakeholder<Hash>
where
	Hash: HashT,
{
	type Hash = Hash;
	type EditableFields = EditableStakeholderField;

	fn is_complete(&self) -> bool {
		self.ipi.is_some() &&
			(self.nickname.is_some() || (self.last_name.is_some() || self.first_name.is_some()))
	}

	fn is_valid(&self) -> bool {
		// IPI valid format check
		if let Some(ipi) = &self.ipi {
			return ipi.0 < 100_000_000_000 && ipi.0 > 1;
		}

		true
	}

	fn hash(&self) -> <Hash as HashT>::Output {
		let mut bytes = Vec::new();

		bytes.extend_from_slice(&self.ipi.encode());
		bytes.extend_from_slice(&self.first_name.encode());
		bytes.extend_from_slice(&self.last_name.encode());
		bytes.extend_from_slice(&self.nickname.encode());

		<Self::Hash as HashT>::hash(&bytes)
	}

	fn update_field(&mut self, data: Self::EditableFields) -> Result<(), DispatchError> {
		match data {
			EditableStakeholderField::IPIBase(x) => self.ipi = x,
			EditableStakeholderField::LastName(x) => self.last_name = x,
			EditableStakeholderField::Nickname(x) => self.nickname = x,
			EditableStakeholderField::FirstName(x) => self.first_name = x,
		}
		Ok(())
	}

	/// Create a basic instance of the midds.
	#[cfg(feature = "runtime-benchmarks")]
	fn create_midds() -> Self {
		let sample_data: NameLike = vec![0; 32].try_into().unwrap();

		Self {
			ipi: Some(123456789u64.try_into().expect("benchmark valid value")),
			first_name: Some(sample_data.clone()),
			last_name: Some(sample_data.clone()),
			nickname: Some(sample_data),
			_phantom_data: Default::default(),
		}
	}
}
