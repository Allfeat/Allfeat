use enumflags2::{bitflags, BitFlags};
use frame_election_provider_support::{BoundedVec, Decode, Encode};
use frame_support::{
	pallet_prelude::{MaxEncodedLen, TypeInfo},
	CloneNoBound, EqNoBound, PartialEqNoBound, RuntimeDebugNoBound,
};
use pallet_identity::{Data, IdentityInformationProvider};
use scale_info::{build::Variants, Path, Type};
use sp_core::{Get, RuntimeDebug};
use sp_std::prelude::*;

#[cfg(feature = "runtime-benchmarks")]
use enumflags2::BitFlag;

/// The fields that we use to identify the owner of an account with. Each corresponds to a field
/// in the `IdentityInfo` struct.
#[bitflags]
#[repr(u64)]
#[derive(Clone, Copy, PartialEq, Eq, RuntimeDebug)]
pub enum IdentityField {
	Display,
	Legal,
	Web,
	Matrix,
	Email,
	Twitter,
	Instagram,
	Youtube,
	Tiktok,
	Discord,
	Telegram,
}

impl TypeInfo for IdentityField {
	type Identity = Self;

	fn type_info() -> scale_info::Type {
		Type::builder().path(Path::new("IdentityField", module_path!())).variant(
			Variants::new()
				.variant("Display", |v| v.index(0))
				.variant("Legal", |v| v.index(1))
				.variant("Web", |v| v.index(2))
				.variant("Riot", |v| v.index(3))
				.variant("Email", |v| v.index(4))
				.variant("Image", |v| v.index(6))
				.variant("Twitter", |v| v.index(7)),
		)
	}
}

/// Information concerning the identity of the controller of an account.
///
/// NOTE: This should be stored at the end of the storage item to facilitate the addition of extra
/// fields in a backwards compatible way through a specialized `Decode` impl.
#[derive(
	CloneNoBound,
	Encode,
	Decode,
	EqNoBound,
	MaxEncodedLen,
	PartialEqNoBound,
	RuntimeDebugNoBound,
	TypeInfo,
)]
#[codec(mel_bound())]
#[cfg_attr(test, derive(frame_support::DefaultNoBound))]
#[scale_info(skip_type_params(FieldLimit))]
pub struct IdentityInfo<FieldLimit: Get<u32>> {
	/// Additional fields of the identity that are not catered for with the struct's explicit
	/// fields.
	pub additional: BoundedVec<(Data, Data), FieldLimit>,

	/// A reasonable display name for the controller of the account. This should be whatever it is
	/// that it is typically known as and should not be confusable with other entities, given
	/// reasonable context.
	///
	/// Stored as UTF-8.
	pub display: Data,

	/// The full legal name in the local jurisdiction of the entity. This might be a bit
	/// long-winded.
	///
	/// Stored as UTF-8.
	pub legal: Data,

	/// A representative website held by the controller of the account.
	///
	/// NOTE: `https://` is automatically prepended.
	///
	/// Stored as UTF-8.
	pub web: Data,

	/// The Riot/Matrix handle held by the controller of the account.
	///
	/// Stored as UTF-8.
	pub matrix: Data,

	/// The email address of the controller of the account.
	///
	/// Stored as UTF-8.
	pub email: Data,

	/// The Twitter identity. The leading `@` character may be elided.
	pub twitter: Data,

	/// The Instgram identity, may contain only the instagram username.
	pub instagram: Data,

	/// The Youtube identity, containing the username or URL ID.
	pub youtube: Data,

	/// The Tiktok identity, containing the tiktok username.
	pub tiktok: Data,

	/// The Discord identity, containing the discord username.
	pub discord: Data,

	/// The Telegram identity, the leading `@` character may be elided.
	pub telegram: Data,
}

impl<FieldLimit: Get<u32> + 'static> IdentityInformationProvider for IdentityInfo<FieldLimit> {
	type FieldsIdentifier = u64;

	fn has_identity(&self, fields: Self::FieldsIdentifier) -> bool {
		self.fields().bits() & fields == fields
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn create_identity_info() -> Self {
		let data = Data::Raw(vec![0; 32].try_into().unwrap());

		IdentityInfo {
			additional: vec![(data.clone(), data.clone()); FieldLimit::get().try_into().unwrap()]
				.try_into()
				.unwrap(),
			display: data.clone(),
			legal: data.clone(),
			web: data.clone(),
			matrix: data.clone(),
			email: data.clone(),
			twitter: data.clone(),
			instagram: data.clone(),
			youtube: data.clone(),
			tiktok: data.clone(),
			discord: data.clone(),
			telegram: data.clone(),
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn all_fields() -> Self::FieldsIdentifier {
		IdentityField::all().bits()
	}
}

impl<FieldLimit: Get<u32>> IdentityInfo<FieldLimit> {
	pub(crate) fn fields(&self) -> BitFlags<IdentityField> {
		let mut res = <BitFlags<IdentityField>>::empty();
		if !self.display.is_none() {
			res.insert(IdentityField::Display);
		}
		if !self.legal.is_none() {
			res.insert(IdentityField::Legal);
		}
		if !self.web.is_none() {
			res.insert(IdentityField::Web);
		}
		if !self.matrix.is_none() {
			res.insert(IdentityField::Matrix);
		}
		if !self.email.is_none() {
			res.insert(IdentityField::Email);
		}
		if !self.twitter.is_none() {
			res.insert(IdentityField::Twitter);
		}
		if !self.instagram.is_none() {
			res.insert(IdentityField::Instagram);
		}
		if !self.youtube.is_none() {
			res.insert(IdentityField::Youtube);
		}
		if !self.tiktok.is_none() {
			res.insert(IdentityField::Tiktok);
		}
		if !self.discord.is_none() {
			res.insert(IdentityField::Discord);
		}
		if !self.telegram.is_none() {
			res.insert(IdentityField::Telegram);
		}
		res
	}
}
