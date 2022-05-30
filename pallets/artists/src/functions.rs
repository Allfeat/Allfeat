use super::*;
use frame_support::traits::Get;

impl<T: Config<I>, I: 'static> Pallet<T, I> {
    // Assets relative
    pub fn create_and_init_asset(
        id: T::AssetId,
        account: T::AccountId,
        name: Vec<u8>,
        symbol: Vec<u8>,
    ) -> Result<(), DispatchError> {
        // Create the asset of the new artist
        T::Assets::create(
            id,
            account.clone(),
            false,
            T::MinBalance::get(),
        )?;
        // Set the metadatas of the artist asset
        T::Assets::set(
            id,
            &account,
            name,
            symbol,
            T::Decimals::get(),
        )?;
        // Mint the default supply of the artist asset
        T::Assets::mint_into(
            id,
            &account,
            T::DefaultSupply::get(),
        )?;

        Ok(())
    }
}