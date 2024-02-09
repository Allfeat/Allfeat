// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @dev The Artists contract's address.
address constant ARTISTS_ADDRESS = 0x0000000000000000000000000000000000000803;

/// @dev The Artists contract's instance.
Artists constant ARTISTS_CONTRACT = Artists(
    ARTISTS_ADDRESS
)

/// @title Artists precompile
/// Allows to interact with the artists pallet from the EVM.
/// Addresses:
/// - 0x0000000000000000000000000000000000000803: Artists
interface Artists {
    struct Verification {
        bool is_verified,
        uint32 verified_at,
    }

    struct Alias {
        bool has_alias,
        string alias,
    }

    struct DescriptionPreimage {
        bool has_preimage,
        bytes32 preimage,
    }

    struct ArtistData {
        address owner,
        uint32 registered_at,
        Verification verification,
        string main_name,
        Alias alias,
        bytes[] genres,
        DescriptionPreimage description,
        bytes32[] assets,
        address[] contracts,
    }

    struct Artist {
        bool is_artist,
        ArtistData data
    }

    /// @dev Retrieve artist data of the specified account.
    /// @custom:selector e27e4da4
    /// @param who The requested account
    function get_artist(address account) external view returns (Artist memory)
}