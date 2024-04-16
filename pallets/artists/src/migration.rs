use super::*;
use frame_support::{migrations::VersionedMigration, pallet_prelude::*, traits::OnRuntimeUpgrade};

#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;

pub mod versioned {
	use super::*;

	pub type V0ToV1<T> = VersionedMigration<
		0,
		1,
		v1::VersionUncheckedMigrateV0ToV1<T>,
		Pallet<T>,
		<T as frame_system::Config>::DbWeight,
	>;
}

pub mod v1 {
	use super::*;
	use sp_runtime::Saturating;

	/// The log target.
	const TARGET: &'static str = "runtime::artists::migration::v1";

	/// The old artist types, useful in pre-upgrade.
	mod v0 {
		use super::*;
		use derive_getters::Getters;
		use frame_support::pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo};
		use frame_system::pallet_prelude::BlockNumberFor;
		use sp_runtime::RuntimeDebug;

		pub(super) type ArtistAliasOf<T> = BoundedVec<u8, <T as Config>::MaxNameLen>;

		#[derive(
			Encode, MaxEncodedLen, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, Getters,
		)]
		#[scale_info(skip_type_params(T))]
		pub struct Artist<T>
		where
			T: frame_system::Config + Config,
		{
			// Main data
			/// The artist's identifier. While the predominant mapping employs AccountId => Artist,
			/// it's essential to include this in the artist's data since verified artists can be
			/// retrieved by their name as well.
			pub(super) owner: AccountIdOf<T>,
			/// When the artist got registered on-chain.
			pub(super) registered_at: BlockNumberFor<T>,
			/// When the artist got verified.
			verified_at: Option<BlockNumberFor<T>>,
			// Metadata
			/// The name of the artist.
			/// This is generally the main name of how we usually call the artist (e.g: 'The
			/// Weeknd') This is fixed and can't be changed after the registration.
			pub(super) main_name: BoundedVec<u8, T::MaxNameLen>,
			/// An alias to the main name.
			/// This name can be changed compared to the 'nickname'
			alias: Option<ArtistAliasOf<T>>,
			/// The main music genres of the artists.
			pub(super) genres: BoundedVec<MusicGenre, T::MaxGenres>,
			// Metadata Fingerprint
			// Given the significant size of certain data associated with an artist,
			// we choose to store a digital fingerprint (hash) of this data rather than
			// the raw data itself. This fingerprint acts as a unique digital reference,
			// and services can use it to compare and validate the artist's data, ensuring
			// that it has been approved and recorded on the blockchain by the artist themselves.
			/// The digital fingerprint (hash) of the artist's description.
			pub(crate) description: Option<T::Hash>,
			/// Digital assets (such as photos, profile pictures, banners, videos, etc.)
			/// that officially represent the artist. These fingerprints allow for the
			/// verification of the authenticity of these assets.
			pub(super) assets: BoundedVec<T::Hash, T::MaxAssets>,
			// Linked chain logic data
			/// Associated smart-contracts deployed by dApps for the artist (e.g: royalties
			/// contracts)
			contracts: BoundedVec<AccountIdOf<T>, ConstU32<512>>,
		}
	}

	impl<T: Config> From<v0::Artist<T>> for Artist<T> {
		fn from(value: v0::Artist<T>) -> Self {
			Self {
				owner: value.owner,
				registered_at: value.registered_at,
				main_name: value.main_name,
				main_type: Default::default(),
				extra_types: Default::default(),
				genres: value.genres,
				description: value.description,
				assets: value.assets,
			}
		}
	}

	/// Migration to V1 Artist struct.
	/// - Removing contracts, alias and verification.
	/// - Adding artist types.
	pub struct VersionUncheckedMigrateV0ToV1<T>(PhantomData<T>);
	impl<T: Config> OnRuntimeUpgrade for VersionUncheckedMigrateV0ToV1<T> {
		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
			let artists = crate::ArtistOf::<T>::iter().count();
			log::info!(
				target: TARGET,
				"pre-upgrade state contains '{}' artists.",
				artists
			);
			Ok((artists as u64).encode())
		}

		fn on_runtime_upgrade() -> Weight {
			log::info!(
				target: TARGET,
				"running storage migration from version 0 to version 1."
			);

			let mut translated: u64 = 0;

			ArtistOf::<T>::translate::<v0::Artist<T>, _>(|_key, old_artist| {
				translated.saturating_inc();
				Some(old_artist.into())
			});
			StorageVersion::new(1).put::<Pallet<T>>();

			log::info!(target: TARGET, "all {} artists migrated", translated);

			T::DbWeight::get().reads_writes(translated + 1, translated + 1)
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(state: Vec<u8>) -> Result<(), TryRuntimeError> {
			let artists_to_migrate: u64 = Decode::decode(&mut &state[..])
				.expect("failed to decode the state from pre-upgrade.");
			let artists = ArtistOf::<T>::iter().count() as u64;
			log::info!("post-upgrade expects '{}' artists to have been migrated.", artists);
			ensure!(artists_to_migrate == artists, "must migrate all artists.");
			log::info!(target: TARGET, "migrated all artists.");
			Ok(())
		}
	}
}
