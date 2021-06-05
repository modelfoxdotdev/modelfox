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
    naersk = {
      url = "github:nmattia/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = { nixpkgs, flake-utils, fenix, naersk, ... }: flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs {
        inherit system;
      };
      rustToolchain = (with fenix.packages.${system};
        combine [
          stable.rustc
          latest.cargo
          targets.wasm32-unknown-unknown.stable.rust-std
        ]);
    in rec {
      defaultPackage = packages.tangram;
      packages = {
        tangram = (naersk.lib.${system}.override {
          rustc = rustToolchain;
          cargo = rustToolchain;
        }).buildPackage {
          pname = "tangram";
          src = ./.;
          targets = [ "tangram_cli" ];
          copySources = [ "." ];
          buildInputs = with pkgs; [ lld_12 python3 ];
          CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER = "${pkgs.lld_12}/bin/lld";
        };
      };
      defaultApp = apps.tangram;
      apps = {
        tangram = flake-utils.lib.mkApp {
          drv = packages.tangram;
        };
      };
      devShell = pkgs.mkShell {
        LD_LIBRARY_PATH = "${pkgs.stdenv.cc.cc.lib}/lib";
        buildInputs = with pkgs; [
          createrepo_c
          dpkg
          elixir
          go
          nodejs-16_x
          python39
          rpm
          rust-cbindgen
          ruby
          sequoia
          sqlite
        ];
      };
    }
  );
}
