#!/usr/bin/env bash
set -e

generate_account_id() {
  subkey inspect ${2:-} ${3:-} "$SECRET//$1" | grep "Account ID" | awk '{ print $3 }'
}

generate_address() {
  subkey inspect ${2:-} ${3:-} "$SECRET//$1" | grep "SS58 Address" | awk '{ print $3 }'
}

generate_address_and_account_id() {
  ACCOUNT=$(generate_account_id $1 $2 $3)
  ADDRESS=$(generate_address $1 $2 $3)

  printf "//$ADDRESS\n<[u8; 32]>::dehexify(\"${ACCOUNT#'0x'}\").unwrap().unchecked_into(),"
}

AUTHORITIES=""

AUTHORITIES+="(\n"
AUTHORITIES+="$(generate_address_and_account_id grandpa '--scheme ed25519')\n"
AUTHORITIES+="$(generate_address_and_account_id babe '--scheme sr25519')\n"
AUTHORITIES+="$(generate_address_and_account_id im_online '--scheme sr25519')\n"
AUTHORITIES+="$(generate_address_and_account_id authority_discovery '--scheme sr25519')\n"
AUTHORITIES+="),\n"

printf "$AUTHORITIES"
