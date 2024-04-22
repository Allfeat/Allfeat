#!/bin/sh

PROFILE=${1:-ci}

for p in allfeat
do
  cargo clean -p ${p} --profile ${PROFILE} 2> /dev/null || true
done

# ---

for r in allfeat harmonie
do
  rm -rf target/${PROFILE}/wbuild/${r}-runtime 2> /dev/null || true
done

# for r in polkadot kusama westend rococo
# do
#   rm -rf target/${PROFILE}/wbuild/${r}-runtime/target/release 2> /dev/null || true
# done