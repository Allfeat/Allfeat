// This file is part of Allfeat.

// Copyright (C) 2022-2024 Allfeat.
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

/// Implements encoding and decoding traits for a wrapper type that represents
/// bitflags. The wrapper type should contain a field of type `$size`, where
/// `$size` is an integer type (e.g., u8, u16, u32) that can represent the bitflags.
/// The `$bitflag_enum` type is the enumeration type that defines the individual bitflags.
///
/// This macro provides implementations for the following traits:
/// - `MaxEncodedLen`: Calculates the maximum encoded length for the wrapper type.
/// - `Encode`: Encodes the wrapper type using the provided encoding function.
/// - `EncodeLike`: Trait indicating the type can be encoded as is.
/// - `Decode`: Decodes the wrapper type from the input.
/// - `TypeInfo`: Provides type information for the wrapper type.
macro_rules! impl_codec_bitflags {
	($wrapper:ty, $size:ty, $bitflag_enum:ty) => {
		use parity_scale_codec::EncodeLike;
		use scale_info::{build::Fields, meta_type, Path, Type, TypeParameter};

		impl MaxEncodedLen for $wrapper {
			fn max_encoded_len() -> usize {
				<$size>::max_encoded_len()
			}
		}
		impl Encode for $wrapper {
			fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
				self.0.bits().using_encoded(f)
			}
		}
		impl EncodeLike for $wrapper {}
		impl Decode for $wrapper {
			fn decode<I: parity_scale_codec::Input>(
				input: &mut I,
			) -> ::core::result::Result<Self, parity_scale_codec::Error> {
				let field = <$size>::decode(input)?;
				Ok(Self(BitFlags::from_bits(field as $size).map_err(|_| "invalid value")?))
			}
		}

		impl TypeInfo for $wrapper {
			type Identity = Self;

			fn type_info() -> Type {
				Type::builder()
					.path(Path::new("BitFlags", module_path!()))
					.type_params(vec![TypeParameter::new("T", Some(meta_type::<$bitflag_enum>()))])
					.composite(
						Fields::unnamed()
							.field(|f| f.ty::<$size>().type_name(stringify!($bitflag_enum))),
					)
			}
		}
	};
}
pub(crate) use impl_codec_bitflags;
