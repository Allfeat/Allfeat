#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use fp_evm::PrecompileHandle;
use frame_support::dispatch::{GetDispatchInfo, PostDispatchInfo};
use pallet_evm::AddressMapping;
use parity_scale_codec::Encode;
use precompile_utils::{prelude::*, solidity, EvmResult};
use sp_core::{MaxEncodedLen, H160, H256, U256};
use sp_runtime::{traits::Dispatchable, SaturatedConversion};
use sp_std::{marker::PhantomData, vec, vec::Vec};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// Precompile exposing pallet_artists as an EVM-compatible interface.
pub struct ArtistsPrecompile<Runtime>(PhantomData<Runtime>);

type ArtistOf<T> = Artist<<T as pallet_artists::Config>::MaxNameLen>;
type ArtistDataOf<T> = ArtistData<<T as pallet_artists::Config>::MaxNameLen>;

#[precompile_utils::precompile]
#[precompile::test_concrete_types(mock::Runtime)]
impl<Runtime> ArtistsPrecompile<Runtime>
where
	Runtime: pallet_artists::Config + pallet_evm::Config + pallet_timestamp::Config,
	Runtime::AccountId: Into<H160>,
	Runtime::Hash: From<H256>,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::RuntimeCall: From<pallet_artists::Call<Runtime>>,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	<Runtime as pallet_timestamp::Config>::Moment: Into<U256>,
	H256: From<<Runtime as frame_system::Config>::Hash>,
{
	#[precompile::public("get_artist(address)")]
	#[precompile::view]
	fn get_artist(
		handle: &mut impl PrecompileHandle,
		account: Address,
	) -> EvmResult<ArtistOf<Runtime>> {
		handle.record_db_read::<Runtime>(pallet_artists::Artist::<Runtime>::max_encoded_len())?;

		let account: H160 = account.into();
		let account: Runtime::AccountId = Runtime::AddressMapping::into_account_id(account);

		let result = pallet_artists::Pallet::<Runtime>::get_artist_by_id(&account);
		Ok(Self::artist_to_output(result)?)
	}

	fn artist_to_output(
		artist: Option<pallet_artists::Artist<Runtime>>,
	) -> MayRevert<ArtistOf<Runtime>> {
		if artist.is_none() {
			return Ok(ArtistOf::<Runtime>::default());
		}

		let artist = artist.expect("none case checked above; qed");
		let artist_data: ArtistDataOf<Runtime> = ArtistData {
			owner: Address::from(artist.owner().clone().into()),
			registered_at: artist.registered_at().clone().saturated_into(),
			verification: match artist.verified_at() {
				Some(x) => Verification { is_verified: true, verified_at: (*x).saturated_into() },
				None => Verification { is_verified: false, verified_at: Default::default() },
			},
			main_name: artist.main_name().to_vec().into(),
			alias: match artist.alias() {
				Some(x) => Alias { has_alias: true, alias: x.to_vec().into() },
				None => Alias { has_alias: false, alias: BoundedBytes::from(vec![]) },
			},
			genres: artist.genres().iter().map(|genre| genre.encode()).collect(),
			description: match artist.description() {
				Some(x) => DescriptionPreimage { has_preimage: true, preimage: (*x).into() },
				None => DescriptionPreimage { has_preimage: false, preimage: Default::default() },
			},
			assets: artist.assets().iter().map(|hash| (*hash).into()).collect(),
			contracts: artist
				.contracts()
				.iter()
				.map(|id| Address::from(id.clone().into()))
				.collect(), // TODO
		};
		let evm_artist: ArtistOf<Runtime> = Artist { is_artist: true, data: artist_data };

		Ok(evm_artist)
	}
}

#[derive(Eq, PartialEq, Default, Debug, solidity::Codec)]
pub struct Verification {
	is_verified: bool,
	verified_at: u32,
}

#[derive(Eq, PartialEq, Debug, solidity::Codec)]
pub struct Alias<NameLen> {
	has_alias: bool,
	alias: BoundedBytes<NameLen>,
}

#[derive(Eq, PartialEq, Default, Debug, solidity::Codec)]
pub struct DescriptionPreimage {
	has_preimage: bool,
	preimage: H256,
}

#[derive(Eq, PartialEq, Debug, solidity::Codec)]
pub struct ArtistData<NameLen> {
	owner: Address,
	registered_at: u32,
	verification: Verification,
	main_name: BoundedBytes<NameLen>,
	alias: Alias<NameLen>,
	// Genres are stored as the scale encoded enum
	genres: Vec<Vec<u8>>,
	description: DescriptionPreimage,
	assets: Vec<H256>,
	contracts: Vec<Address>,
}

impl<T> Default for ArtistData<T> {
	fn default() -> Self {
		Self {
			owner: Default::default(),
			registered_at: Default::default(),
			verification: Default::default(),
			main_name: BoundedBytes::from(vec![]),
			alias: Alias { has_alias: Default::default(), alias: BoundedBytes::from(vec![]) },
			genres: Default::default(),
			description: Default::default(),
			assets: Default::default(),
			contracts: Default::default(),
		}
	}
}

#[derive(Debug, solidity::Codec)]
pub struct Artist<NameLen> {
	is_artist: bool,
	data: ArtistData<NameLen>,
}

impl<T> Default for Artist<T> {
	fn default() -> Self {
		Self { is_artist: Default::default(), data: Default::default() }
	}
}
