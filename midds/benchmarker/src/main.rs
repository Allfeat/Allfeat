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

use melodie_runtime::Balance;
use melodie_runtime::{musical_works, party_identifier, release, track};
use midds::{pallet_prelude::*, Midds};
use shared_runtime::currency::AFT;

fn main() {
	benchmark::<PartyIdentifier>(Some(EconomicInfo {
		unit: AFT,
		byte_cost: party_identifier::ByteDepositCost::get(),
	}));
	benchmark::<MusicalWork>(Some(EconomicInfo {
		unit: AFT,
		byte_cost: musical_works::ByteDepositCost::get(),
	}));
	benchmark::<Track>(Some(EconomicInfo { unit: AFT, byte_cost: track::ByteDepositCost::get() }));
	benchmark::<Release>(Some(EconomicInfo {
		unit: AFT,
		byte_cost: release::ByteDepositCost::get(),
	}));
}

fn benchmark<T: Midds>(economic_info: Option<EconomicInfo>) {
	let max_size = T::max_encoded_len();
	let mock_size = T::BenchmarkHelper::build_mock().encoded_size();

	println!("========== {} ==========", T::NAME);

	println!("⛓️ Size Informations:");
	println!("Max size: {} bytes", max_size);
	println!("Mock size: {} bytes\n", mock_size);

	if let Some(x) = economic_info {
		println!("💸 Economic Informations:");
		println!("Max cost: {} AFT", (max_size as u128 * x.byte_cost) as f64 / x.unit as f64);

		println!("Mock cost: {} AFT", (mock_size as u128 * x.byte_cost) as f64 / x.unit as f64);
		println!();
	} else {
		println!()
	}
}

struct EconomicInfo {
	pub unit: Balance,
	pub byte_cost: Balance,
}
