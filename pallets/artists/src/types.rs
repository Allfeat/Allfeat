use super::*;
use frame_support::pallet_prelude::*;


/// The informations stored on-chain for an artist.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct ArtistInfos<ArtistId, AccountId, BoundedString, BlockNumber> {
    /// The identifier of the artist.
    pub(super) id: ArtistId,
    /// The identifier of the account of the artist.
    pub(super) account: AccountId,
    /// The name of the artist.
    pub(super) name: BoundedString,
    /// The block number when the artist was created
    pub(super) age: BlockNumber,
}