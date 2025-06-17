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
use midds::pallet_prelude::{PartyIdentifier, PartyType, Person};
use pallet_midds::MiddsOf;
use sp_core::Get;
use sp_runtime::Weight;

mod v1 {
    use midds::{
        pallet_prelude::Entity,
        types::party_identifier::{
            Ipi, Isni, PersonAliases, PersonFullName, PersonGender, PersonType,
        },
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

    #[derive(
        Clone,
        PartialEq,
        Eq,
        Encode,
        Decode,
        DecodeWithMemTracking,
        MaxEncodedLen,
        RuntimeDebug,
        TypeInfo,
    )]
    pub struct Person {
        /// Legal name of the person.
        pub full_name: PersonFullName,
        /// Alternative names/stage names.
        pub aliases: PersonAliases,
        /// Indicates if this is a solo artist or a group.
        pub person_type: PersonType,
        /// Declared gender identity.
        pub genre: PersonGender,
    }

    #[derive(
        RuntimeDebug,
        Clone,
        PartialEq,
        Eq,
        Encode,
        Decode,
        DecodeWithMemTracking,
        MaxEncodedLen,
        TypeInfo,
    )]
    pub enum PartyType {
        Person(Person),
        Entity(Entity),
    }
}

impl From<v1::Person> for Person {
    fn from(value: v1::Person) -> Self {
        Self {
            full_name: value.full_name,
            aliases: value.aliases,
            person_type: value.person_type,
            genre: Some(value.genre),
        }
    }
}

impl From<v1::PartyType> for PartyType {
    fn from(value: v1::PartyType) -> Self {
        match value {
            v1::PartyType::Person(x) => PartyType::Person(x.into()),
            v1::PartyType::Entity(x) => PartyType::Entity(x),
        }
    }
}

impl From<v1::PartyIdentifier> for PartyIdentifier {
    fn from(value: v1::PartyIdentifier) -> Self {
        Self {
            ipi: value.ipi,
            isni: value.isni,
            party_type: value.party_type.into(),
        }
    }
}

pub type PartyIdentifierV1ToV2<T> = VersionedMigration<
    1,
    2,
    InnerPartyIdentifierMigrationV1ToV2<T>,
    pallet_midds::Pallet<T, crate::midds::PartyIdentifiers>,
    <T as frame_system::Config>::DbWeight,
>;

pub struct InnerPartyIdentifierMigrationV1ToV2<
    T: pallet_midds::Config<crate::midds::PartyIdentifiers>,
>(core::marker::PhantomData<T>);

impl<T: pallet_midds::Config<crate::midds::PartyIdentifiers>> UncheckedOnRuntimeUpgrade
    for InnerPartyIdentifierMigrationV1ToV2<T>
where
    <T as pallet_midds::Config<crate::midds::PartyIdentifiers>>::MIDDS: From<v1::PartyIdentifier>,
{
    fn on_runtime_upgrade() -> Weight {
        let mut count = 0;

        MiddsOf::<T, crate::midds::PartyIdentifiers>::translate::<v1::PartyIdentifier, _>(
            |_k, old| {
                count += 1;
                Some(old.into())
            },
        );

        log::info!(
            target: LOG_TARGET,
            "Storage migration v2 for party_identifiers finished.",
        );

        // calculate and return migration weights
        T::DbWeight::get().reads_writes(count as u64 + 1, count as u64 + 1)
    }
}
