// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

import "./NFTS_TYPES.sol";

interface NFTS_SWAP {
    /**
     * @notice Register a new atomic swap, declaring an intention to send an `item` in exchange for.
     * `desired_item` from caller to target on the current blockchain.
     * The target can execute the swap during the specified `duration` of blocks (if set).
	 * Additionally, the price could be set for the desired `item`.
	 * Caller must be the Owner of the item.
	 *
     * @param offered_collection The collection of the item.
     * @param offered_item The item an owner wants to give.
     * @param desired_collection The collection of the desired item.
     * @param maybe_desired_item The desired item an owner wants to receive.
     * @param maybe_price The price an owner is willing to pay or receive for the desired `item`.
     * @param duration  A deadline for the swap. Specified by providing the number of blocks
	 * after which the swap will expire.
     */
    /// @custom:selector b754cefa
    function create_swap(
        uint256 offered_collection,
        uint256 offered_item,
        uint256 desired_collection,
        OptionalU256 maybe_desired_item,
        OptionalPriceWithDirection maybe_price,
        uint256 duration
    ) external returns (bool);

    /**
     * @notice Cancel an atomic swap.
     * Caller must be an owner of the `item` if the deadline hasn't expired.
	 *
     * @param offered_collection The collection of the item.
     * @param offered_item The item an owner wants to give.
     */
    /// @custom:selector cd568c38
    function cancel_swap(uint256 offered_collection, uint256 offered_item) external returns (bool);

    /**
     * @notice Claim an atomic swap.
     * This method executes a pending swap, that was created by a counterpart before.
	 * Caller must be the Owner of the item.
	 *
     * @param send_collection  The collection of the item to be sent.
     * @param send_item The item to be sent.
     * @param receive_collection The collection of the item to be received.
     * @param receive_item The item to be received.
     * @param witness_price A price that was previously agreed on.
     */
    /// @custom:selector 0406ab6c
    function claim_swap(
        uint256 send_collection,
        uint256 send_item,
        uint256 receive_collection,
        uint256 receive_item,
        OptionalPriceWithDirection witness_price
    ) external returns (bool);
}
