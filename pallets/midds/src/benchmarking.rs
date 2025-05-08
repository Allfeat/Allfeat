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
use frame_support::traits::Get;
use frame_support::{sp_runtime::traits::Bounded, traits::fungible::Mutate};
use frame_system::RawOrigin;

use midds::pallet_prelude::BenchmarkHelperT;

fn assert_last_event<T: Config<I>, I: 'static>(generic_event: <T as Config<I>>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

#[instance_benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn register(
		x: Linear<0, <<T::MIDDS as super::Midds>::BenchmarkHelper as BenchmarkHelperT<T::MIDDS>>::FIELD_MAX_SIZE>,
	) {
		let provider = whitelisted_caller();
		let midds = <T::MIDDS as super::Midds>::BenchmarkHelper::build_mock(x);
		/// TODO: size
		let _ = T::Currency::set_balance(&provider, init_bal::<T, I>());

		#[extrinsic_call]
		_(RawOrigin::Signed(provider), Box::new(midds.clone()));

		assert!(MiddsDb::<T, I>::get(0).is_some())
	}

	#[benchmark]
	fn unregister(
		x: Linear<0, <<T::MIDDS as super::Midds>::BenchmarkHelper as BenchmarkHelperT<T::MIDDS>>::FIELD_MAX_SIZE>,
	) -> Result<(), BenchmarkError> {
		let provider = whitelisted_caller();
		let midds = <T::MIDDS as super::Midds>::BenchmarkHelper::build_mock(x);

		let _ = T::Currency::set_balance(&provider, init_bal::<T, I>());

		MiddsPallet::<T, I>::register(
			RawOrigin::Signed(provider.clone()).into(),
			Box::new(midds.clone()),
		)?;

		#[extrinsic_call]
		_(RawOrigin::Signed(provider), 0);

		assert_last_event::<T, I>(Event::MIDDSUnregistered { midds_id: 0 }.into());
		Ok(())
	}

	fn init_bal<T: Config<I>, I: 'static>() -> BalanceOf<T, I> {
		BalanceOf::<T, I>::max_value() / 10u32.into()
	}

	impl_benchmark_test_suite!(MiddsPallet, crate::mock::new_test_ext(), crate::mock::Test);
}
