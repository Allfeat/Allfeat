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

extern crate alloc;
use allfeat_support::{
	traits::Midds,
	types::{MusicalWorkTitle, MusicalWorkType, ISWC},
};
use alloc::{vec, vec::Vec};
use core::marker::PhantomData;
use frame_support::{
	ensure,
	sp_runtime::{traits::Hash as HashT, DispatchError, DispatchResult, Percent, RuntimeDebug},
	traits::ConstU32,
	BoundedVec, Parameter,
};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

pub type SharesVec<StakeholderHashId> = BoundedVec<Share<StakeholderHashId>, ConstU32<64>>;

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum MusicalWorkEditableField<StakeholderHashId> {
	ISWC(Option<ISWC>),
	Title(Option<MusicalWorkTitle>),
	Duration(Option<u32>),
	Type(Option<MusicalWorkType>),
	Shares(SharesEditAction<StakeholderHashId>),
}

impl<StakeholderHashId> Default for MusicalWorkEditableField<StakeholderHashId> {
	fn default() -> Self {
		MusicalWorkEditableField::ISWC(Some(Default::default()))
	}
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum SharesEditAction<StakeholderHashId> {
	Add(Share<StakeholderHashId>),
	Remove(u8),
}

#[derive(Encode, Default, MaxEncodedLen, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct MusicalWork<Hash, StakeholderHashId> {
	pub iswc: Option<ISWC>,
	pub title: Option<MusicalWorkTitle>,
	pub duration: Option<u32>,
	pub _type: Option<MusicalWorkType>,
	pub shares: Option<SharesVec<StakeholderHashId>>,
	_phantom_data: PhantomData<Hash>,
}

impl<Hash, StakeholderHashId> MusicalWork<Hash, StakeholderHashId> {
	pub fn validate_shares(&self) -> Result<(), DispatchError> {
		if let Some(shares) = &self.shares {
			let total_performance_share: u8 = shares
				.iter()
				.map(|share| share.share_info.performance_share.deconstruct())
				.sum();

			let total_mechanical_share: u8 =
				shares.iter().map(|share| share.share_info.mechanical_share.deconstruct()).sum();

			ensure!(
				total_performance_share == 100 && total_mechanical_share == 100,
				DispatchError::Other("Shares aren't equals to 100%")
			);
		};

		Ok(())
	}
}

impl<Hash, StakeholderHashId> Midds for MusicalWork<Hash, StakeholderHashId>
where
	Hash: HashT,
	StakeholderHashId: Parameter + 'static,
{
	type Hash = Hash;
	type EditableFields = MusicalWorkEditableField<StakeholderHashId>;

	fn is_complete(&self) -> bool {
		self.iswc.is_some() &&
			self.duration.is_some() &&
			self.title.is_some() &&
			self._type.is_some() &&
			self.shares.is_some() &&
			self.validate_shares().is_ok() // Shares should be valid to be complete
	}

	fn is_valid(&self) -> bool {
		true
	}

	fn hash(&self) -> <Self::Hash as HashT>::Output {
		let mut bytes = Vec::new();

		bytes.extend_from_slice(&self.iswc.encode());
		bytes.extend_from_slice(&self._type.encode());
		bytes.extend_from_slice(&self.duration.encode());
		bytes.extend_from_slice(&self.title.encode());
		bytes.extend_from_slice(&self.shares.encode());

		<Self::Hash as HashT>::hash(&bytes)
	}

	fn update_field(&mut self, data: Self::EditableFields) -> DispatchResult {
		match data {
			MusicalWorkEditableField::ISWC(x) => self.iswc = x,
			MusicalWorkEditableField::Type(x) => self._type = x,
			MusicalWorkEditableField::Duration(x) => self.duration = x,
			MusicalWorkEditableField::Title(x) => self.title = x,
			MusicalWorkEditableField::Shares(action) => match action {
				SharesEditAction::Add(share) =>
					if self.shares.is_some() {
						self.shares.as_mut().expect("already checked").try_push(share).map_err(
							|_| {
								DispatchError::Other(
									"Cannot add new Share, potentially hit the limit.",
								)
							},
						)?
					} else {
						self.shares = Some(vec![share].try_into().unwrap())
					},
				SharesEditAction::Remove(index) =>
					if let Some(shares) = self.shares.as_mut() {
						if (index as usize) < shares.len() {
							shares.swap_remove(index as usize);
						}
					},
			},
		};
		Ok(())
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn create_midds() -> Self {
		let sample_data = vec![0; 32].try_into().unwrap();

		Self {
			iswc: Default::default(),
			title: Some(sample_data),
			duration: Some(1u32),
			_type: Some(MusicalWorkType::Song),
			shares: Default::default(),
			_phantom_data: Default::default(),
		}
	}
}

#[derive(Encode, Default, MaxEncodedLen, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct Share<StakeholderHashId> {
	// Reference to the Stakeholder MIDDS from his MIDDS Hash ID.
	pub stakeholder_id: StakeholderHashId,
	pub share_info: ShareInfo,
}

#[derive(Encode, Default, MaxEncodedLen, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct ShareInfo {
	pub role: Role,
	pub performance_share: Percent,
	pub mechanical_share: Percent,
}

#[derive(Default, MaxEncodedLen, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum Role {
	#[default]
	A,
	AD,
	AM,
	AR,
	C,
	CA,
	E,
	ES,
	PA,
	PR,
	SA,
	SE,
	SR,
	TR,
}
