{
  inputs = {
    nixpkgs = {
      url = "github:nixos/nixpkgs/nixos-unstable";
    };
    flake-utils = {
      url = "github:numtide/flake-utils";
    };
  };
  outputs = inputs: inputs.flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = inputs.nixpkgs.legacyPackages.${system};
    in {
      devShell = pkgs.mkShell {
        PYO3_NO_PYTHON = "1";
        LD_LIBRARY_PATH = "${pkgs.stdenv.cc.cc.lib}/lib";
        buildInputs = with pkgs; [
          createrepo_c
          dpkg
          elixir
          go
          nodejs-16_x
          python39
          rpm
          ruby
          sequoia
          sqlite
        ];
      };
    }
  );
}
