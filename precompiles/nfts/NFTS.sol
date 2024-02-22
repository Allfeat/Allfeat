// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

import "./NFTS_TYPES.sol";

/**
 * @title NFTs Collection Interface
 * @author Allfeat labs.
 * @dev Interface to interact with collections of the native NFTs pallet of the Allfeat chain.
 */
interface INFTS {
    // Storage getters

    // Extrinsic Call

    /**
     * @notice Mint a new item of the specified collection. Caller must comply with mint settings.
     * @param item_id The minted item ID
     * @param mint_to Where to send the minted item
     * @param Optional witness data about the item
     */
    /// @custom:selector cd568c38
    function mint(uint256 item_id, address mint_to, OptionalMintWitness calldata witness_data) external returns (bool);

    /**
     * @notice Destroy a single item. Caller must be the item owner.
     * @param item_id The item ID to burn
     */
    /// @custom:selector 42966c68
    function burn(uint256 item_id) external returns (bool);

    /**
     * @notice Move an item from the sender account to another. Caller nust be the owner or an approved delegate.
     * @param item_id The item ID to transfer
     * @param dest The account receiver of the item
     */
    /// @custom:selector b7760c8f
    function transfer(uint256 item_id, address dest) external returns (bool);

    /**
     * @notice Disallow further unprivileged transfer of an item. Caller should be the Freezer of the collection.
     * @param item_id The item ID to transfer
     */
    /// @custom:selector 81c2e1e8
    function lock_item_transfer(uint256 item_id) external returns (bool);

    /**
     * @notice Re-allow unprivileged transfer of an item. Caller should be the Freezer of the collection.
     * @param item_id The item ID to transfer
     */
    /// @custom:selector 3b8413a5
    function unlock_item_transfer(uint256 item_id) external returns (bool);

    /**
     * @notice Seal specified settings for the whole collection. Caller should be the Owner of the collection.
     * @dev Settings can be sealed but can't be unsealed.
     * @param lock_settings The settings to be sealed
     */
    /// @custom:selector a872c4c8
    function seal_collection(CollectionSettings settings) external returns (bool);

    /**
     * @notice Change the Owner of a collection. Caller should be the Owner of the collection.
     * @param owner The new Owner of this collection (should be approved with set_accept_ownership first)
     */
    /// @custom:selector f0350c04
    function transfer_ownership(address owner) external returns (bool);

    /**
     * @notice Change the Issuer, Admin and Freezer of a collection. Caller should be the Owner of the collection.
     * @param issuer The new Issuer of this collection
     * @param admin The new Admin of this collection
     * @param freezer The new Freezer of this collection
     */
    /// @custom:selector f8bf8e95
    function set_team(address issuer, address admin, address freezer) external returns (bool);

    /**
     * @notice Approve an item to be transferred by a delegated third-party account. Caller should be the Owner of the item.
     * @param item The item to be approved for delegated transfer.
     * @param delegate The account to delegate permission to transfer the item.
     * @param maybe_deadline Optional deadline for the approval. Specified by providing the
	 * number of blocks after which the approval will expire
     */
    /// @custom:selector 0df4508b
    function approve_transfer(uint256 item, address delegate, OptionalU256 maybe_deadline) external returns (bool);

    /**
     * @notice Cancel one of the transfer approvals for a specific item. Caller should be the Owner of the item.
     * @param item The item of the collection of whose approval will be cancelled.
     * @param delegate The account that is going to loose their approval.
     */
    /// @custom:selector 22b856f3
    function cancel_approval(uint256 item, address delegate) external returns (bool);

    /**
     * @notice Cancel all the approvals of a specific item. Caller should be the Owner of the item.
     * @param item The item of the collection of whose approvals will be cleared.
     */
    /// @custom:selector 6f83fe8a
    function cancel_all_approvals(uint256 item) external returns (bool);
}
