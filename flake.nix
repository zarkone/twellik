{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  inputs.rust-oxalica  = { url = "github:oxalica/rust-overlay"; };
  inputs.flake-utils.url = "github:numtide/flake-utils";

    outputs = { self, nixpkgs, rust-oxalica, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ rust-oxalica.overlays.default ];
        config = { allowUnfree = true; };
      };
      rust-nightly = pkgs.rust-bin.nightly.latest.default.override {
        targets = [ "wasm32-unknown-unknown" ];
        extensions = [ "rust-analyzer" "rust-src" ];
      };

    in {
      devShell = pkgs.mkShell {
        name = "twellik";
        packages = with pkgs; [ wasm-pack python3 watchexec ];
        nativeBuildInputs = [
          # build
          rust-nightly
          pkgs.wasm-tools
        ];
      };
    });
}
