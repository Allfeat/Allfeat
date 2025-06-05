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
	pallet_prelude::Track,
	types::{genre::MusicGenre, track::TrackVersion, utils::Key},
	MiddsId,
};

use alloc::vec;

use super::{fill_boundedvec, BenchmarkHelperT};

pub struct BenchmarkHelper;

impl BenchmarkHelperT<Track> for BenchmarkHelper {
	const FIELD_MAX_SIZE: u32 = 256;

	fn build_sized_mock(size: u32) -> Track {
		let midds_id = 1; // simulate a fixed MIDDS ID

		Track {
			isrc: b"USRC17607839".to_vec().try_into().expect("valid ISRC"),
			musical_work: midds_id,
			artist: midds_id,
			producers: fill_boundedvec(midds_id, size),
			performers: fill_boundedvec(midds_id, size),
			contributors: fill_boundedvec(midds_id, size),
			title: fill_boundedvec(b'T', size),
			title_aliases: fill_boundedvec(fill_boundedvec(b'A', size), 2),
			recording_year: 2023,
			genre: MusicGenre::Pop(None),
			genre_extras: fill_boundedvec(MusicGenre::Electronic(None), size),
			version: TrackVersion::Original,
			duration: 210,
			bpm: 120,
			key: Key::C,
			recording_place: fill_boundedvec(b'R', size),
			mixing_place: fill_boundedvec(b'M', size),
			mastering_place: fill_boundedvec(b'P', size),
		}
	}

	fn build_mock() -> Track {
		let midds_id: MiddsId = 1;
		let producer_id: MiddsId = 2;
		let performer_id: MiddsId = 3;
		let contributor_id: MiddsId = 4;

		Track {
			isrc: b"USUG11904269".to_vec().try_into().expect("valid ISRC"),
			musical_work: midds_id,
			artist: midds_id,
			producers: vec![producer_id].try_into().unwrap(),
			performers: vec![performer_id].try_into().unwrap(),
			contributors: vec![contributor_id].try_into().unwrap(),
			title: b"Blinding Lights".to_vec().try_into().unwrap(),
			title_aliases: vec![
				"Feux Aveuglants".as_bytes().to_vec().try_into().unwrap(),
				"盲目的灯光".as_bytes().to_vec().try_into().unwrap(),
			]
			.try_into()
			.unwrap(),
			recording_year: 2019,
			genre: MusicGenre::Pop(None),
			genre_extras: vec![MusicGenre::Electronic(None)].try_into().unwrap(),
			version: TrackVersion::Original,
			duration: 200,
			bpm: 171,
			key: Key::Fs,
			recording_place: b"Los Angeles, CA".to_vec().try_into().unwrap(),
			mixing_place: b"MixStar Studios, Virginia Beach".to_vec().try_into().unwrap(),
			mastering_place: b"Sterling Sound, Edgewater".to_vec().try_into().unwrap(),
		}
	}
}
