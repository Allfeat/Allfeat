# Thin Allfeat (mainnet runtime) dev-node image: the official
# polkadot-omni-node image plus a dev chain spec generated from the runtime
# wasm built at the same commit. Nothing is compiled in here — see
# .github/workflows/build-push-dev-image.yml, which builds the wasm,
# generates allfeat-dev.json and passes it as context.
#
# The base image tag must stay in the same polkadot-sdk band as the workspace
# umbrella crate, so the node provides every host function the runtime expects.
ARG OMNI_NODE_IMAGE=docker.io/parity/polkadot-omni-node:stable2603-3
FROM ${OMNI_NODE_IMAGE}

COPY allfeat-dev.json /specs/allfeat-dev.json

EXPOSE 9944 9615

# Manual-seal dev chain, no relay chain involved — the parachain equivalent of
# the old `allfeat --dev`. State is ephemeral (--tmp): override the args with
# --base-path on a volume to persist it across restarts.
CMD ["--chain", "/specs/allfeat-dev.json", "--dev-block-time", "6000", "--tmp", "--rpc-external", "--rpc-cors=all"]
