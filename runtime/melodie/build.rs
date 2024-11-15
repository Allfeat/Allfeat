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

#[cfg(all(feature = "std", not(feature = "metadata-hash")))]
fn main() {
	polkadot_sdk::substrate_wasm_builder::WasmBuilder::build_using_defaults()
}

#[cfg(all(feature = "std", feature = "metadata-hash"))]
fn main() {
	polkadot_sdk::substrate_wasm_builder::WasmBuilder::init_with_defaults()
		.enable_metadata_hash("MEL", 18)
		.build()
}

#[cfg(not(feature = "std"))]
fn main() {}