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
          (sequoia.overrideAttrs (drv: rec {
            version = "0.25.0";
            src = fetchFromGitLab {
              owner = "sequoia-pgp";
              repo = "sequoia";
              rev = "sq/v${version}";
              sha256 = "13f582g10vba0cpbdmqkkfzgd5jgagb640jaz1w425wf5nbh6q50";
            };
            cargoDeps = drv.cargoDeps.overrideAttrs (lib.const {
              inherit src;
              name = "${drv.pname}-vendor.tar.gz";
              outputHash = "sha256-JJl+HBQrt0zE1L5MQbkQOdUGkpa5h0dWEYV/3+dxci0=";
            });
          }))
          sqlite
        ];
      };
    }
  );
}
