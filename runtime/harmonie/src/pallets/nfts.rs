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

use crate::*;
use frame_support::{parameter_types, traits::AsEnsureOriginWithArg};
use frame_system::{EnsureRoot, EnsureSigned};
use sp_runtime::traits::Verify;

impl
	pallet_evm_precompile_nfts_collections::AddressToCollectionId<
		<Runtime as pallet_nfts::Config>::CollectionId,
	> for Runtime
{
	fn address_to_collection_id(
		address: H160,
	) -> Option<<Runtime as pallet_nfts::Config>::CollectionId> {
		let mut data = [0u8; 16];
		let address_bytes: [u8; 20] = address.into();
		if precompiles::NFTS_PRECOMPILE_ADDRESS_PREFIX.eq(&address_bytes[0..4]) {
			data.copy_from_slice(&address_bytes[4..20]);
			Some(u128::from_be_bytes(data))
		} else {
			None
		}
	}

	fn collection_id_to_address(
		collection_id: <Runtime as pallet_nfts::Config>::CollectionId,
	) -> H160 {
		let mut data = [0u8; 20];
		data[0..4].copy_from_slice(precompiles::NFTS_PRECOMPILE_ADDRESS_PREFIX);
		data[4..20].copy_from_slice(&collection_id.to_be_bytes());
		H160::from(data)
	}
}

parameter_types! {
	pub Features: pallet_nfts::PalletFeatures = pallet_nfts::PalletFeatures::all_enabled();
	pub const MaxAttributesPerCall: u32 = 10;
	pub const CollectionDeposit: Balance = 10 * AFT;
	pub const ItemDeposit: Balance = deposit(1,0);
	pub const ApprovalsLimit: u32 = 20;
	pub const ItemAttributesApprovalsLimit: u32 = 20;
	pub const MaxTips: u32 = 10;
	pub const MaxDeadlineDuration: BlockNumber = 12 * 30 * DAYS;
	pub const MetadataDepositBase: Balance = 1 * AFT;
	pub const MetadataDepositPerByte: Balance = deposit(0,1);
}

impl pallet_nfts::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type CollectionId = u128;
	type ItemId = u128;
	type Currency = Balances;
	type ForceOrigin = EnsureRoot<AccountId>;
	type CollectionDeposit = CollectionDeposit;
	type ItemDeposit = ItemDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type AttributeDepositBase = MetadataDepositBase;
	type DepositPerByte = MetadataDepositPerByte;
	type StringLimit = ConstU32<256>;
	type KeyLimit = ConstU32<64>;
	type ValueLimit = ConstU32<256>;
	type ApprovalsLimit = ApprovalsLimit;
	type ItemAttributesApprovalsLimit = ItemAttributesApprovalsLimit;
	type MaxTips = MaxTips;
	type MaxDeadlineDuration = MaxDeadlineDuration;
	type MaxAttributesPerCall = MaxAttributesPerCall;
	type Features = Features;
	type OffchainSignature = Signature;
	type OffchainPublic = <Signature as Verify>::Signer;
	type WeightInfo = pallet_nfts::weights::SubstrateWeight<Runtime>;
	#[cfg(feature = "runtime-benchmarks")]
	type Helper = ();
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<AccountId>>;
	type Locker = ();
}
