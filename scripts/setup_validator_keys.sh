#!/usr/bin/env bash

# We generate a random seed to generate the validator keys to inject.
RANDOM_SECRET=$(cargo run --release -- key generate | grep "Secret phrase" | awk -F': ' '{print $2}' | sed 's/^ *//')

printf "=======================================================================================\n"
printf "KEEP THIS SEED SAFE, IT GIVES CONTROL TO YOUR VALIDATOR\n"
printf "%s\n" "$RANDOM_SECRET"
printf "=======================================================================================\n"

# Assign public keys
GRANDPA_PUBLIC_KEY=$(cargo run --release -- key inspect --scheme ed25519 "$RANDOM_SECRET//grandpa" | grep "Account ID" | awk '{print $3}')
AURA_PUBLIC_KEY=$(cargo run --release -- key inspect --scheme sr25519 "$RANDOM_SECRET//aura" | grep "Account ID" | awk '{print $3}')
IM_ONLINE_PUBLIC_KEY=$(cargo run --release -- key inspect --scheme sr25519 "$RANDOM_SECRET//im_online" | grep "Account ID" | awk '{print $3}')

# NODE_PATH="${NODE_PATH:-/path/to/your/node}"

# Insert keys into the node
./target/release/allfeat key insert --base-path "$NODE_PATH" --scheme Ed25519 --suri "$RANDOM_SECRET//grandpa" --key-type gran
./target/release/allfeat key insert --base-path "$NODE_PATH" --scheme Sr25519 --suri "$RANDOM_SECRET//aura" --key-type aura
./target/release/allfeat key insert --base-path "$NODE_PATH" --scheme Sr25519 --suri "$RANDOM_SECRET//im_online" --key-type imon

printf "Successfully injected the following session keys into your node:\n\n"

printf "Grandpa Public Key: %s\n" "$GRANDPA_PUBLIC_KEY"
printf "Aura Public Key: %s\n" "$AURA_PUBLIC_KEY"
printf "Im_online Public Key: %s\n" "$IM_ONLINE_PUBLIC_KEY"

printf "You can now set these session keys to your node controller account through the Session pallet interface.\n"
