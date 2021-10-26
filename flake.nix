{
  inputs = {
    nixpkgs = {
      url = "github:nixos/nixpkgs/nixos-unstable";
    };
    flake-utils = {
      url = "github:numtide/flake-utils";
    };
    fenix = {
      url = "github:nix-community/fenix";
    };
  };
  outputs = { nixpkgs, flake-utils, fenix, ... }: flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs {
        inherit system;
      };
      rust = (with fenix.packages.${system}; combine [
        latest.rustc
        latest.cargo
        latest.clippy-preview
        latest.rustfmt-preview
        latest.rust-std
        targets.wasm32-unknown-unknown.latest.rust-std
        latest.rust-src
        rust-analyzer
      ]);
    in rec {
      defaultApp = flake-utils.lib.mkApp {
        drv = defaultPackage;
      };
      defaultPackage = (pkgs.makeRustPlatform {
        rustc = rust;
        cargo = rust;
      }).buildRustPackage {
        pname = "tangram";
        version = "0.7.0";
        src = ./.;
        doCheck = false;
        nativeBuildInputs = with pkgs; [
          clang_12
          lld_12
        ];
        cargoSha256 = "sha256-8mX/RmgE3VmoOBGMWnunucXqT+/7HSYDUcp5wempt3M=";
        cargoBuildFlags = [ "--bin" "tangram" ];
        CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER = "clang";
        CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER = "lld";
      };
      devShell = pkgs.mkShell {
        buildInputs = with pkgs; [
          cachix
          cargo-insta
          clang_12
          createrepo_c
          doxygen
          dpkg
          elixir
          go
          lld_12
          mold
          nodejs-16_x
          (php.withExtensions ({ all, ...}: with all; [
            curl
            dom
            ffi
            fileinfo
            filter
            iconv
            mbstring
            simplexml
            tokenizer
          ]))
          php.packages.composer
          (python39.withPackages(ps: with ps; [
            catboost
            lightgbm
            numpy
            pandas
            pytorch
            scikitlearn
            # xgboost
          ]))
          rpm
          ruby
          rust
          rust-cbindgen
          sequoia
          sqlite
          time
          wasm-bindgen-cli
        ];
        CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER = toString ./. + "/scripts/clang";
        CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER = "lld";
      };
    }
  );
}
