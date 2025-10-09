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

use crate::*;
use alloc::vec::Vec;
use frame_support::{
    parameter_types,
    sp_runtime::{impl_opaque_keys, traits::OpaqueKeys},
};
use pallet_session::PeriodicSessions;
use shared_runtime::weights;
use sp_runtime::traits::ConvertInto;

impl_opaque_keys! {
    pub struct SessionKeys {
        pub grandpa: Grandpa,
        pub aura: Aura,
        pub im_online: ImOnline,
    }
}

parameter_types! {
    pub const SessionPeriod: BlockNumber = 1800; // 6 hours for 12sec block
    pub const SessionOffset: BlockNumber = 0;
}

impl pallet_session::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type KeyDeposit = ();
    type ValidatorId = <Self as frame_system::Config>::AccountId;
    type ValidatorIdOf = ConvertInto;
    type ShouldEndSession = PeriodicSessions<SessionPeriod, SessionOffset>;
    type NextSessionRotation = PeriodicSessions<SessionPeriod, SessionOffset>;
    type SessionManager = pallet_session::historical::NoteHistoricalRoot<Self, Validators>;
    type SessionHandler = <SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
    type Keys = SessionKeys;
    type DisablingStrategy = pallet_session::disabling::UpToLimitWithReEnablingDisablingStrategy;
    type WeightInfo = weights::session::AllfeatWeight<Runtime>;
}

impl pallet_session::historical::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type FullIdentification = Self::ValidatorId;
    type FullIdentificationOf = Self::ValidatorIdOf;
}
