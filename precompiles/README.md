# NFTs precompiles

This folder contains a set of EVM precompiles designed to optimize and extend the functionality of non-fungible tokens (NFTs) on our blockchain. The implementation spans several aspects of NFT operations, including creation, trading, and management, structured into different modules.
It's based on the substrate [nfts pallet](https://docs.rs/pallet-nfts/latest/pallet_nfts/)

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
