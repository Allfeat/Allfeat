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

#![cfg(test)]

use super::*;
use frame_support::sp_runtime::testing::H256;
use mock::*;

fn generate_midds_id() -> MiddsHashIdOf<Test> {
	H256::random()
}

mod vote {
	use core::time::Duration;

	use crate::{
		tests::{Balance, RuntimeOrigin, System, Test, Timestamp},
		types::PrecertifInfos,
		Certifications, Config, Error, Event, PreCertificationsQueue, Votes,
	};
	use alloc::collections::VecDeque;
	use frame_support::traits::Get;

	use super::{generate_midds_id, new_test_ext, CertifState, CertifStatus, Certification};
	use allfeat_support::traits::Certifier;
	use frame_support::{assert_err, assert_ok};

	#[test]
	fn vote_without_min_shouldnt_works() {
		let truster_id = 1;
		let midds_id = generate_midds_id();
		let vote_amount: u64 = 1;

		new_test_ext().execute_with(|| {
			Certification::add_to_certif_process(midds_id)
				.expect("Test midds ID added to certif process");

			assert_err!(
				Certification::vote_for(RuntimeOrigin::signed(truster_id), midds_id, vote_amount),
				Error::<Test>::MinVoteAmountRequired
			);
		})
	}

	#[test]
	fn vote_with_max_shouldnt_works() {
		let truster_id = 1;
		let midds_id = generate_midds_id();
		let vote_amount: u64 = 2000;

		new_test_ext().execute_with(|| {
			Certification::add_to_certif_process(midds_id)
				.expect("Test midds ID added to certif process");

			assert_err!(
				Certification::vote_for(RuntimeOrigin::signed(truster_id), midds_id, vote_amount),
				Error::<Test>::MaxVoteAmountExceeded
			);
		})
	}

	#[test]
	fn vote_for_unknown_midds_shouldnt_works() {
		let truster_id = 1;
		let midds_id = generate_midds_id();
		let vote_amount: u64 = 20;

		new_test_ext().execute_with(|| {
			assert_err!(
				Certification::vote_for(RuntimeOrigin::signed(truster_id), midds_id, vote_amount),
				Error::<Test>::NoCertifState
			);
		})
	}

	#[test]
	fn vote_works() {
		let truster_id = 1;
		let midds_id = generate_midds_id();
		let vote_amount: u64 = 20;

		let mut expected_certif_state = CertifState::<u64>::new();
		match expected_certif_state.status {
			CertifStatus::Voting(ref mut infos) => infos.add_staked(vote_amount),
			_ => unreachable!("expecting VOTING status"),
		}

		new_test_ext().execute_with(|| {
			Certification::add_to_certif_process(midds_id)
				.expect("Test midds ID added to certif process");

			// vote_for extrinsic submittion
			assert_ok!(Certification::vote_for(
				RuntimeOrigin::signed(truster_id),
				midds_id,
				vote_amount
			));

			// Direct storages check
			assert_eq!(Certifications::<Test>::get(midds_id), Some(expected_certif_state.clone()));
			assert_eq!(Votes::<Test>::get(truster_id, midds_id), Some(vote_amount));

			System::assert_last_event(
				Event::<Test>::Voted { truster: truster_id, midds_id, vote_amount }.into(),
			);

			// Ensure that the vote funds are reserved
			assert_eq!(Balance::reserved_balance(truster_id), vote_amount);

			let new_vote_amount = 5;
			match expected_certif_state.status {
				CertifStatus::Voting(ref mut infos) => infos.add_staked(new_vote_amount),
				_ => unreachable!("expecting VOTING status"),
			}

			// Emit a new vote to ensure it update correctly
			assert_ok!(Certification::vote_for(
				RuntimeOrigin::signed(truster_id),
				midds_id,
				new_vote_amount
			));

			// Direct storages check
			assert_eq!(Certifications::<Test>::get(midds_id), Some(expected_certif_state));
			assert_eq!(
				Votes::<Test>::get(truster_id, midds_id),
				Some(vote_amount + new_vote_amount)
			);

			System::assert_last_event(
				Event::<Test>::Voted {
					truster: truster_id,
					midds_id,
					vote_amount: new_vote_amount,
				}
				.into(),
			);

			// Ensure that the new vote funds are reserved
			assert_eq!(Balance::reserved_balance(truster_id), vote_amount + new_vote_amount);
		})
	}

	#[test]
	fn vote_above_threshold_trigger_precertif() {
		let truster_id_1 = 1;
		let truster_id_2 = 2;
		let midds_id = generate_midds_id();
		let vote_amount: u64 = 500;

		let mut expected_certif_state = CertifState::<u64>::new();
		match expected_certif_state.status {
			CertifStatus::Voting(ref mut infos) => infos.add_staked(vote_amount),
			_ => unreachable!("expecting VOTING status"),
		}

		new_test_ext().execute_with(|| {
			Timestamp::set_timestamp(1);

			Certification::add_to_certif_process(midds_id)
				.expect("Test midds ID added to certif process");

			let required_for_precertif: u64 = <Test as Config>::ThresholdCertifiedAmount::get();
			assert_eq!(required_for_precertif, vote_amount * 2);

			assert_ok!(Certification::vote_for(
				RuntimeOrigin::signed(truster_id_1),
				midds_id,
				vote_amount
			));
			// This vote should trigger the threshold and upgrade to precertif.
			assert_ok!(Certification::vote_for(
				RuntimeOrigin::signed(truster_id_2),
				midds_id,
				vote_amount
			));
			assert_err!(
				Certification::vote_for(RuntimeOrigin::signed(3), midds_id, vote_amount),
				Error::<Test>::NoVotingPeriod
			);

			expected_certif_state.status = CertifStatus::Precertif(PrecertifInfos {
				precertif_timestamp: Duration::from_millis(1),
			});
			let mut expected_queue = VecDeque::new();
			expected_queue.push_back(midds_id);
			assert_eq!(PreCertificationsQueue::<Test>::get(), expected_queue);
			assert_eq!(Certifications::<Test>::get(midds_id), Some(expected_certif_state));
		})
	}
}

mod hooks {
	mod on_idle {
		use core::time::Duration;

		use alloc::collections::VecDeque;
		use frame_support::traits::{Get, Hooks, UnixTime};

		use crate::{
			tests::{
				generate_midds_id, new_test_ext, CertifState, CertifStatus, Certification, System,
				Test, Timestamp,
			},
			types::PrecertifInfos,
			Certifications, Config, MiddsHashIdOf, PreCertificationsQueue,
		};

		#[test]
		#[should_panic(expected = "certif data not found, this is highly unexpected.")]
		fn no_certif_state_during_precertif_upgrade_panics() {
			let midds_id = generate_midds_id();
			let test_queue: Vec<MiddsHashIdOf<Test>> = vec![midds_id];

			new_test_ext().execute_with(|| {
				Timestamp::set_timestamp(1);
				System::set_block_number(1);
				let current_block = System::block_number();

				// Add the mock MIDDS ID to the queue.
				PreCertificationsQueue::<Test>::put(test_queue.clone());

				let now = <Timestamp as UnixTime>::now();
				assert_eq!(now, Duration::from_millis(1));

				// No timestamp change, nothing should be removed from the queue.
				Certification::on_idle(current_block, 1000000.into());
			})
		}

		#[test]
		fn precertif_upgrade_works() {
			let midds_id = generate_midds_id();
			let midds_id_2 = generate_midds_id();
			let midds_id_3 = generate_midds_id();
			let mut test_queue: VecDeque<MiddsHashIdOf<Test>> = VecDeque::new();
			let mut certif_state = CertifState::<u64>::new();
			let mut certif_state_2 = CertifState::<u64>::new();
			certif_state.status = CertifStatus::Precertif(PrecertifInfos {
				precertif_timestamp: Duration::from_millis(0),
			});
			certif_state_2.status = CertifStatus::Precertif(PrecertifInfos {
				precertif_timestamp: Duration::from_millis(102),
			});
			test_queue.push_back(midds_id);
			test_queue.push_back(midds_id_2);
			test_queue.push_back(midds_id_3);

			let expected_wait_time: u128 = <Test as Config>::WaitTimeForPreToCertif::get();
			let expected_wait_time_as_u64: u64 =
				expected_wait_time.try_into().expect("Test conversion should work");

			let base_timestamp = 100;

			new_test_ext().execute_with(|| {
				Timestamp::set_timestamp(base_timestamp);
				System::set_block_number(1);
				let current_block = System::block_number();

				// Add the certif state of the mock MIDDS to storage.
				Certifications::<Test>::insert(midds_id, certif_state.clone());
				Certifications::<Test>::insert(midds_id_2, certif_state.clone());
				Certifications::<Test>::insert(midds_id_3, certif_state_2.clone());
				// Add the mock MIDDS ID to the storage queue.
				PreCertificationsQueue::<Test>::put(test_queue.clone());

				let now = <Timestamp as UnixTime>::now();
				assert_eq!(now, Duration::from_millis(100));

				// No timestamp change, nothing should be removed from the queue.
				Certification::on_idle(current_block, 1000000.into());
				assert_eq!(test_queue, PreCertificationsQueue::<Test>::get());

				// Must be greater than expected_wait_time to trigger the upgrade logic so we add 1.
				Timestamp::set_timestamp(base_timestamp + expected_wait_time_as_u64 + 1u64);

				// Timestamp changed to wait time period for upgrade to Certified.
				// Queue should now be purged from the mock MIDDS ID 1 and 2.
				// Certif status 1 and 2 should be upgraded to CERTIFIED.
				Certification::on_idle(current_block, 1000000.into());
				test_queue.pop_front();
				test_queue.pop_front();
				certif_state.status = CertifStatus::Certif(());
				assert_eq!(test_queue, PreCertificationsQueue::<Test>::get());
				assert_eq!(Some(certif_state.clone()), Certifications::<Test>::get(midds_id));
				assert_eq!(Some(certif_state), Certifications::<Test>::get(midds_id_2));
				assert_eq!(Some(certif_state_2), Certifications::<Test>::get(midds_id_3));

				// TODO: weights test
				// assert_eq!(remaining_weights, 999999.into());
			})
		}
	}
}
