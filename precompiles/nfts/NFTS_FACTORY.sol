// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

import "./NFTS_TYPES.sol";

/**
 * @title NFTs interface for collection deployment.
 * @author Allfeat labs.
 * @dev Interface to interact with creates function of the native NFTs pallet of the Allfeat chain.
 */
interface NFTS_FACTORY {
    /**
     * @dev Emitted when `collection_id` NFT collection are created from account (`creator`) with
     * (`owner`) being the owner/administrator.
     *
     */
    /// @custom:selector ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef
    event Collection_created(address indexed owner, address indexed creator, uint256 collection_id);

    /**
     * @notice Create a new collection from the caller address.
     * @param admin The address which is admin of everything in the collection at the creation
     * @param config The configuration of the collection
     */
    /// @custom:selector cd568c38
    function create(address admin, CollectionConfig calldata config) external returns (bool);
}
