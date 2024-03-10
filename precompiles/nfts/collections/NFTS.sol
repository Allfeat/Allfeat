// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

import "../NFTS_TYPES.sol";

/**
 * @title NFTs Collection Interface
 * @author Allfeat labs.
 * @dev Interface to interact with collections of the native NFTs pallet of the Allfeat chain.
 */
interface INFTS {
    // Getters

    /// @notice Get the details of the collection.
    /// @custom:selector b87f86b7
    function get_details() external view returns (CollectionDetails);

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

    /**
     * @notice Cancel all the approvals of a specific item. Caller should be the Owner of the item.
     * @param item The item of the collection of whose approvals will be cleared.
     */
    /// @custom:selector 6f83fe8a
    function cancel_all_approvals(uint256 item) external returns (bool);

    /**
     * @notice Disallows changing the metadata or attributes of the item (as a Sealing). Caller should be the Admin of the Collection.
     * @param item The item to be locked
     * @param lock_metadata Specifies whether the metadata should be sealed.
     * @param lock_attributes Specifies whether the attributes in the `CollectionOwner` namespace
	 * should be sealed.
     */
    /// @custom:selector 91743611
    function seal_item_properties(uint256 item, bool lock_metadata, bool lock_attributes) external returns (bool);

    /**
     * @notice Set an attribute for the collection. Caller should fit with the namespace passed.
     * @param namespace Attribute's namespace.
     * @param key The key of the attribute.
     * @param value The value of the attribute.
     */
    /// @custom:selector e8971f23
    function set_collection_attribute(AttributeNamespaceInfo namespace, string key, string value) external returns (bool);

    /**
     * @notice Set an attribute for a specified item. Caller should fit with the namespace passed.
     * @param item Set the attribute of item.
     * @param namespace Attribute's namespace.
     * @param key The key of the attribute.
     * @param value The value of the attribute.
     */
    /// @custom:selector 123ffb18
    function set_item_attribute(uint256 item, AttributeNamespaceInfo namespace, string key, string value) external returns (bool);

    /**
     * @notice Clear an attribute for the collection. Caller should be the Owner of the attribute.
     * @param namespace Attribute's namespace.
     * @param key The key of the attribute to clear.
     */
    /// @custom:selector 07ac98df
    function clear_collection_attribute(AttributeNamespaceInfo namespace, string key) external returns (bool);

    /**
     * @notice Clear an attribute for an item. Caller should be the Owner of the attribute.
     * @param item Clear the attribute of item.
     * @param namespace Attribute's namespace.
     * @param key The key of the attribute to clear.
     */
    /// @custom:selector 29eaab3f
    function clear_item_attribute(uint256 item, AttributeNamespaceInfo namespace, string key) external returns (bool);

    /**
     * @notice Approve item's attributes to be changed by a delegated third-party account. Caller should be the Owner of the item.
     * @param item The item that holds attributes.
     * @param delegate The account to delegate permission to change attributes of the item.
     */
    /// @custom:selector 620fea0d
    function approve_item_attributes(uint256 item, address delegate) external returns (bool);

    /**
     * @notice Cancel the previously provided approval to change item's attributes. Caller should be the Owner of the item.
     * @param item The item that holds attributes.
     * @param delegate The delegated account from to clear attributes of the item.
     */
    /// @custom:selector e96389a9
    function cancel_item_attributes_approval(uint256 item, address delegate, CancelAttributesApprovalWitness witness) external returns (bool);

    /**
     * @notice Set the metadata for an item. Caller should be the Owner of the collection.
     * @param item The item that hold the metadata.
     * @param data The general information of this item.
     */
    /// @custom:selector 914384e8
    function set_metadata(uint256 item, string data) external returns (bool);

    /**
     * @notice Clear the metadata for an item. Caller should be the Owner of the collection.
     * @param item The item that hold metadata to clear.
     */
    /// @custom:selector f7948baa
    function clear_metadata(uint256 item) external returns (bool);

    /**
     * @notice Set the metadata for the collection. Caller should be the Owner of the collection.
     * @param data The general information of this collection.
     */
    /// @custom:selector ee9b0247
    function set_collection_metadata(string data) external returns (bool);

    /**
     * @notice Clear the metadata for the collection. Caller should be the Owner of the collection.
     */
    /// @custom:selector 8699f6de
    function clear_collection_metadata() external returns (bool);

    /**
     * @notice the maximum number of items a collection could have. Caller should be the Owner of the collection.
     * @param max_supply The maximum number of items a collection could have.
     */
    /// @custom:selector 5c59e577
    function set_collection_max_supply(uint32 max_supply) external returns (bool);

    /**
     * @notice Update mint settings. Caller should be the Issuer of the collection.
     * @param mint_settings The new mint settings.
     */
    /// @custom:selector 9f8ca97d
    function update_mint_settings(MintSettings mint_settings) external returns (bool);

    /**
     * @notice Set (or reset) the price for an item. Caller should not the Owner of the item.
     * @param item The item to set the price for.
     * @param price The price for the item. No value reset the price.
     * @param whitelisted_buyer Restricts the buy operation to a specific account.
     */
    /// @custom:selector fc019a21
    function set_price(uint256 item, OptionalU256 price, OptionalAddress whitelisted_buyer) external returns (bool);

    /**
     * @notice Allows to buy an item if it's up for sale. Caller should not be the Owner of the item.
     * @param item The item the caller wants to buy.
     * @param bid_price The price the caller is willing to pay.
     */
    /// @custom:selector 0a6169cf
    function buy_item(uint256 item, uint256 bid_price) external returns (bool);
}
