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
use midds::{
    pallet_prelude::{Entity, PartyIdentifier, PartyType},
    types::party_identifier::EntityType,
};
use pallet_midds::MiddsOf;
use sp_core::Get;
use sp_runtime::Weight;

mod legacy {
    use midds::{
        pallet_prelude::Artist,
        types::party_identifier::{EntityName, Ipi, Isni},
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
        /// ISNI identifier (max 16 characters). Optional but either `isni` or `ipi`
        /// must be provided.
        pub isni: Option<Isni>,
        /// IPI identifier (11-digit u64). Optional but either `isni` or `ipi` must
        /// be provided.
        pub ipi: Option<Ipi>,
        /// Variant defining if the party is a `Artist` or an `Entity` with data.
        pub party_type: PartyType,
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
        Artist(Artist),
        Entity(Entity),
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
    pub struct Entity {
        /// Entity Name.
        pub name: EntityName,
        /// The role played by the organization (e.g., publisher, producer).
        pub entity_type: EntityType,
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
    pub enum EntityType {
        Publisher,
        Producer,
        DistribAggr,
    }
}

impl From<legacy::EntityType> for EntityType {
    fn from(value: legacy::EntityType) -> Self {
        match value {
            legacy::EntityType::Publisher => Self::Publisher,
            legacy::EntityType::Producer => Self::Producer,
            legacy::EntityType::DistribAggr => Self::Publisher,
        }
    }
}

impl From<legacy::Entity> for Entity {
    fn from(value: legacy::Entity) -> Self {
        Self {
            name: value.name,
            entity_type: value.entity_type.into(),
        }
    }
}

impl From<legacy::PartyType> for PartyType {
    fn from(value: legacy::PartyType) -> Self {
        match value {
            legacy::PartyType::Artist(x) => Self::Artist(x),
            legacy::PartyType::Entity(x) => Self::Entity(x.into()),
        }
    }
}

impl From<legacy::PartyIdentifier> for PartyIdentifier {
    fn from(value: legacy::PartyIdentifier) -> Self {
        Self {
            isni: value.isni,
            ipi: value.ipi,
            party_type: value.party_type.into(),
        }
    }
}

pub type PartyIdentifierV3ToV4<T> = VersionedMigration<
    3,
    4,
    InnerPartyIdentifierV3ToV4<T>,
    pallet_midds::Pallet<T, crate::midds::PartyIdentifiers>,
    <T as frame_system::Config>::DbWeight,
>;

pub struct InnerPartyIdentifierV3ToV4<T: pallet_midds::Config<crate::midds::PartyIdentifiers>>(
    core::marker::PhantomData<T>,
);

impl<T: pallet_midds::Config<crate::midds::PartyIdentifiers>> UncheckedOnRuntimeUpgrade
    for InnerPartyIdentifierV3ToV4<T>
where
    <T as pallet_midds::Config<crate::midds::PartyIdentifiers>>::MIDDS:
        From<legacy::PartyIdentifier>,
{
    fn on_runtime_upgrade() -> Weight {
        let mut count = 0;

        MiddsOf::<T, crate::midds::PartyIdentifiers>::translate::<legacy::PartyIdentifier, _>(
            |_k, old| {
                count += 1;
                Some(old.into())
            },
        );

        log::info!(
            target: LOG_TARGET,
            "Storage migration v4 for party identifiers finished.",
        );

        // calculate and return migration weights
        T::DbWeight::get().reads_writes(count as u64 + 1, count as u64 + 1)
    }
}
