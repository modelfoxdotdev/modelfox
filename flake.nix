{
  inputs = {
    nixpkgs = {
      url = "github:nixos/nixpkgs/nixos-unstable";
    };
    flake-utils = {
      url = "github:numtide/flake-utils";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = { nixpkgs, flake-utils, fenix, ... }: flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs {
        inherit system;
      };
      rust = (with fenix.packages.${system}; combine [
        stable.rustc
        stable.cargo
        targets.aarch64-apple-darwin.stable.rust-std
        targets.aarch64-unknown-linux-gnu.stable.rust-std
        targets.wasm32-unknown-unknown.stable.rust-std
        targets.x86_64-apple-darwin.stable.rust-std
      ]);
    in rec {
      defaultApp = flake-utils.lib.mkApp {
        drv = (pkgs.makeRustPlatform {
          rustc = rust;
          cargo = rust;
        }).buildRustPackage {
          CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER = toString ./. + "/scripts/linker/x86_64-unknown-linux-gnu/clang";
          CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER = "lld";
          buildInputs = with pkgs; [
            clang_12
            lld_12
            python39
          ];
          pname = "tangram";
          src = ./.;
          cargoSha256 = pkgs.lib.fakeSha256;
        };
      };
      devShell = pkgs.mkShell {
        AR_aarch64_unknown_linux_gnu = toString ./. + "/scripts/zar";
        CC_aarch64_unknown_linux_gnu = toString ./. + "/scripts/zcc";
        CXX_aarch64_unknown_linux_gnu = toString ./. + "/scripts/zxx";
        CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER = toString ./. + "/scripts/zcc";
        CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER = toString ./. + "/scripts/clang";
        CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER = "lld";
        buildInputs = with pkgs; [
          clang_12
          createrepo_c
          dpkg
          elixir
          go
          lld_12
          nodejs-16_x
          python39
          rpm
          ruby
          rust
          rust-cbindgen
          sequoia
          sqlite
          zig
        ];
      };
    }
  );
}
