{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let
      overlays = [
        rust-overlay.overlays.default
        (final: prev: {
          rustToolchain =
            prev.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        })
      ];
      supportedSystems =
        [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forEachSupportedSystem = f:
        nixpkgs.lib.genAttrs supportedSystems
        (system: f { pkgs = import nixpkgs { inherit overlays system; }; });
    in {
      devShells = forEachSupportedSystem ({ pkgs }: {
        default = pkgs.mkShell {
          packages = with pkgs;
            [
              git
              rustToolchain
              openssl
              protobuf
              llvmPackages.libclang
              pkg-config
              rust-analyzer
            ] ++ lib.optional
            (system == "x86_64-darwin" || system == "aarch64-darwin")
            darwin.apple_sdk.frameworks.SystemConfiguration;
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          BINDGEN_EXTRA_CLANG_ARGS =
            "-isystem ${pkgs.llvmPackages.libclang.lib}/lib/clang/17/include";
        };
      });

      packages = forEachSupportedSystem ({ pkgs, ... }: {
        default = pkgs.stdenv.mkDerivation {
          pname = "allfeat";
          version = "0.1.0";
          src = ./.;

          buildInputs = with pkgs;
            [
              rustToolchain
              cacert
              llvmPackages.libclang
              openssl
              protobuf
              pkg-config
            ] ++ lib.optional
            (system == "x86_64-darwin" || system == "aarch64-darwin")
            darwin.apple_sdk.frameworks.SystemConfiguration;

          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          BINDGEN_EXTRA_CLANG_ARGS =
            "-isystem ${pkgs.llvmPackages.libclang.lib}/lib/clang/17/include";

          buildPhase = ''
            export CARGO_HOME=$TMPDIR/cargo
            export RUSTUP_HOME=$TMPDIR/rustup
            cargo build --locked --release
          '';

          installPhase = ''
            mkdir -p bin/
            cp target/release/allfeat bin/
          '';

          # specify the content hash of this derivations output
          outputHashAlgo = "sha256";
          outputHashMode = "recursive";
          outputHash = "sha256-Om4BcXK76QrExnKcDzw574l+h75C8yK/EbccpbcvLsQ=";
        };
      });
    };
}
