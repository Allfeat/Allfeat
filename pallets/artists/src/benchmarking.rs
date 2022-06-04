//! Artists pallet benchmarking.
#![allow(unused_imports)]
#![allow(dead_code)]

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{
	account, benchmarks_instance_pallet, whitelist_account, whitelisted_caller,
};
use frame_support::{
	traits::{EnsureOrigin, Get},
};
use frame_system::RawOrigin as SystemOrigin;
use sp_std::prelude::*;
use sp_runtime::traits::Bounded;

use crate::Pallet as Artists;

type BalanceOf<T, I> =
<<T as Config<I>>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

fn assert_last_event<T: Config<I>, I: 'static>(generic_event: <T as Config<I>>::Event) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

fn assert_event<T: Config<I>, I: 'static>(generic_event: <T as Config<I>>::Event) {
	frame_system::Pallet::<T>::assert_has_event(generic_event.into());
}

benchmarks_instance_pallet! {
	force_create {
		let a in 1 .. T::StringLimit::get();
		let b in 1 .. T::StringLimit::get();
		let c in 1 .. 4;

		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());

		let artist_name = vec![0u8; a as usize];
		let artist_asset_name = vec![0u8; b as usize];
		let artist_symbol = vec![0u8; c as usize];

		T::Currency::make_free_balance_be(&caller, BalanceOf::<T, I>::max_value());
	}: _(SystemOrigin::Root, Default::default(), caller_lookup, artist_name.clone(), artist_asset_name, artist_symbol)
	verify {
		assert_last_event::<T, I>(Event::ArtistCreated { artist_id: Default::default(), block: <frame_system::Pallet<T>>::block_number(), name: artist_name.try_into().unwrap() }.into());
	}

	impl_benchmark_test_suite!(Artists, crate::mock::new_test_ext(false), crate::mock::Test)
}
