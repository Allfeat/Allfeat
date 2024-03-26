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

#![cfg_attr(not(feature = "std"), no_std)]

use enumflags2::BitFlags;
use frame_support::traits::{Currency, OriginTrait};
use frame_system::pallet_prelude::BlockNumberFor;
use hex_literal::hex;
use pallet_evm::AddressMapping;
use pallet_evm_precompile_nfts_types::solidity::{
	CollectionConfig, CollectionSettings, MintSettings,
};
use pallet_nfts::CollectionSetting;
use sp_core::H160;
use sp_runtime::{traits::StaticLookup, BuildStorage};

type AccountIdOf<R> = <R as frame_system::Config>::AccountId;

type BalanceOf<R> = <R as pallet_balances::Config>::Balance;

type NftsBalanceOf<R> = <<R as pallet_nfts::Config>::Currency as Currency<
	<R as frame_system::Config>::AccountId,
>>::Balance;

pub const ALICE: H160 = H160::repeat_byte(0xAA);
pub const BOB: H160 = H160::repeat_byte(0xBB);
pub const CHARLIE: H160 = H160::repeat_byte(0xCC);

pub const ALICE_COLLECTION_PRECOMPILE_ADDRESS: [u8; 20] =
	hex!("FFFFFFFF00000000000000000000000000000000");
pub const BOB_COLLECTION_PRECOMPILE_ADDRESS: [u8; 20] =
	hex!("FFFFFFFF00000000000000000000000000000001");

pub fn solidity_collection_config_all_enabled() -> CollectionConfig {
	CollectionConfig {
		settings: CollectionSettings::all_enabled(),
		max_supply: Default::default(),
		mint_settings: MintSettings::item_settings_all_enabled(),
	}
}

type OriginOf<R> = <R as frame_system::Config>::RuntimeOrigin;

type CollectionConfigFor<R> = pallet_nfts::CollectionConfig<
	NftsBalanceOf<R>,
	BlockNumberFor<R>,
	<R as pallet_nfts::Config>::CollectionId,
>;

pub struct ExtBuilder<R>
where
	R: pallet_balances::Config,
{
	// endowed accounts with balances
	balances: Vec<(AccountIdOf<R>, BalanceOf<R>)>,
}

impl<R> Default for ExtBuilder<R>
where
	R: pallet_balances::Config,
{
	fn default() -> ExtBuilder<R> {
		ExtBuilder { balances: vec![] }
	}
}

impl<R> ExtBuilder<R>
where
	R: pallet_balances::Config + frame_system::Config + pallet_nfts::Config + pallet_evm::Config,
	BlockNumberFor<R>: From<u32>,
{
	pub fn with_balances(mut self, balances: Vec<(AccountIdOf<R>, BalanceOf<R>)>) -> Self {
		self.balances = balances;
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::<R>::default()
			.build_storage()
			.expect("Frame system builds valid default genesis config");

		pallet_balances::GenesisConfig::<R> { balances: self.balances }
			.assimilate_storage(&mut t)
			.expect("Pallet balances storage can be assimilated");

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| frame_system::Pallet::<R>::set_block_number(1u32.into()));
		ext
	}

	/// Same as `build` but also create two mock collections to work with for testing.
	pub fn build_with_collections(self) -> sp_io::TestExternalities {
		let mut ext = self.build();
		ext.execute_with(|| {
			let alice: AccountIdOf<R> = R::AddressMapping::into_account_id(ALICE);
			let bob: AccountIdOf<R> = R::AddressMapping::into_account_id(BOB);

			pallet_nfts::Pallet::<R>::force_create(
				OriginOf::<R>::root(),
				<R as frame_system::Config>::Lookup::unlookup(alice),
				Self::default_collection_config(),
			)
			.expect("mocking call");
			pallet_nfts::Pallet::<R>::force_create(
				OriginOf::<R>::root(),
				<R as frame_system::Config>::Lookup::unlookup(bob),
				Self::default_collection_config(),
			)
			.expect("mocking call 2");
		});
		ext
	}

	fn collection_config_from_disabled_settings(
		settings: BitFlags<CollectionSetting>,
	) -> CollectionConfigFor<R> {
		pallet_nfts::CollectionConfig {
			settings: pallet_nfts::CollectionSettings::from_disabled(settings),
			max_supply: None,
			mint_settings: pallet_nfts::MintSettings::default(),
		}
	}

	fn default_collection_config() -> CollectionConfigFor<R> {
		Self::collection_config_from_disabled_settings(CollectionSetting::DepositRequired.into())
	}
}
