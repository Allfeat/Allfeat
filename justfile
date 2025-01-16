#!/usr/bin/env just --justfile

set positional-arguments := true

optimized_node_args := "-- --database=paritydb \
  --rpc-methods=unsafe"

# Cargo profile used to execute cargo commands.
CARGO_PROFILE := env("PROFILE", "release")

# Build the node (default to release profile)
[no-exit-message]
build:
  echo "Starting to build Allfeat Node with profile '{{CARGO_PROFILE}}'"
  cargo build --profile {{CARGO_PROFILE}}

# Start the node with default arguments in default mode (Melodie Testnet Live)
[no-exit-message]
start args='': (_start-base "--sync=warp" args)

# Start the node with default arguments in development mode.
[no-exit-message]
start-dev args='': (_start-base "--dev" args)

# Check for compilation errors, default to debug mode
[no-exit-message]
check:
  cargo check

# Check code quality with clippy
[no-exit-message]
clippy:
  cargo clippy

[no-exit-message]
_start-base args0='' args1='':
  cargo run --profile {{CARGO_PROFILE}} {{optimized_node_args}} {{args0}} {{args1}}
