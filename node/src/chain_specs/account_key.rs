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

use bip32::{
	Error as Bip32Error, PrivateKey as PrivateKeyT, PrivateKeyBytes, PublicKey as PublicKeyT,
	PublicKeyBytes,
};
use libsecp256k1::{PublicKey, SecretKey};

// `libsecp256k1::PublicKey` wrapped type
pub struct Secp256k1PublicKey(pub PublicKey);
// `libsecp256k1::Secret`  wrapped type
pub struct Secp256k1SecretKey(pub SecretKey);

impl PublicKeyT for Secp256k1PublicKey {
	fn from_bytes(bytes: PublicKeyBytes) -> Result<Self, Bip32Error> {
		let public = PublicKey::parse_compressed(&bytes).map_err(|_| return Bip32Error::Decode)?;
		Ok(Self(public))
	}

	fn to_bytes(&self) -> PublicKeyBytes {
		self.0.serialize_compressed()
	}

	fn derive_child(&self, other: PrivateKeyBytes) -> Result<Self, Bip32Error> {
		let mut child = self.0.clone();
		let secret = SecretKey::parse(&other).map_err(|_| return Bip32Error::Decode)?;
		let _ = child.tweak_add_assign(&secret);
		Ok(Self(child))
	}
}

impl PrivateKeyT for Secp256k1SecretKey {
	type PublicKey = Secp256k1PublicKey;

	fn from_bytes(bytes: &PrivateKeyBytes) -> Result<Self, Bip32Error> {
		let secret = SecretKey::parse(&bytes).map_err(|_| return Bip32Error::Decode)?;
		Ok(Self(secret))
	}

	fn to_bytes(&self) -> PrivateKeyBytes {
		self.0.serialize()
	}

	fn derive_child(&self, other: PrivateKeyBytes) -> Result<Self, Bip32Error> {
		let mut child = self.0.clone();
		let secret = SecretKey::parse(&other).map_err(|_| return Bip32Error::Decode)?;
		let _ = child.tweak_add_assign(&secret);
		Ok(Self(child))
	}

	fn public_key(&self) -> Self::PublicKey {
		Secp256k1PublicKey(PublicKey::from_secret_key(&self.0))
	}
}
