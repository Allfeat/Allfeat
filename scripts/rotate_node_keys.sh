#!/usr/bin/env bash

# Execute the CURL command and capture the response
RESPONSE=$(curl -s -H "Content-Type: application/json" -d '{"id":1, "jsonrpc":"2.0", "method": "author_rotateKeys", "params":[]}' http://localhost:9944)

# Extract the "result" field from the response
RESULT=$(echo "$RESPONSE" | jq -r '.result')

# Check if the hexadecimal string starts with "0x" and has a length of 256 characters (excluding "0x")
if [[ ${#RESULT} -ne 194 ]] || [[ "$RESULT" != 0x* ]]; then
  echo "Error: The hexadecimal string is invalid or does not have the correct length."
  exit 1
fi

# Remove the "0x" prefix for easier splitting
HEX_STRING=${RESULT:2}

# Split the hexadecimal string into 4 parts of 64 characters each
PART1="0x${HEX_STRING:0:64}"
PART2="0x${HEX_STRING:64:64}"
PART3="0x${HEX_STRING:128:64}"

# Print the split parts
echo "Grandpa Public Key: $PART1"
echo "Aura Public Key: $PART2"
echo "ImOnline Public Key: $PART3"
