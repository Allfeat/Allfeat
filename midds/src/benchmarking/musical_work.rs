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

extern crate alloc;
use crate::{
    musical_work::MusicalWork,
    types::{
        musical_work::{
            Iswc, MusicalWorkParticipants, MusicalWorkTitle, MusicalWorkType, Participant,
            ParticipantRole,
        },
        utils::{Key, Language},
    },
};
use parity_scale_codec::Encode;

use super::{BenchmarkHelperT, fill_boundedvec_to_fit};

pub struct BenchmarkHelper;

impl BenchmarkHelperT<MusicalWork> for BenchmarkHelper {
    fn build_base() -> MusicalWork {
        MusicalWork {
            iswc: Default::default(),
            title: Default::default(),
            creation_year: Some(0),
            instrumental: Some(false),
            language: None,
            bpm: None,
            key: None,
            work_type: Some(MusicalWorkType::Original),
            participants: Default::default(),
        }
    }

    fn build_sized(target_size: usize) -> MusicalWork {
        let mut midds = Self::build_base_with_checked_target_size(target_size);

        if midds.encoded_size() >= target_size {
            return midds;
        }

        midds.language = Some(Language::French);
        if midds.encoded_size() >= target_size {
            return midds;
        }

        midds.bpm = Some(128);
        if midds.encoded_size() >= target_size {
            return midds;
        }

        midds.key = Some(Key::C);
        if midds.encoded_size() >= target_size {
            return midds;
        }

        midds.work_type = Some(MusicalWorkType::Original);
        if midds.encoded_size() >= target_size {
            return midds;
        }

        let current_size = midds.encoded_size();
        midds.title =
            fill_boundedvec_to_fit(b'a', MusicalWorkTitle::bound(), current_size, target_size);
        midds.iswc = fill_boundedvec_to_fit(b'a', Iswc::bound(), current_size, target_size);
        let base_participant = Participant {
            id: 0,
            role: ParticipantRole::Arranger,
        };
        midds.participants = fill_boundedvec_to_fit(
            base_participant,
            MusicalWorkParticipants::bound(),
            current_size,
            target_size,
        );

        midds
    }
}
