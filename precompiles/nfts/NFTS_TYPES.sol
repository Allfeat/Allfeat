// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

/**
 * @author Allfeat labs.
 */
library NFTS_TYPES {
    struct OptionalU256 {
        bool has_value;
        uint256 value;
    }

    struct OptionalMintWitness {
        bool has_witness;
        MintWitness data;
    }

    struct CollectionSettings {
        bool is_transferable_items;
        bool is_unlocked_metadata;
        bool is_unlocked_attributes;
        bool is_unlocked_max_supply;
        bool is_deposit_required;
    }

    struct ItemSettings {
        bool is_transferable;
        bool is_unlocked_metadata;
        bool is_unlocked_attributes;
    }

    enum MintType {
        Issuer,
        Public,
        HolderOf
    }

    struct MintInfo {
        MintType mint_type;
        uint256 collection_id; // Only used if type is HolderOf
    }

    struct MintSettings {
        MintInfo mint_info;
        OptionalU256 price;
        OptionalU256 start_block;
        OptionalU256 end_block;
        ItemSettings default_item_settings;
    }

    struct CollectionConfig {
        CollectionSettings settings;
        OptionalU256 max_supply;
        MintSettings mint_settings;
    }

    struct MintWitness {
        OptionalU256 owned_item;
        OptionalU256 mint_price;
    }
}
