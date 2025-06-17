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
use midds::pallet_prelude::MusicalWork;
use pallet_midds::MiddsOf;
use sp_core::Get;
use sp_runtime::Weight;

mod v1 {
    use midds::types::{
        musical_work::{
            Iswc, MusicalWorkBpm, MusicalWorkCreationYear, MusicalWorkParticipants,
            MusicalWorkTitle, MusicalWorkType,
        },
        utils::{Key, Language},
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
    pub struct MusicalWork {
        /// The ISWC (International Standard Musical Work Code) uniquely identifying the work.
        pub iswc: Iswc,

        /// The title of the musical work.
        pub title: MusicalWorkTitle,

        /// The year the work was created (4-digit Gregorian year).
        pub creation_year: MusicalWorkCreationYear,

        /// Indicates whether the work is instrumental (i.e., without lyrics).
        pub instrumental: bool,

        /// The optional language of the lyrics (if any).
        pub language: Option<Language>,

        /// Optional tempo in beats per minute (BPM).
        pub bpm: Option<MusicalWorkBpm>,

        /// Optional musical key of the work (e.g., C, G#, etc.).
        pub key: Option<Key>,

        /// Type of the musical work (original, medley, mashup, or adaptation).
        pub work_type: MusicalWorkType,

        /// List of contributors to the work, along with their roles.
        pub participants: MusicalWorkParticipants,
    }
}

impl From<v1::MusicalWork> for MusicalWork {
    fn from(value: v1::MusicalWork) -> Self {
        Self {
            iswc: value.iswc,
            title: value.title,
            creation_year: Some(value.creation_year),
            instrumental: Some(value.instrumental),
            language: value.language,
            bpm: value.bpm,
            key: value.key,
            work_type: Some(value.work_type),
            participants: value.participants,
        }
    }
}

pub type MusicalWorkV1ToV2<T> = VersionedMigration<
    1,
    2,
    InnerMusicalWorkMigrationV1ToV2<T>,
    pallet_midds::Pallet<T, crate::midds::MusicalWorks>,
    <T as frame_system::Config>::DbWeight,
>;

pub struct InnerMusicalWorkMigrationV1ToV2<T: pallet_midds::Config<crate::midds::MusicalWorks>>(
    core::marker::PhantomData<T>,
);

impl<T: pallet_midds::Config<crate::midds::MusicalWorks>> UncheckedOnRuntimeUpgrade
    for InnerMusicalWorkMigrationV1ToV2<T>
where
    <T as pallet_midds::Config<crate::midds::MusicalWorks>>::MIDDS: From<v1::MusicalWork>,
{
    fn on_runtime_upgrade() -> Weight {
        let mut count = 0;

        MiddsOf::<T, crate::midds::MusicalWorks>::translate::<v1::MusicalWork, _>(|_k, old| {
            count += 1;
            Some(old.into())
        });

        log::info!(
            target: LOG_TARGET,
            "Storage migration v2 for musical_works finished.",
        );

        // calculate and return migration weights
        T::DbWeight::get().reads_writes(count as u64 + 1, count as u64 + 1)
    }
}
