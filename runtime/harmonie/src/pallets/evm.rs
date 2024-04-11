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
use frame_support::{parameter_types, traits::FindAuthor};
use pallet_ethereum::EthereumBlockHashMapping;
use pallet_evm::{EVMCurrencyAdapter, IdentityAddressMapping};
use shared_runtime::TransactionPaymentGasPrice;
use sp_runtime::{ConsensusEngineId, RuntimeAppPublic};

pub struct FindAuthorTruncated<F>(PhantomData<F>);
impl<F: FindAuthor<u32>> FindAuthor<H160> for FindAuthorTruncated<F> {
	fn find_author<'a, I>(digests: I) -> Option<H160>
	where
		I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
	{
		if let Some(author_index) = F::find_author(digests) {
			let (authority_id, _) = Babe::authorities()[author_index as usize].clone();
			return Some(H160::from_slice(&authority_id.to_raw_vec()[4..24]));
		}
		None
	}
}

parameter_types! {
	pub BlockGasLimit: U256 = U256::from(BLOCK_GAS_LIMIT);
	pub PrecompilesValue: AllfeatPrecompiles<Runtime> = AllfeatPrecompiles::<_>::new();
	pub WeightPerGas: Weight = Weight::from_parts(fp_evm::weight_per_gas(BLOCK_GAS_LIMIT, shared_runtime::NORMAL_DISPATCH_RATIO, WEIGHT_MILLISECS_PER_BLOCK), 0);
	pub SuicideQuickClearLimit: u32 = 0;
	pub GasLimitPovSizeRatio: u64 = 0;
}

const BLOCK_GAS_LIMIT: u64 = 75_000_000;

impl pallet_evm::Config for Runtime {
	type FeeCalculator = TransactionPaymentGasPrice<Runtime, WeightPerGas>;
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;
	type BlockHashMapping = EthereumBlockHashMapping<Runtime>;
	type CallOrigin = pallet_evm::EnsureAddressRoot<AccountId>;
	type WithdrawOrigin = pallet_evm::EnsureAddressNever<AccountId>;
	type AddressMapping = IdentityAddressMapping;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type PrecompilesType = AllfeatPrecompiles<Self>;
	type PrecompilesValue = PrecompilesValue;
	type ChainId = ConstU64<441>;
	type BlockGasLimit = BlockGasLimit;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type OnChargeTransaction = EVMCurrencyAdapter<Balances, DealWithFees<Runtime>>;
	type OnCreate = ();
	type FindAuthor = FindAuthorTruncated<Babe>;
	type GasLimitPovSizeRatio = GasLimitPovSizeRatio;
	type Timestamp = Timestamp;
	type SuicideQuickClearLimit = SuicideQuickClearLimit;
	type WeightInfo = ();
}
