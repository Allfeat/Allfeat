// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(test)]
pub mod mock;
#[cfg(test)]
pub mod tests;

mod types;
mod functions;
pub mod weights;
pub use types::*;

use codec::HasCompact;
use sp_std::prelude::*;
use sp_runtime::traits::{AtLeast32BitUnsigned, StaticLookup};
use frame_support::{
    Blake2_128Concat, BoundedVec, dispatch::DispatchResult,
    traits::{
		fungibles::{Create, Mutate, metadata::Mutate as MetadataMutate},
		Currency,
	},
    dispatch::DispatchError
};

pub use pallet::*;
pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
    use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T, I = ()>(_);

    // Pallet configuration
    #[pallet::config]
    pub trait Config<I: 'static = ()>: frame_system::Config {
        type Event: From<Event<Self, I>> + IsType<<Self as frame_system::Config>::Event>;

        /// The units in which we record balances.
		type Balance: Member
            + Parameter
            + AtLeast32BitUnsigned
            + Default
            + Copy
            + MaybeSerializeDeserialize
            + MaxEncodedLen
            + TypeInfo;

		type Currency: Currency<Self::AccountId>;

        /// The identifier of an artist
        type ArtistId: Member
            + Parameter
			+ Default
			+ Copy
			+ HasCompact
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen
			+ TypeInfo;

        /// Identifier for the class of asset.
		type AssetId: Member
            + Parameter
            + Default
            + Copy
            + HasCompact
            + MaybeSerializeDeserialize
            + MaxEncodedLen
            + TypeInfo
            + From<Self::ArtistId>;

        type Assets: Create<Self::AccountId,
                AssetId = Self::AssetId,
                Balance = Self::Balance,
            >
            + Mutate<Self::AccountId,
                AssetId = Self::AssetId,
                Balance = Self::Balance,
            >
            + MetadataMutate<Self::AccountId,
                AssetId = Self::AssetId,
            >;

        /// The maximum length of an artist name or symbol stored on-chain.
		#[pallet::constant]
		type StringLimit: Get<u32>;

        #[pallet::constant]
		type DefaultSupply: Get<Self::Balance>;

        #[pallet::constant]
		type MinBalance: Get<Self::Balance>;

        #[pallet::constant]
		type Decimals: Get<u8>;

		type WeightInfo: WeightInfo;
    }

    #[pallet::storage]
    #[pallet::getter(fn get_artist)]
    pub(super) type ArtistStorage<T: Config<I>, I: 'static = ()> = StorageMap<
        _,
        Blake2_128Concat,
        T::ArtistId,
        ArtistInfos<
            T::ArtistId,
            T::AccountId,
            BoundedVec<u8, T::StringLimit>,
            T::BlockNumber,
        >,
        OptionQuery,
    >;

    #[pallet::genesis_config]
	pub struct GenesisConfig<T: Config<I>, I: 'static = ()> {
        /// The exisiting artists at the genesis
        pub artists: Vec<(T::ArtistId, T::AccountId, Vec<u8>, Vec<u8>, Vec<u8>)>,
    }

    #[cfg(feature = "std")]
    impl<T: Config<I>, I: 'static> Default for GenesisConfig<T, I> {
        fn default() -> Self {
            Self {
                artists: Default::default(),
            }
        }
    }

	#[pallet::genesis_build]
    impl <T: Config<I>, I: 'static> GenesisBuild<T, I> for GenesisConfig<T, I> {
        fn build(&self) {
            for (id, account, name, asset_name, asset_symbol) in &self.artists {
                assert!(!ArtistStorage::<T, I>::contains_key(id), "Artist ID already in use");

                let artist_name: BoundedVec<u8, T::StringLimit>
                    = name.clone().try_into().expect("name is too long");
                let age: T::BlockNumber = <frame_system::Pallet<T>>::block_number();

                // Create, set the metadatas and mint the supply of the artist asset
                T::Assets::create(
                    id.clone().into(),
                    account.clone(),
                    false,
                    T::MinBalance::get(),
                ).unwrap();
                // Set the metadatas of the artist asset
                T::Assets::set(
                    id.clone().into(),
                    &account,
                    asset_name.to_vec(),
                    asset_symbol.to_vec(),
                    T::Decimals::get(),
                ).unwrap();
                // Mint the default supply of the artist asset
                T::Assets::mint_into(
                    id.clone().into(),
                    &account,
                    T::DefaultSupply::get(),
                ).unwrap();

                // Inserting the new artist in the storage
                ArtistStorage::<T, I>::insert(
                    id,
                    ArtistInfos {
                        id: *id,
                        account: account.clone(),
                        name: artist_name.clone(),
                        age,
                    }
                );
            }
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config<I>, I: 'static = ()> {
        /// An artist was created.
        ArtistCreated { artist_id: T::ArtistId, name: BoundedVec<u8, T::StringLimit>, block: T::BlockNumber },
    }

    #[pallet::error]
    pub enum Error<T, I = ()> {
        /// The artist id is already used.
        AlreadyExist,
        /// The given string is longer than `T::StringLimit`.
        StringTooLong,
        /// The given string is  too short to be valid.
        StringTooShort,
    }

    #[pallet::call]
    impl<T: Config<I>, I: 'static> Pallet<T, I> {
        /// Create and insert a new artist.
        ///
        /// Must be called from the root origin.
        ///
        /// The artist asset is initalized with the same ID than the artist and with a supply of `T::DefaultSupply`.
        ///
        /// Parameters:
        /// - `id`: The ID of the new artist to create.
        /// - `asset_id`: The ID of the artist asset to create.
        /// - `account`: The main account of the artist.
        /// - `name`: The name of the artist.
        /// Should be less or equal than `T::StringLimit`.
        /// - `asset_name`: The name of the artist asset.
        /// - `asset_symbol`: The symbol of the artist asset.
        ///
        /// Emits `ArtistCreated` when the artist is successfuly inserted in storage.
        #[pallet::weight(T::WeightInfo::force_create(name.len() as u32, asset_name.len() as u32, asset_symbol.len() as u32,))]
        pub fn force_create(
            origin: OriginFor<T>,
            #[pallet::compact] id: T::ArtistId,
            account: <T::Lookup as StaticLookup>::Source,
            name: Vec<u8>,
            asset_name: Vec<u8>,
            asset_symbol: Vec<u8>,
        ) -> DispatchResult {
			let acc = T::Lookup::lookup(account)?;

            ensure_root(origin)?;
            ensure!(!ArtistStorage::<T, I>::contains_key(id), Error::<T, I>::AlreadyExist);

            let artist_name: BoundedVec<u8, T::StringLimit>
                = name.try_into().expect("name is too long");
            let age: T::BlockNumber = <frame_system::Pallet<T>>::block_number();

            // Create, set the metadatas and mint the supply of the artist asset
            Self::create_and_init_asset(
                id.into(),
                acc.clone(),
                asset_name,
                asset_symbol,
            )?;

            // Inserting the new artist in the storage
            ArtistStorage::<T, I>::insert(
                id,
                ArtistInfos {
                    id,
                    account: acc,
                    name: artist_name.clone(),
                    age,
                }
            );

            Self::deposit_event(Event::ArtistCreated {
                artist_id: id,
                name: artist_name,
                block: age,
            });

            Ok(())
        }
    }
}
