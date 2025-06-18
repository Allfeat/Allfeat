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

use frame_support::{
    LOG_TARGET, migrations::VersionedMigration, traits::UncheckedOnRuntimeUpgrade,
};
use midds::pallet_prelude::PartyIdentifier;
use pallet_midds::MiddsOf;
use sp_core::Get;
use sp_runtime::Weight;

mod v2 {
    use midds::{
        pallet_prelude::PartyType,
        types::party_identifier::{Ipi, Isni},
    };
    use parity_scale_codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
    use scale_info::TypeInfo;
    use sp_core::RuntimeDebug;

    #[derive(
        Clone,
        Eq,
        PartialEq,
        Encode,
        Decode,
        DecodeWithMemTracking,
        TypeInfo,
        MaxEncodedLen,
        RuntimeDebug,
    )]
    pub struct PartyIdentifier {
        /// ISNI identifier (max 16 characters).
        pub isni: Isni,
        /// IPI identifier (11-digit u64).
        pub ipi: Ipi,
        /// Variant defining if the party is a `Person` or an `Entity` with data.
        pub party_type: PartyType,
    }
}

impl From<v2::PartyIdentifier> for PartyIdentifier {
    fn from(value: v2::PartyIdentifier) -> Self {
        Self {
            ipi: Some(value.ipi),
            isni: Some(value.isni),
            party_type: value.party_type,
        }
    }
}

pub type PartyIdentifierV2ToV3<T> = VersionedMigration<
    2,
    3,
    InnerPartyIdentifierMigrationV2ToV3<T>,
    pallet_midds::Pallet<T, crate::midds::PartyIdentifiers>,
    <T as frame_system::Config>::DbWeight,
>;

pub struct InnerPartyIdentifierMigrationV2ToV3<
    T: pallet_midds::Config<crate::midds::PartyIdentifiers>,
>(core::marker::PhantomData<T>);

impl<T: pallet_midds::Config<crate::midds::PartyIdentifiers>> UncheckedOnRuntimeUpgrade
    for InnerPartyIdentifierMigrationV2ToV3<T>
where
    <T as pallet_midds::Config<crate::midds::PartyIdentifiers>>::MIDDS: From<v2::PartyIdentifier>,
{
    fn on_runtime_upgrade() -> Weight {
        let mut count = 0;

        MiddsOf::<T, crate::midds::PartyIdentifiers>::translate::<v2::PartyIdentifier, _>(
            |_k, old| {
                count += 1;
                Some(old.into())
            },
        );

        log::info!(
            target: LOG_TARGET,
            "Storage migration v3 for party_identifiers finished.",
        );

        // calculate and return migration weights
        T::DbWeight::get().reads_writes(count as u64 + 1, count as u64 + 1)
    }
}
