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

use frame_support::{pallet_prelude::TypedGet, sp_runtime::TokenError, testing_prelude::*};
use parity_scale_codec::Encode;

use crate::{MiddsOf, mock::*};

#[test]
fn it_registers_midds_to_pending_successfully() {
    sp_tracing::init_for_tests();

    let provider = 1;
    let midds = MockMiddsStruct { value: 1 };
    let expected_lock_cost = (midds.encoded_size() as u64)
        .saturating_mul(<<Test as crate::Config>::ByteDepositCost as TypedGet>::get());

    new_test_ext().execute_with(|| {
        assert_ok!(MockMidds::register(
            RuntimeOrigin::signed(provider),
            Box::new(midds.clone())
        ));

        assert_eq!(expected_lock_cost, Balances::reserved_balance(provider));
        assert_eq!(MiddsOf::<Test>::get(0).expect("testing value"), midds)
    })
}

#[test]
fn register_without_enough_funds_fail() {
    sp_tracing::init_for_tests();

    let provider = 5;
    let midds = MockMiddsStruct { value: 1 };
    let expected_lock_cost = (midds.encoded_size() as u64)
        .saturating_mul(<<Test as crate::Config>::ByteDepositCost as TypedGet>::get());

    new_test_ext().execute_with(|| {
        assert!(Balances::free_balance(provider) < expected_lock_cost);

        assert_err!(
            MockMidds::register(RuntimeOrigin::signed(provider), Box::new(midds.clone())),
            TokenError::FundsUnavailable
        );
    })
}

#[test]
fn register_same_midds_data_fail() {
    sp_tracing::init_for_tests();

    let provider = 1;
    let midds = MockMiddsStruct { value: 1 };

    new_test_ext().execute_with(|| {
        assert_ok!(MockMidds::register(
            RuntimeOrigin::signed(provider),
            Box::new(midds.clone())
        ));
        assert_err!(
            MockMidds::register(RuntimeOrigin::signed(provider), Box::new(midds.clone())),
            crate::Error::<Test>::MiddsDataAlreadyExist
        );
    })
}
