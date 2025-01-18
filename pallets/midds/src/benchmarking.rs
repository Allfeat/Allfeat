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

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as MiddsPallet;

use frame_benchmarking::v2::*;
use frame_support::{sp_runtime::traits::Bounded, traits::fungible::Mutate};
use frame_system::RawOrigin;

fn assert_last_event<T: Config<I>, I: 'static>(generic_event: <T as Config<I>>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

#[instance_benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn register() {
		let provider = whitelisted_caller();
		let midds = T::MIDDS::create_midds();

		let _ = T::Currency::set_balance(&provider, init_bal::<T, I>());

		#[extrinsic_call]
		_(RawOrigin::Signed(provider), Box::new(midds.clone()));

		assert!(PendingMidds::<T, I>::get(midds.hash()).is_some())
	}

	#[benchmark]
	fn update_field() -> Result<(), BenchmarkError> {
		let provider = whitelisted_caller();
		let mut midds = T::MIDDS::create_midds();

		let _ = T::Currency::set_balance(&provider, init_bal::<T, I>());

		MiddsPallet::<T, I>::register(
			RawOrigin::Signed(provider.clone()).into(),
			Box::new(midds.clone()),
		)?;

		let original_hash = midds.hash();
		midds.update_field(<T::MIDDS as Midds>::EditableFields::default())?;
		let expected_new_hash = midds.hash();

		#[extrinsic_call]
		_(
			RawOrigin::Signed(provider),
			original_hash,
			<T::MIDDS as Midds>::EditableFields::default(),
		);

		assert_last_event::<T, I>(Event::MIDDSUpdated { hash_id: expected_new_hash }.into());
		Ok(())
	}

	#[benchmark]
	fn unregister() -> Result<(), BenchmarkError> {
		let provider = whitelisted_caller();
		let midds = T::MIDDS::create_midds();

		let _ = T::Currency::set_balance(&provider, init_bal::<T, I>());

		MiddsPallet::<T, I>::register(
			RawOrigin::Signed(provider.clone()).into(),
			Box::new(midds.clone()),
		)?;

		#[extrinsic_call]
		_(RawOrigin::Signed(provider), midds.hash());

		assert_last_event::<T, I>(Event::MIDDSUnregistered { hash_id: midds.hash() }.into());
		Ok(())
	}

	fn init_bal<T: Config<I>, I: 'static>() -> BalanceOf<T, I> {
		BalanceOf::<T, I>::max_value() / 10u32.into()
	}

	impl_benchmark_test_suite!(MiddsPallet, crate::mock::new_test_ext(), crate::mock::Test);
}
