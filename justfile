#!/usr/bin/env just --justfile

set positional-arguments := true

# Cargo profile used to execute cargo commands.
CARGO_PROFILE := env("PROFILE", "release")
BENCHER := env("BENCHER_PATH", "frame-omni-bencher")

# Build the workspace, i.e. the Melodie runtime (default to release profile).
# The chain runs on `polkadot-omni-node` — there is no node crate to build here.
[no-exit-message]
build:
  echo "Building the Melodie parachain runtime with profile '{{CARGO_PROFILE}}'"
  cargo build --profile {{CARGO_PROFILE}}

[no-exit-message]
build-melodie:
  cargo build --profile=production --package melodie-runtime --features on-chain-release-build

[no-exit-message]
benchmark-pallet runtime="melodie" pallet="":
    cargo build --profile production --features runtime-benchmarks --package {{runtime}}-runtime
    {{BENCHER}} v1 benchmark pallet \
      --runtime "./target/production/wbuild/{{runtime}}-runtime/{{runtime}}_runtime.compact.compressed.wasm" \
      --genesis-builder-preset="development" \
      --pallet={{pallet}} \
      --extrinsic="*" \
      --header="./HEADER" \
      --template=./.maintain/frame-weight-template.hbs 2>&1

[no-exit-message]
benchmark-weights-testnet:
  ./scripts/generate_weights_testnet.sh

[no-exit-message]
format:
  cargo fmt --all

# Check for compilation errors, default to debug mode
[no-exit-message]
check:
  cargo check

# Check code quality with clippy
[no-exit-message]
clippy:
  cargo clippy
