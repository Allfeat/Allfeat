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

use super::Validators as ValidatorsStorage;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok, traits::OnInitialize};

#[test]
fn genesis_validators_are_set_correctly() {
    new_test_ext().execute_with(|| {
        let validators = ValidatorsStorage::<Test>::get();
        assert_eq!(validators, vec![1, 2, 3]);
    });
}

#[test]
fn session_should_use_validator_set() {
    new_test_ext().execute_with(|| {
        for n in 0..3 {
            Session::on_initialize(n);
        }
        let current_validators = Session::validators();
        assert_eq!(current_validators, vec![1, 2, 3]);
    });
}

#[test]
fn historical_session_is_recorded_with_root_and_count() {
    new_test_ext().execute_with(|| {
        for n in 0..3 {
            Session::on_initialize(n);
        }

        let session_index = pallet_session::Pallet::<Test>::current_index();
        let record = pallet_session::historical::HistoricalSessions::<Test>::get(session_index)
            .expect("historical session must exist");

        assert!(record.0 != Default::default()); // root hash non trivial
        assert_eq!(record.1, 3); // validator count
    });
}

#[test]
fn validators_can_be_added_up_to_max_individually() {
    new_test_ext().execute_with(|| {
        // Initial validators: [1, 2, 3]
        // MaxValidators: 5
        assert_ok!(Validators::add_validator(RuntimeOrigin::root(), 4));
        assert_ok!(Validators::add_validator(RuntimeOrigin::root(), 5));

        assert_eq!(ValidatorsStorage::<Test>::get(), vec![1, 2, 3, 4, 5]);

        assert_noop!(
            Validators::add_validator(RuntimeOrigin::root(), 6),
            super::Error::<Test>::TooManyValidators
        );
    });
}

#[test]
fn adding_existing_validator_should_fail() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Validators::add_validator(RuntimeOrigin::root(), 1),
            super::Error::<Test>::ValidatorAlreadyPresent
        );
    });
}

#[test]
fn validator_can_be_removed() {
    new_test_ext().execute_with(|| {
        assert_ok!(Validators::remove_validator(RuntimeOrigin::root(), 2));
        assert_eq!(ValidatorsStorage::<Test>::get(), vec![1, 3]);
    });
}

#[test]
fn removing_nonexistent_validator_should_fail() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Validators::remove_validator(RuntimeOrigin::root(), 42),
            super::Error::<Test>::ValidatorNotFound
        );
    });
}

#[test]
fn non_root_cannot_add_or_remove_validators() {
    new_test_ext().execute_with(|| {
        let origin = RuntimeOrigin::signed(99);
        assert_noop!(
            Validators::add_validator(origin.clone(), 42),
            sp_runtime::DispatchError::BadOrigin
        );
        assert_noop!(
            Validators::remove_validator(origin, 1),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn validator_set_changes_take_effect_in_next_session() {
    new_test_ext().execute_with(|| {
        for n in 0..3 {
            Session::on_initialize(n);
        }
        assert_eq!(Session::current_index(), 1);
        assert_eq!(Session::validators(), vec![1, 2, 3]);

        assert_ok!(Validators::add_validator(RuntimeOrigin::root(), 4));
        assert_ok!(Validators::remove_validator(RuntimeOrigin::root(), 1));

        for n in 3..6 {
            Session::on_initialize(n);
        }

        assert_eq!(Session::validators(), vec![1, 2, 3]);

        for n in 6..9 {
            Session::on_initialize(n);
        }

        assert_eq!(Session::validators(), vec![2, 3, 4]);
    });
}
