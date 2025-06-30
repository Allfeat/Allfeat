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
    pallet_prelude::MusicalWork,
    types::{musical_work::MusicalWorkType, utils::Key},
};
use pallet_midds::MiddsOf;
use sp_core::Get;
use sp_runtime::Weight;

mod legacy {
    use midds::types::{
        musical_work::{
            DerivedWorks, Iswc, MusicalWorkBpm, MusicalWorkCreationYear, MusicalWorkParticipants,
            MusicalWorkTitle,
        },
        utils::Language,
    };
    use parity_scale_codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
    use scale_info::TypeInfo;
    use sp_core::RuntimeDebug;

    /// Legacy type
    #[repr(u8)]
    #[derive(
        RuntimeDebug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        Encode,
        Decode,
        DecodeWithMemTracking,
        MaxEncodedLen,
        TypeInfo,
    )]
    pub enum Key {
        C = 0,
        Cs = 1, // C♯ / D♭
        D = 2,
        Ds = 3, // D♯ / E♭
        E = 4,
        F = 5,
        Fs = 6, // F♯ / G♭
        G = 7,
        Gs = 8, // G♯ / A♭
        A = 9,
        As = 10, // A♯ / B♭
        B = 11,
    }

    /// Legacy type
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
    pub enum MusicalWorkType {
        /// A standalone, original composition.
        Original,
        /// A combination of multiple existing works (referenced via their IDs).
        Medley(DerivedWorks),
        /// A mixed version using components of existing works.
        Mashup(DerivedWorks),
        /// A modified version of existing work(s), with optional lyrics or melody changes.
        Adaptation(AdapationWork),
    }

    /// Legacy type
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
    pub struct AdapationWork {
        pub references: DerivedWorks,
        pub lyrics_adaptation: bool,
        pub song_adaptation: bool,
    }

    /// Legacy type
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
        pub creation_year: Option<MusicalWorkCreationYear>,

        /// Indicates whether the work is instrumental (i.e., without lyrics).
        pub instrumental: Option<bool>,

        /// The optional language of the lyrics (if any).
        pub language: Option<Language>,

        /// Optional tempo in beats per minute (BPM).
        pub bpm: Option<MusicalWorkBpm>,

        /// Optional musical key of the work (e.g., C, G#, etc.).
        pub key: Option<Key>,

        /// Type of the musical work (original, medley, mashup, or adaptation).
        pub work_type: Option<MusicalWorkType>,

        /// List of contributors to the work, along with their roles.
        pub participants: MusicalWorkParticipants,
    }
}

impl From<legacy::MusicalWorkType> for MusicalWorkType {
    fn from(value: legacy::MusicalWorkType) -> Self {
        match value {
            legacy::MusicalWorkType::Original => Self::Original,
            legacy::MusicalWorkType::Medley(x) => Self::Medley(x),
            legacy::MusicalWorkType::Mashup(x) => Self::Mashup(x),
            legacy::MusicalWorkType::Adaptation(x) => {
                let first_id = x.references.first().unwrap_or(&0u64);
                Self::Adaptation(*first_id)
            }
        }
    }
}

impl From<legacy::Key> for Key {
    fn from(value: legacy::Key) -> Self {
        match value {
            legacy::Key::C => Key::C,
            legacy::Key::Cs => Key::Cs, // C♯
            legacy::Key::D => Key::D,
            legacy::Key::Ds => Key::Ds, // D♯
            legacy::Key::E => Key::E,
            legacy::Key::F => Key::F,
            legacy::Key::Fs => Key::Fs, // F♯
            legacy::Key::G => Key::G,
            legacy::Key::Gs => Key::Gs, // G♯
            legacy::Key::A => Key::A,
            legacy::Key::As => Key::As, // A♯
            legacy::Key::B => Key::B,
        }
    }
}

impl From<legacy::MusicalWork> for MusicalWork {
    fn from(value: legacy::MusicalWork) -> Self {
        Self {
            iswc: value.iswc,
            title: value.title,
            creation_year: value.creation_year,
            instrumental: value.instrumental,
            language: value.language,
            bpm: value.bpm,
            key: value.key.map(Key::from),
            work_type: value.work_type.map(MusicalWorkType::from),
            participants: value.participants,
            classical_info: None,
        }
    }
}

pub type MusicalWorkV2ToV3<T> = VersionedMigration<
    2,
    3,
    InnerMusicalWorkMigrationV2ToV3<T>,
    pallet_midds::Pallet<T, crate::midds::MusicalWorks>,
    <T as frame_system::Config>::DbWeight,
>;

pub struct InnerMusicalWorkMigrationV2ToV3<T: pallet_midds::Config<crate::midds::MusicalWorks>>(
    core::marker::PhantomData<T>,
);

impl<T: pallet_midds::Config<crate::midds::MusicalWorks>> UncheckedOnRuntimeUpgrade
    for InnerMusicalWorkMigrationV2ToV3<T>
where
    <T as pallet_midds::Config<crate::midds::MusicalWorks>>::MIDDS: From<legacy::MusicalWork>,
{
    fn on_runtime_upgrade() -> Weight {
        let mut count = 0;

        MiddsOf::<T, crate::midds::MusicalWorks>::translate::<legacy::MusicalWork, _>(|_k, old| {
            count += 1;
            Some(old.into())
        });

        log::info!(
            target: LOG_TARGET,
            "Storage migration v3 for musical_works finished.",
        );

        // calculate and return migration weights
        T::DbWeight::get().reads_writes(count as u64 + 1, count as u64 + 1)
    }
}
