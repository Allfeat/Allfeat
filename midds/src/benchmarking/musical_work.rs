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
		musical_work::{MusicalWorkType, Participant, PartipantRole},
		utils::{Key, Language},
	},
};
use alloc::vec;

use super::{fill_boundedvec, BenchmarkHelperT};

pub struct BenchmarkHelper;

impl BenchmarkHelperT<MusicalWork> for BenchmarkHelper {
	const FIELD_MAX_SIZE: u32 = 256;

	fn build_sized_mock(size: u32) -> MusicalWork {
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
			work_type,
			participants,
		}
	}
	fn build_mock() -> MusicalWork {
		MusicalWork {
			iswc: b"T0702330071".to_vec().try_into().expect("Mock value"),
			title: b"Billie Jean".to_vec().try_into().expect("Mock value"),
			creation_year: 1983,
			instrumental: false,
			language: Some(Language::English),
			bpm: Some(117),
			key: Some(Key::B),
			work_type: MusicalWorkType::Original,
			participants: vec![
				Participant { id: 1, role: PartipantRole::Composer },
				Participant { id: 2, role: PartipantRole::Editor },
				Participant { id: 3, role: PartipantRole::Editor },
				Participant { id: 4, role: PartipantRole::Editor },
			]
			.try_into()
			.expect("Mock value"),
		}
	}
}
