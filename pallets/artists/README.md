# 🎵 Artist Registry Pallet 🎵

Welcome to the Artist Registry Pallet for the Allfeat Network. This pallet manage registering of artists on the blockchain, allowing them to associate with an artist name, a list of musical genres, assets, and more!

## 🌟 Features

1. **Artist Registration**: Artists can self-register on the blockchain.
2. **Artist Unregistration**: Artists can unregister themselves after a specified period.
3. **Information Update**: Artists can update their information such as aliases, genres, description, and assets.
4. **Deposit Reservation**: A deposit is required for registration to prevent spam.
5. **Artist Lookup**: Retrieve an artist by their account ID or by their name.

## 🔧 Pallet Configuration

To use this pallet in your Substrate runtime, you need to specify several configurations:

- `Currency`: How to handle the deposit for artist creation.
- `BaseDeposit`: The base deposit required for registration.
- `ByteDeposit`: Deposit per byte for placing data hashes.
- `UnregisterPeriod`: How long a registered artist must wait before unregistering.
- `MaxNameLen`: Maximum length of the artist's name.
- `MaxGenres`: Maximum number of genres an artist can have.
- `MaxAssets`: Maximum number of assets an artist can have.
- `MaxContracts`: Maximum number of contracts an artist can have.

## 🚀 How to Use (via Substrate)

1. **Registration**:
    ```rust
    let main_name = b"MyArtistName".to_vec();
    let genres = vec![MusicGenre::Rock, MusicGenre::Pop];
    let assets = vec![b"Asset1".to_vec(), b"Asset2".to_vec()];
    ArtistRegistry::register(origin, main_name, None, genres, None, assets)?;
    ```

2. **Unregistration**:
    ```rust
    ArtistRegistry::unregister(origin)?;
    ```

3. **Update**:
    ```rust
    let new_data = UpdatableData { ... };  // your new data
    ArtistRegistry::update(origin, new_data)?;
    ```

## ❗ Possible Errors

The pallet defines several errors that can be returned during calls:

- `NotUniqueGenre`: A genre appears multiple times.
- `NameUnavailable`: The name is already taken by a verified artist.
- `NotRegistered`: Account isn't registered as an artist.
- `AlreadyRegistered`: This account ID is already registered as an artist.
- `IsVerified`: The artist is verified and can't unregister.
- `PeriodNotPassed`: The unregistering period hasn't fully passed.
- `Full`: The maximum value possible for this field has been breached.
- `NotFound`: The element wasn't found.

## 💌 Conclusion

This pallet provides a robust and flexible solution to manage an artist registry on the Allfeat networks. With tweakable configurations, it can be adapted to various use-cases and requirements. Integrate it into the runtime and give artists a fresh platform to shine!