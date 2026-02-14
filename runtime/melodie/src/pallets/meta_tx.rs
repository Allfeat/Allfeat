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

use sp_runtime::traits::Verify;

use crate::*;

impl pallet_meta_tx::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Extension = MetaTxExtension;
    type WeightInfo = weights::meta_tx::AllfeatWeight<Runtime>;
}

impl pallet_verify_signature::Config for Runtime {
    type Signature = Signature;
    type AccountIdentifier = <Signature as Verify>::Signer;
    type WeightInfo = weights::verify_signature::AllfeatWeight<Runtime>;
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = ();
}
