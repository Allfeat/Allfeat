#!/bin/bash

BENCHER="frame-omni-bencher"

if ! command -v "$BENCHER" &> /dev/null
then
    echo "$BENCHER is not installed ! Please install it by following https://github.com/paritytech/polkadot-sdk/tree/master/substrate/utils/frame/omni-bencher."
    exit 1
fi

if [ -z "$1" ]; then
    echo "Runtime: $0 <required_string>"
    exit 1
fi

RUNTIME="$1"

if [ "$skip_build" != true ]
then
  echo "[+] Compiling $RUNTIME runtime with benchmarks feature..."
  cargo build --profile=production --package $RUNTIME-runtime --features runtime-benchmarks
fi

RUNTIME_PATH="./target/production/wbuild/$RUNTIME-runtime/${RUNTIME}_runtime.compact.compressed.wasm"

# Manually exclude some pallets.
EXCLUDED_PALLETS=(
  # Pallets without automatic benchmarking
  "pallet_babe"
  "pallet_grandpa"
  "pallet_mmr"
  # Only used for testing, does not need real weights.
  "frame_benchmarking_pallet_pov"
)

# Load all pallet names in an array.
ALL_PALLETS=($(
  $BENCHER v1 benchmark pallet --runtime $RUNTIME_PATH --list=pallets --no-csv-header --genesis-builder-preset=development
))

# Filter out the excluded pallets by concatenating the arrays and discarding duplicates.
PALLETS=($({ printf '%s\n' "${ALL_PALLETS[@]}" "${EXCLUDED_PALLETS[@]}"; } | sort | uniq -u))

echo "[+] Benchmarking ${#PALLETS[@]} Substrate pallets on a total of ${#ALL_PALLETS[@]}."

# Define the error file.
ERR_FILE="benchmarking_errors.txt"
# Delete the error file before each run.
rm -f $ERR_FILE

# Benchmark each pallet.
for PALLET in "${PALLETS[@]}"; do
    FOLDER="$(echo "${PALLET#*_}" | tr '-' '_')";
    WEIGHT_FILE="./runtime/shared/src/weights/${FOLDER}.rs"
    echo "[+] Benchmarking $PALLET with weight file $WEIGHT_FILE";

    EXTRINSIC="*"
  # pallet-evm have only one function to witdhraw and atm "*" is causing a crash.
    if [ "$PALLET" = "pallet_evm" ]; then
      EXTRINSIC="withdraw"
    fi

    OUTPUT=$(
    $BENCHER v1 benchmark pallet \
    --runtime $RUNTIME_PATH \
    --genesis-builder-preset="development" \
    --pallet="$PALLET" \
    --extrinsic="$EXTRINSIC" \
    --output="$WEIGHT_FILE" \
    --header="./HEADER" \
    --template=./.maintain/frame-weight-template.hbs 2>&1
    )
    if [ $? -ne 0 ]; then
        echo "$OUTPUT" >> "$ERR_FILE"
        echo "[-] Failed to benchmark $PALLET. Error written to $ERR_FILE; continuing..."
    fi
done

# Check if the error file exists.
if [ -f "$ERR_FILE" ]; then
  echo "[-] Some benchmarks failed. See: $ERR_FILE"
  exit 1
else
  echo "[+] All benchmarks passed."
  exit 0
fi