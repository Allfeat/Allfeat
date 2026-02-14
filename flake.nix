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

              # Polkadot SDK
              psvm
              subkey
            ]
            ++ lib.optionals stdenv.hostPlatform.isLinux [ rust-jemalloc-sys-unprefixed ];

          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
          # New flag required since https://github.com/eigerco/polka-storage/pull/730
          CRATE_CC_NO_DEFAULTS = 1;
        };
      }
    );
}
