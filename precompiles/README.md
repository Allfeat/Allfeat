# NFTs precompiles

This folder contains a precompile in order to interact with the substrate [nfts pallet](https://docs.rs/pallet-nfts/latest/pallet_nfts/) through EVM. The implementation spans several aspects of NFT operations, including creation, trading, and management, structured into different modules.

## Structure

The project is organized into multiple directories, each serving a different aspect of the NFT lifecycle:

- `collections`: Handling of NFT collections.
- `factory`: Factory pattern for NFT creation.
- `swap`: NFT trading mechanisms.
- `tests`: Testing suite for all NFT-related functionality.
- `types`: Rust types for NFT operations.

## Components

Each directory contains:

- Solidity interface (`.sol` files).
- Rust Cargo project (`.toml` files and `src/` directory) which includes:
  - `lib.rs`: Main library file.
  - `mock.rs`: Mocks for unit testing.
  - `tests.rs`: Unit tests.
