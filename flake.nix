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
        ];
      };
    }
  );
}

# (let python = python39; in python.withPackages (p: with p; [
#   pip
#   wheel
#   setuptools
#   setuptools-rust
#   (python.pkgs.buildPythonPackage rec {
#     pname = "pdoc";
#     version = "6.6.0";
#     format = "wheel";
#     doCheck = false;
#     propagatedBuildInputs = [
#       jinja2
#       pygments
#     ];
#     src = python.pkgs.fetchPypi {
#       inherit pname version format;
#       python = "py3";
#       sha256 = "d0b8cfe21fbdd243feaad133a40c1af3e98c2333f5fc2f221144961ec2d2c491";
#     };
#   })
# ]))
