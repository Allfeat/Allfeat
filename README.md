<div align="center">

![logo](docs/logo.svg)

[![Check Build](https://github.com/allfeat/allfeat/actions/workflows/check-node.yml/badge.svg)](https://github.com/allfeat/allfeat/actions/workflows/check-node.yml)
[![Runtime Checks](https://github.com/allfeat/allfeat/actions/workflows/check-melodie-runtime.yml/badge.svg)](https://github.com/allfeat/allfeat/actions/workflows/check-melodie-runtime.yml)
[![Format](https://github.com/allfeat/allfeat/actions/workflows/check-format.yml/badge.svg)](https://github.com/allfeat/allfeat/actions/workflows/check-format.yml)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

[allfeat.org](https://allfeat.org) · [discord.allfeat.org](https://discord.allfeat.org)

</div>

---

# Allfeat

## Vision

Allfeat delivers a music-native blockchain where creators certify, protect, and monetise metadata without intermediaries. The network is preparing its mainnet launch and this repository captures everything required to ship production-grade runtimes and node software.

## What It Offers

- **Metadata integrity**: MIDDS (Metadata Integrity, Decentralisation & Distribution System) anchors music metadata on-chain for transparent provenance.
- **ATS protection-first**: Asset Trusted Shield enforces artist-controlled access policies and notarises works before distribution to curb misuse.
- **Creator-first economics**: On-chain incentives and allocation logic distribute value between artists, contributors, and stakeholders.
- **Operational readiness**: Dual runtimes (Melodie Testnet & upcoming Mainnet) and a hardened node client built on the latest Polkadot SDK.

## Technical Overview

- **Foundation**: Rust workspace leveraging Substrate/Polkadot SDK, optimised for deterministic runtime builds and lightweight node execution.
- **Runtimes**: `runtime/melodie` powers the Harmonie testnet; `runtime/mainnet` contains the production configuration. Shared logic lives in `runtime/shared`.
- **Custom pallets**: Bespoke modules such as `pallet-midds`, `pallet-validators`, `pallet-token-allocation`, and ATS provide Allfeat-specific business logic and governance rules.
- **Node client**: The `node/` crate hosts CLI, RPC, consensus wiring (Aura/Grandpa), and chain-spec plumbing with runtime selection at launch time.

## Repository Structure

- `node/`: Executable node, CLI, chain specs, service wiring, benchmarking glue.
- `runtime/`: Melodie & Mainnet runtimes plus shared constants, ensuring deterministic WASM builds.
- `pallets/`: Domain-specific pallets with unit tests, benchmarking stubs, and weight templates in `.maintain/`.
- `primitives/`: Shared type aliases (`AccountId`, `Balance`, etc.) used across runtimes and client code.
- `scripts/`: Operational tooling for key rotation, validator setup, benchmarking, and testnet preparation.
- `helm/`: Kubernetes Helm chart for deploying validator or node instances.
- `docs/`: Developer documentation, including environment bootstrap instructions (`docs/rust-setup.md`).

## Getting Started

1. **Install prerequisites**
   - Follow `docs/rust-setup.md` or launch the Nix development shell: `nix develop`.
   - Ensure `wasm32-unknown-unknown` target and `subkey` are available (provided via the flake).
2. **Clone & enter workspace**
   ```bash
   git clone https://github.com/allfeat/allfeat.git
   cd allfeat
   ```
3. **Bootstrap dependencies**
   - Optional: `just` is recommended (installed via Nix shell).
   - Environment variable `PROFILE` controls build profile (defaults to `release`).

## Build & Test Workflows

- **Build node**: `just build` (`cargo build --profile $PROFILE`).
- **Run node (warp sync)**: `just start`. For a local dev chain: `just start-dev`.
- **Compile runtimes**: `just build-melodie` or `cargo build --profile production -p melodie-runtime --features on-chain-release-build`.
- **Unit tests**: `cargo test --workspace`. Target pallets individually with `cargo test -p pallet-midds`.
- **Linting**: `just format` (rustfmt), `just clippy`.
- **Static checks**: `just check` (`cargo check`).

## Runtime Artifacts & Release Process

- Deterministic WASM builds are produced via the `release-build-srtool-runtime.yml` workflow using `srtool` with `on-chain-release-build` features.
- Runtime weights and benchmarks rely on templates stored in `.maintain/`. Update weights before tagging a runtime release.
- Mainnet weights pipeline: `./scripts/generate_weights_mainnet.sh` (or `just benchmark-weights-mainnet`).
- Testnet weights pipeline: `./scripts/generate_weights_testnet.sh` (or `just benchmark-weights-testnet`).
- Production binaries are compiled in CI (`release-build-node.yml`) for x86_64 and aarch64 and distributed through releases S3 hosted by OVH.

## Operational Tooling

- `scripts/prepare_testnet.sh`: Generates authority keys using `subkey` (requires `SECRET` env var).
- `scripts/rotate_node_keys.sh`, `scripts/setup_validator_keys.sh`: Support validator lifecycle management.
- `scripts/generate_weights_mainnet.sh`, `scripts/generate_weights_testnet.sh`: End-to-end weights generation with build checks and timestamped logs in `target/weight-logs/`.
- Helm chart (`helm/`) encapsulates Kubernetes deployment defaults for validators and RPC nodes.

## Contributing & Governance

- Contributions follow GPLv3 licensing. Please open issues or pull requests against `master` with CI passing.
- Keep `Cargo.lock` up to date and regenerate runtime weights when modifying pallets impacting extrinsics or fee models.
- Security-related disclosures should be reported privately to `security@allfeat.com` (see repository SECURITY policy if present).

## License

Allfeat is released under the [GNU General Public License v3.0](LICENSE).
