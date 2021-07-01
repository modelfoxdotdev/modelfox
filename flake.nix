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
          pname = "tangram";
          version = "0.4.0";
          src = ./.;
          doCheck = false;
          nativeBuildInputs = with pkgs; [
            clang_12
            lld_12
          ];
          cargoSha256 = "sha256-wV518+LTFRtOE4EYJxZzmXR3k3VvSEvpIa42DB5b+EE=";
          cargoBuildFlags = [ "--bin" "tangram" ];
          CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER = "lld";
        };
      };
      devShell = pkgs.mkShell {
        CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER = toString ./. + "/scripts/clang";
        CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER = "lld";
        buildInputs = with pkgs; [
          (stdenv.mkDerivation {
            pname = "mold";
            version = "0.9.1";
            src = fetchgit {
              url = "https://github.com/rui314/mold";
              rev = "v0.9.1";
              sha256 = "sha256-yIkW6OCXhlHZ1jC8/yMAdJbSgY9K40POT2zWv6wYr5E=";
            };
            nativeBuildInputs = [ clang_12 cmake lld_12 tbb xxHash zlib openssl git ];
            dontUseCmakeConfigure = "true";
            buildPhase = "make -j $NIX_BUILD_CORES";
            installPhase = "mkdir -p $out $out/bin $out/share/man/man1 && PREFIX=$out make install";
          })
          cargo-insta
          clang_12
          createrepo_c
          doxygen
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
