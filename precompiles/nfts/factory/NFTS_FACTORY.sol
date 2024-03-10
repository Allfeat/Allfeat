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
     * @notice Create a new collection from the caller address.
     * @param admin The address which is admin of everything in the collection at the creation
     * @param config The configuration of the collection
     */
    /// @custom:selector 28d66e67
    function create(address admin, CollectionConfig calldata config) external returns (bool);

    /**
     * @notice Set the acceptance of ownership for a particular account.
     * @param collection The identifier of the collection whose ownership the caller is
	 * willing to accept
     */
    /// @custom:selector 8c462cc0
    function set_accept_ownership(uint256 collection) external returns (bool);
}
