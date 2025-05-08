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

use crate::{
	musical_work::MusicalWork,
	types::{
		musical_work::{MusicalWorkType, Participant, PartipantRole},
		utils::{Key, Language, Mode},
	},
};

use super::{fill_boundedvec, BenchmarkHelperT};

pub struct BenchmarkHelper;

impl BenchmarkHelperT<MusicalWork> for BenchmarkHelper {
	const FIELD_MAX_SIZE: u32 = 256;

	fn build_mock(size: u32) -> MusicalWork {
		let iswc = b"T1234567890".to_vec().try_into().expect("ISWC mock is valid");
		let title = fill_boundedvec(b'M', size);

		let participant = Participant { id: 1, role: PartipantRole::Composer };

		let participants = fill_boundedvec(participant, size);

		let work_type = MusicalWorkType::Original;

		MusicalWork {
			iswc,
			title,
			creation_year: 2024,
			instrumental: false,
			language: Some(Language::French),
			bpm: Some(120),
			key: Some(Key::C),
			mode: Some(Mode::Major),
			work_type,
			participants,
		}
	}
}
