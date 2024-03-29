
//! Autogenerated weights for pallet_artists
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2024-01-24, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `harmonie-node-01`, CPU: `Intel(R) Xeon(R) Platinum 8280 CPU @ 2.70GHz`
//! EXECUTION: , WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// target/release/allfeat
// benchmark
// pallet
// --chain=dev
// --steps=50
// --repeat=20
// --pallet=pallet-artists
// --extrinsic=*
// --wasm-execution=compiled
// --heap-pages=4096
// --output=runtime/harmonie/src/weights/artists.rs
// --template=./.maintain/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for pallet_artists.
pub trait WeightInfo {
	fn register(n: u32, g: u32, a: u32, ) -> Weight;
	fn force_unregister(n: u32, g: u32, a: u32, ) -> Weight;
	fn unregister(n: u32, g: u32, a: u32, ) -> Weight;
	fn update_alias(n: u32, x: u32, ) -> Weight;
	fn update_add_genres(n: u32, ) -> Weight;
	fn update_remove_genres(n: u32, ) -> Weight;
	fn update_clear_genres(n: u32, ) -> Weight;
	fn update_description() -> Weight;
	fn update_add_assets(n: u32, ) -> Weight;
	fn update_remove_assets(n: u32, ) -> Weight;
	fn update_clear_assets(n: u32, ) -> Weight;
}

/// Weights for pallet_artists using the Allfeat node and recommended hardware.
pub struct AllfeatWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_artists::weights::WeightInfo for AllfeatWeight<T> {
	/// Storage: `Artists::ArtistOf` (r:1 w:1)
	/// Proof: `Artists::ArtistOf` (`max_values`: None, `max_size`: Some(12659), added: 15134, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Holds` (r:1 w:1)
	/// Proof: `Balances::Holds` (`max_values`: None, `max_size`: Some(937), added: 3412, mode: `MaxEncodedLen`)
	/// The range of component `n` is `[1, 128]`.
	/// The range of component `g` is `[0, 5]`.
	/// The range of component `a` is `[0, 64]`.
	fn register(n: u32, g: u32, a: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `76`
		//  Estimated: `16124`
		// Minimum execution time: 225_483_000 picoseconds.
		Weight::from_parts(224_202_449, 16124)
			// Standard Error: 34_797
			.saturating_add(Weight::from_parts(27_419, 0).saturating_mul(n.into()))
			// Standard Error: 795_289
			.saturating_add(Weight::from_parts(2_717_801, 0).saturating_mul(g.into()))
			// Standard Error: 68_728
			.saturating_add(Weight::from_parts(45_314_688, 0).saturating_mul(a.into()))
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: `Balances::Holds` (r:1 w:1)
	/// Proof: `Balances::Holds` (`max_values`: None, `max_size`: Some(937), added: 3412, mode: `MaxEncodedLen`)
	/// Storage: `Artists::ArtistOf` (r:0 w:1)
	/// Proof: `Artists::ArtistOf` (`max_values`: None, `max_size`: Some(12659), added: 15134, mode: `MaxEncodedLen`)
	/// The range of component `n` is `[1, 128]`.
	/// The range of component `g` is `[0, 5]`.
	/// The range of component `a` is `[0, 64]`.
	fn force_unregister(_n: u32, _g: u32, a: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `118`
		//  Estimated: `4402`
		// Minimum execution time: 130_955_000 picoseconds.
		Weight::from_parts(149_126_719, 4402)
			// Standard Error: 5_635
			.saturating_add(Weight::from_parts(17_363, 0).saturating_mul(a.into()))
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: `Artists::ArtistOf` (r:1 w:1)
	/// Proof: `Artists::ArtistOf` (`max_values`: None, `max_size`: Some(12659), added: 15134, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Holds` (r:1 w:1)
	/// Proof: `Balances::Holds` (`max_values`: None, `max_size`: Some(937), added: 3412, mode: `MaxEncodedLen`)
	/// The range of component `n` is `[1, 128]`.
	/// The range of component `g` is `[0, 5]`.
	/// The range of component `a` is `[0, 64]`.
	fn unregister(_n: u32, g: u32, a: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `314 + a * (32 ±0) + g * (3 ±0) + n * (2 ±0)`
		//  Estimated: `16124`
		// Minimum execution time: 194_639_000 picoseconds.
		Weight::from_parts(214_525_593, 16124)
			// Standard Error: 74_497
			.saturating_add(Weight::from_parts(69_914, 0).saturating_mul(g.into()))
			// Standard Error: 6_438
			.saturating_add(Weight::from_parts(72_709, 0).saturating_mul(a.into()))
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: `Artists::ArtistOf` (r:1 w:1)
	/// Proof: `Artists::ArtistOf` (`max_values`: None, `max_size`: Some(12659), added: 15134, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Holds` (r:1 w:0)
	/// Proof: `Balances::Holds` (`max_values`: None, `max_size`: Some(937), added: 3412, mode: `MaxEncodedLen`)
	/// The range of component `n` is `[1, 128]`.
	/// The range of component `x` is `[1, 128]`.
	fn update_alias(_n: u32, x: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `295 + n * (2 ±0)`
		//  Estimated: `16124`
		// Minimum execution time: 24_913_000 picoseconds.
		Weight::from_parts(78_092_942, 16124)
			// Standard Error: 6_537
			.saturating_add(Weight::from_parts(22_103, 0).saturating_mul(x.into()))
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: `Artists::ArtistOf` (r:1 w:1)
	/// Proof: `Artists::ArtistOf` (`max_values`: None, `max_size`: Some(12659), added: 15134, mode: `MaxEncodedLen`)
	/// The range of component `n` is `[0, 4]`.
	fn update_add_genres(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `198 + n * (3 ±0)`
		//  Estimated: `16124`
		// Minimum execution time: 20_162_000 picoseconds.
		Weight::from_parts(22_125_236, 16124)
			// Standard Error: 20_662
			.saturating_add(Weight::from_parts(122_102, 0).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `Artists::ArtistOf` (r:1 w:1)
	/// Proof: `Artists::ArtistOf` (`max_values`: None, `max_size`: Some(12659), added: 15134, mode: `MaxEncodedLen`)
	/// The range of component `n` is `[1, 5]`.
	fn update_remove_genres(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `198 + n * (3 ±0)`
		//  Estimated: `16124`
		// Minimum execution time: 18_736_000 picoseconds.
		Weight::from_parts(20_999_332, 16124)
			// Standard Error: 12_641
			.saturating_add(Weight::from_parts(22_922, 0).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `Artists::ArtistOf` (r:1 w:1)
	/// Proof: `Artists::ArtistOf` (`max_values`: None, `max_size`: Some(12659), added: 15134, mode: `MaxEncodedLen`)
	/// The range of component `n` is `[0, 5]`.
	fn update_clear_genres(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `198 + n * (3 ±0)`
		//  Estimated: `16124`
		// Minimum execution time: 18_117_000 picoseconds.
		Weight::from_parts(20_363_331, 16124)
			// Standard Error: 8_609
			.saturating_add(Weight::from_parts(147_204, 0).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `Artists::ArtistOf` (r:1 w:1)
	/// Proof: `Artists::ArtistOf` (`max_values`: None, `max_size`: Some(12659), added: 15134, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Holds` (r:1 w:1)
	/// Proof: `Balances::Holds` (`max_values`: None, `max_size`: Some(937), added: 3412, mode: `MaxEncodedLen`)
	fn update_description() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `298`
		//  Estimated: `16124`
		// Minimum execution time: 107_144_000 picoseconds.
		Weight::from_parts(108_549_000, 16124)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: `Artists::ArtistOf` (r:1 w:1)
	/// Proof: `Artists::ArtistOf` (`max_values`: None, `max_size`: Some(12659), added: 15134, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Holds` (r:1 w:1)
	/// Proof: `Balances::Holds` (`max_values`: None, `max_size`: Some(937), added: 3412, mode: `MaxEncodedLen`)
	/// The range of component `n` is `[0, 63]`.
	fn update_add_assets(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `316 + n * (32 ±0)`
		//  Estimated: `16124`
		// Minimum execution time: 70_223_000 picoseconds.
		Weight::from_parts(76_592_424, 16124)
			// Standard Error: 3_652
			.saturating_add(Weight::from_parts(215_044, 0).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: `Artists::ArtistOf` (r:1 w:1)
	/// Proof: `Artists::ArtistOf` (`max_values`: None, `max_size`: Some(12659), added: 15134, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Holds` (r:1 w:1)
	/// Proof: `Balances::Holds` (`max_values`: None, `max_size`: Some(937), added: 3412, mode: `MaxEncodedLen`)
	/// The range of component `n` is `[1, 64]`.
	fn update_remove_assets(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `316 + n * (32 ±0)`
		//  Estimated: `16124`
		// Minimum execution time: 60_015_000 picoseconds.
		Weight::from_parts(65_336_473, 16124)
			// Standard Error: 3_196
			.saturating_add(Weight::from_parts(201_734, 0).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: `Artists::ArtistOf` (r:1 w:1)
	/// Proof: `Artists::ArtistOf` (`max_values`: None, `max_size`: Some(12659), added: 15134, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Holds` (r:1 w:1)
	/// Proof: `Balances::Holds` (`max_values`: None, `max_size`: Some(937), added: 3412, mode: `MaxEncodedLen`)
	/// The range of component `n` is `[0, 64]`.
	fn update_clear_assets(n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `316 + n * (32 ±0)`
		//  Estimated: `16124`
		// Minimum execution time: 48_725_000 picoseconds.
		Weight::from_parts(64_654_478, 16124)
			// Standard Error: 5_618
			.saturating_add(Weight::from_parts(73_794, 0).saturating_mul(n.into()))
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
}
