{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        psvm = pkgs.rustPlatform.buildRustPackage rec {
          pname = "psvm";
          version = "febe87df7e01ffb842853e1777b6519b933d0565";

          buildInputs = with pkgs; [ openssl ];

          nativeBuildInputs = with pkgs; [ pkg-config ];

          useFetchCargoVendor = true;

          # Tests rely on network.
          doCheck = false;

          src = pkgs.fetchFromGitHub {
            owner = "paritytech";
            repo = pname;
            rev = version;
            hash = "sha256-QKIr+2fqaysj+7EL/OBWhLCeD8HxgzpKaRAXfczEtM4=";
          };

          cargoHash = "sha256-fG9h//7YuRigbvNmI5+dxDvk//sz1peN9ppHcj9lMGc=";
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
              psvm
            ]
            ++ lib.optionals stdenv.hostPlatform.isLinux [ rust-jemalloc-sys-unprefixed ]
            ++ lib.optionals stdenv.hostPlatform.isDarwin [
              darwin.apple_sdk.frameworks.Security
              darwin.apple_sdk.frameworks.SystemConfiguration
            ];

          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
          # New flag required since https://github.com/eigerco/polka-storage/pull/730
          CRATE_CC_NO_DEFAULTS = 1;
        };
      }
    );
}
