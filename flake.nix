{
  description = "Interactive Mindmap Portfolio";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
          targets = [ "wasm32-unknown-unknown" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust toolchain
            rustToolchain

            # WebAssembly tools
            wasm-pack
            trunk

            # Development tools
            watchexec
            just

            # System dependencies
            pkg-config
            openssl
          ];

          shellHook = ''
            echo "🦀 Rust WebAssembly development environment"
            echo "📦 Available commands:"
            echo "  justファイル見て"
          '';
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "khimoo-portfolio";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          buildInputs = with pkgs; [ pkg-config openssl ];

          # WebAssembly build
          buildPhase = ''
            cargo build --release
            wasm-pack build --target web --out-dir pkg
          '';
        };
      });
}
