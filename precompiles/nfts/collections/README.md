# Collections precompile

This folder contains the collections precompile. It is designed to handle NFTs collections operations such as minting, burning, fetching details, locking and unlocking, transfering and modifying attributes/metadata.

## EVM Address Mapping

In order to convert EVM's `AccountId` to Substrate's `CollectionId`,
we ensure that the prefix is 0XFFFFFFFF and we take the lowest 128 bits as the `CollectionId`.
