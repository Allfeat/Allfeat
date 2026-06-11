{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    polkadot.url = "github:andresilva/polkadot.nix";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      polkadot,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [
          (import rust-overlay)
          polkadot.overlays.default
        ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # polkadot-omni-node depends on wasm-opt-sys, whose build script probes
        # `-std=c++17` through the `cc` crate. cc only accepts a flag when the
        # test compile leaves stderr empty (cc-rs: `status.success() &&
        # stderr.is_empty()`). On aarch64-darwin the crate passes
        # `--target=aarch64-apple-darwin`, and the nixpkgs clang wrapper prints a
        # warning to stderr because its canonical triple is `arm64-apple-darwin`.
        # That stderr noise makes cc report c++17 as unsupported and the build
        # aborts. Suppressing the warning keeps the probe's stderr clean.
        polkadot-omni-node = pkgs.polkadot-omni-node.overrideAttrs (_: {
          NIX_CC_WRAPPER_SUPPRESS_TARGET_WARNING = 1;
        });
        frame-omni-bencher = pkgs.frame-omni-bencher.overrideAttrs (_: {
          NIX_CC_WRAPPER_SUPPRESS_TARGET_WARNING = 1;
        });

      in
      {
        devShells.default = pkgs.mkShell {
          packages =
            with pkgs;
            [
              (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
              clang
              protobuf
              openssl
              pkg-config
              just
              nodejs-slim

              # Polkadot SDK
              try-runtime-cli
              polkadot-omni-node
              frame-omni-bencher
            ]
            ++ lib.optionals stdenv.hostPlatform.isLinux [ rust-jemalloc-sys-unprefixed ];

          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
          # New flag required since https://github.com/eigerco/polka-storage/pull/730
          CRATE_CC_NO_DEFAULTS = 1;
        };
      }
    );
}
