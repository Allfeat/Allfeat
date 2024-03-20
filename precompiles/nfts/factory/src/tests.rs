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

use crate::{mock::*, NftsFactoryPrecompileCall};
use pallet_evm_precompile_nfts_tests::{solidity_collection_config_all_enabled, ExtBuilder, ALICE};
use precompile_utils::testing::*;

type PCall = NftsFactoryPrecompileCall<Runtime>;
fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

#[test]
fn selectors() {
	assert!(PCall::create_selectors().contains(&0x28d66e67));
	assert!(PCall::set_accept_ownership_selectors().contains(&0x8c462cc0));
}

#[test]
fn create_works() {
	ExtBuilder::<Runtime>::default()
		.with_balances(vec![(ALICE.into(), 1000)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					ALICE,
					Precompile1,
					PCall::create {
						admin: ALICE.into(),
						config: solidity_collection_config_all_enabled(),
					},
				)
				.execute_returns(true);
		})
}

#[test]
fn set_accept_ownership_works() {
	todo!()
}
