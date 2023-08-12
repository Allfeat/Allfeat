#!/bin/bash

# Create `WeightInfo` implementations for all the pallets and store it in the weight module of the `symphonie-runtime`.

SYMPHONIE_WEIGHT_DIR=runtime/symphonie/src/weights
NODE=./target/release/allfeat

mkdir -p $SYMPHONIE_WEIGHT_DIR

pallets=(
    "pallet_sudo" \
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
