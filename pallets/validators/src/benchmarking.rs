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

use super::*;
use frame_benchmarking::{v1::account, v2::*};
use frame_support::traits::Get;
use frame_system::RawOrigin;

const SEED: u32 = 0;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn add_validator() {
        /// Need to empty the mock initialized set
        Validators::<T>::kill();

        let new: T::ValidatorId = account("validator", 0, SEED);

        // Pre-fill validators up to N - 1 if needed
        for i in 1..T::MaxValidators::get() {
            let val = account("validator", i, SEED);
            Validators::<T>::mutate(|vals| {
                if !vals.contains(&val) {
                    vals.try_push(val).unwrap();
                }
            });
        }

        #[extrinsic_call]
        _(RawOrigin::Root, new.clone());

        assert!(Validators::<T>::get().contains(&new));
    }

    #[benchmark]
    fn remove_validator() {
        Validators::<T>::kill();
        let existing: T::ValidatorId = account("validator", 42, SEED);

        Validators::<T>::mutate(|vals| {
            if !vals.contains(&existing.clone()) {
                vals.try_push(existing.clone()).unwrap();
            }
        });

        #[extrinsic_call]
        _(RawOrigin::Root, existing.clone());

        assert!(!Validators::<T>::get().contains(&existing));
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
