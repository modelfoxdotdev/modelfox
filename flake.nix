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
        targets.wasm32-unknown-unknown.stable.rust-std
      ]);
    in rec {
      defaultApp = flake-utils.lib.mkApp {
        drv = (pkgs.makeRustPlatform {
          rustc = rust;
          cargo = rust;
        }).buildRustPackage {
          buildInputs = with pkgs; [
            python39
          ];
          pname = "tangram";
          src = ./.;
          cargoSha256 = pkgs.lib.fakeSha256;
        };
      };
      devShell = pkgs.mkShell {
        CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER = toString ./. + "/scripts/clang";
        buildInputs = with pkgs; [
          (pkgs.stdenv.mkDerivation {
            name = "mold";
            src = pkgs.fetchgit {
              url = "https://github.com/rui314/mold";
              rev = "1317132548b1b7fb6d495428cc04ad00c9dd37e4";
              sha256 = "sha256-dHEjjojO7PS9A7iW7tz1GlLYGGIvmrUii6sgVVz7CDo=";
              fetchSubmodules = true;
            };
            nativeBuildInputs = with pkgs; [ clang_12 cmake lld_12 tbb xxHash zlib openssl git ];
            dontUseCmakeConfigure = "true";
            buildPhase = "make -j24";
            patches = [ ./scripts/mold.patch ];
            installPhase = "mkdir -p $out $out/bin $out/share/man/man1 && PREFIX=$out make install";
          })
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
        ];
      };
    }
  );
}
