{
  description = "Yew development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.11";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        # Rustツールチェーン (wasmターゲット付き)
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" ];
          targets = [ "wasm32-unknown-unknown" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            rustToolchain
            trunk             # Yew用WASMビルドツール
            wasm-bindgen-cli # WASMバインディング生成
            binaryen         # WASM最適化ツール
            openssl          # 暗号関連依存
            pkg-config       # ネイティブライブラリ検出
          ];

          # 環境変数設定
          env = {
            RUST_BACKTRACE = "1";
            PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
          };

          # Rustツールチェーンの自動選択
          shellHook = ''
            echo "Rust $(rustc --version)"
            echo "Trunk $(trunk --version)"
          '';
        };
      }
    );
}
