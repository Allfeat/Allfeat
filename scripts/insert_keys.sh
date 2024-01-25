#!/usr/bin/env bash

# $SECRET must include derivation path if any.

./target/release/allfeat key insert --base-path "$NODE_PATH" --chain testnet --scheme Ed25519 --suri "$SECRET//grandpa" --key-type gran
./target/release/allfeat key insert --base-path "$NODE_PATH" --chain testnet --scheme Sr25519 --suri "$SECRET//babe" --key-type babe
./target/release/allfeat key insert --base-path "$NODE_PATH" --chain testnet --scheme Sr25519 --suri "$SECRET//im_online" --key-type imon
./target/release/allfeat key insert --base-path "$NODE_PATH" --chain testnet --scheme Sr25519 --suri "$SECRET//authority_discovery" --key-type audi

printf "Success."
