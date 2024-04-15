// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @dev The IERC20 contract's address.
address constant IERC20_ADDRESS = 0x0000000000000000000000000000000000000800;

/// @dev The IERC20 contract's instance.
IERC20 constant IERC20_CONTRACT = IERC20(IERC20_ADDRESS);

/**
 * @title ERC20 interface
 * @dev Interface of the ERC20 standard as defined in the EIP.
 * @dev copied from https://github.com/OpenZeppelin/openzeppelin-contracts
 * @custom:address 0x0000000000000000000000000000000000000800
 */
interface IERC20 {
    /**
     * @dev Emitted when `value` tokens are moved from one account (`from`) to
     * another (`to`).
     *
     * Note that `value` may be zero.
     */
    /// @custom:selector ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef
    event Transfer(address indexed from, address indexed to, uint256 value);

    /**
     * @dev Emitted when the allowance of a `spender` for an `owner` is set by
     * a call to {approve}. `value` is the new allowance.
     */
    /// @custom:selector 8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925
    event Approval(address indexed owner, address indexed spender, uint256 value);

    /// @dev Returns the name of the token.
    /// @custom:selector 06fdde03
    function name() external view returns (string memory);

    /// @dev Returns the symbol of the token.
    /// @custom:selector 95d89b41
    function symbol() external view returns (string memory);

    /// @dev Returns the decimals places of the token.
    /// @custom:selector 313ce567
    function decimals() external view returns (uint8);

    /**
     * @dev Returns the value of tokens in existence.
     */
    /// @custom:selector 18160ddd
    function totalSupply() external view returns (uint256);

    /**
     * @dev Returns the value of tokens owned by `account`.
     */
    /// @custom:selector 70a08231
    function balanceOf(address account) external view returns (uint256);

    /**
     * @dev Moves a `value` amount of tokens from the caller's account to `to`.
     *
     * Returns a boolean value indicating whether the operation succeeded.
     *
     * Emits a {Transfer} event.
     */
    /// @custom:selector a9059cbb
    function transfer(address to, uint256 value) external returns (bool);

    /**
     * @dev Returns the remaining number of tokens that `spender` will be
     * allowed to spend on behalf of `owner` through {transferFrom}. This is
     * zero by default.
     *
     * This value changes when {approve} or {transferFrom} are called.
     */
    /// @custom:selector dd62ed3e
    function allowance(address owner, address spender) external view returns (uint256);

    /**
     * @dev Sets a `value` amount of tokens as the allowance of `spender` over the
     * caller's tokens.
     *
     * Returns a boolean value indicating whether the operation succeeded.
     *
     * IMPORTANT: Beware that changing an allowance with this method brings the risk
     * that someone may use both the old and the new allowance by unfortunate
     * transaction ordering. One possible solution to mitigate this race
     * condition is to first reduce the spender's allowance to 0 and set the
     * desired value afterwards:
     * https://github.com/ethereum/EIPs/issues/20#issuecomment-263524729
     *
     * Emits an {Approval} event.
     */
    /// @custom:selector 095ea7b3
    function approve(address spender, uint256 value) external returns (bool);

    /**
     * @dev Moves a `value` amount of tokens from `from` to `to` using the
     * allowance mechanism. `value` is then deducted from the caller's
     * allowance.
     *
     * Returns a boolean value indicating whether the operation succeeded.
     *
     * Emits a {Transfer} event.
     */
    /// @custom:selector 23b872dd
    function transferFrom(address from, address to, uint256 value) external returns (bool);
}

/// @title Native currency wrapper interface.
/// @dev Allow compatibility with dApps expecting this precompile to be
/// a WETH-like contract.
/// Allfeat address : 0x0000000000000000000000000000000000000802
interface WrappedNativeCurrency {
    /// @dev Provide compatibility for contracts that expect wETH design.
    /// Returns funds to sender as this precompile tokens and the native tokens are the same.
    /// @custom:selector d0e30db0
    function deposit() external payable;

    /// @dev Provide compatibility for contracts that expect wETH design.
    /// Does nothing.
    /// @custom:selector 2e1a7d4d
    /// @param value uint256 The amount to withdraw/unwrap.
    function withdraw(uint256 value) external;

    /// @dev Event emited when deposit() has been called.
    /// @custom:selector e1fffcc4923d04b559f4d29a8bfc6cda04eb5b0d3c460751c2402c5c5cc9109c
    /// @param owner address Owner of the tokens
    /// @param value uint256 The amount of tokens "wrapped".
    event Deposit(address indexed owner, uint256 value);

    /// @dev Event emited when withdraw(uint256) has been called.
    /// @custom:selector 7fcf532c15f0a6db0bd6d0e038bea71d30d808c7d98cb3bf7268a95bf5081b65
    /// @param owner address Owner of the tokens
    /// @param value uint256 The amount of tokens "unwrapped".
    event Withdrawal(address indexed owner, uint256 value);
}