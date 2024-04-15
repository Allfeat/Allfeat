// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @dev The Artists contract's address.
address constant ARTISTS_ADDRESS = 0x0000000000000000000000000000000000000801;

/// @dev The Artists contract's instance.
Artists constant ARTISTS_CONTRACT = Artists(
    ARTISTS_ADDRESS
);

/// @title Artists precompile
/// Allows to interact with the artists pallet from the EVM.
/// Addresses:
/// - 0x0000000000000000000000000000000000000803: Artists
interface Artists {
    struct DescriptionPreimage {
        bool has_preimage;
        bytes32 preimage;
    }

    enum ArtistType {
        Singer,
        Instrumentalist,
        Composer,
        Lyricist,
        Producer,
        DiscJokey,
        Conductor,
        Arranger,
        Engineer,
        Director
    }

    struct ArtistData {
        address owner;
        uint32 registered_at;
        ArtistType main_type;
        ArtistType[] extra_types;
        string main_name;
        bytes[] genres;
        DescriptionPreimage description;
        bytes32[] assets;
    }

    struct Artist {
        bool is_artist;
        ArtistData data;
    }

    /// @dev Retrieve artist data of the specified account.
    /// @custom:selector e27e4da4
    /// @param account The requested account
    function get_artist(address account) external view returns (Artist memory);
}