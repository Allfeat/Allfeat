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
              cacert
              rustToolchain
              openssl
              protobuf
              pkg-config
              rust-analyzer
            ] ++ lib.optional
            (system == "x86_64-darwin" || system == "aarch64-darwin")
            darwin.apple_sdk.frameworks.SystemConfiguration;
        };
      });

      packages = forEachSupportedSystem ({ pkgs, ... }: {
        default = pkgs.stdenv.mkDerivation {
          pname = "allfeat";
          version = "0.1.0";
          src = ./.;

          buildInputs = with pkgs;
            [ rustToolchain cacert git openssl protobuf pkg-config ]
            ++ lib.optional
            (system == "x86_64-darwin" || system == "aarch64-darwin")
            darwin.apple_sdk.frameworks.SystemConfiguration;

          buildPhase = ''
            export CARGO_HOME=$TMPDIR/cargo
            export RUSTUP_HOME=$TMPDIR/rustup
            cargo build --locked --release
          '';

          installPhase = ''
            mkdir -p bin/
            cp target/release/allfeat bin/
          '';
        };
      });
    };
}
