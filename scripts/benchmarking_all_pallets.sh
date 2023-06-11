#!/bin/bash

# Create `WeightInfo` implementations for all the pallets and store it in the weight module of the `symphonie-runtime`.

SYMPHONIE_WEIGHT_DIR=runtime/symphonie/src/weights
NODE=./target/release/allfeat

mkdir -p $SYMPHONIE_WEIGHT_DIR

pallets=(
    "pallet_artists" \
    "pallet_artist_identity" \
    "pallet_assets" \
    "pallet_bags_list" \
    "pallet_balances" \
    "pallet_contracts" \
    "pallet_election_provider_multi_phase" \
    "pallet_election_provider_support_benchmarking" \
    "pallet_identity" \
    "pallet_im_online" \
    "pallet_indices" \
    "pallet_mmr" \
    "pallet_multisig" \
    "pallet_music_styles" \
    "pallet_nfts" \
    "pallet_preimage" \
    "pallet_proxy" \
    "pallet_recovery" \
    "pallet_scheduler" \
    "pallet_session" \
    "pallet_staking" \
    "pallet_state_trie_migration" \
    "frame_system" \
    "pallet_timestamp" \
    "pallet_utility" \
)

echo building runtime-benchmarking feature...
cargo build --release \
    --features runtime-benchmarks \

for pallet in ${pallets[*]}; do
    echo benchmarking pallet: "$pallet"...

    $NODE \
        benchmark pallet \
        --chain=dev \
        --steps=50 \
        --repeat=20 \
        --pallet="$pallet" \
        --extrinsic="*" \
        --execution=wasm \
        --wasm-execution=compiled \
        --heap-pages=4096 \
        --output=./"$SYMPHONIE_WEIGHT_DIR"/"$pallet".rs \
        --template=./.maintain/frame-weight-template.hbs
done
