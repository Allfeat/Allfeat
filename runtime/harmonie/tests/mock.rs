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

pub use harmonie_runtime::*;

// substrate
use sp_io::TestExternalities;

#[derive(Clone, Default)]
pub struct ExtBuilder {
	balances: Vec<(AccountId, Balance)>,
}

impl ExtBuilder {
	pub fn with_balances(&mut self, balances: Vec<(AccountId, Balance)>) -> &mut Self {
		self.balances = balances;

		self
	}

	pub fn build(&mut self) -> TestExternalities {
		let mut t = <frame_system::GenesisConfig<Runtime>>::default().build_storage().unwrap();

		pallet_balances::GenesisConfig::<Runtime> { balances: self.balances.clone() }
			.assimilate_storage(&mut t)
			.unwrap();

		let mut ext = TestExternalities::new(t);

		ext.execute_with(|| System::set_block_number(1));

		ext
	}
}
