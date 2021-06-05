{
  inputs = {
    nixpkgs = {
      url = "github:nixos/nixpkgs/master";
    };
    flake-utils = {
      url = "github:numtide/flake-utils";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = inputs: inputs.flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import inputs.nixpkgs {
        inherit system;
      };
    in {
      devShell = pkgs.mkShell {
        buildInputs = with pkgs; [
          (python38.withPackages(ps: with ps; [
            numpy
            pandas
            pytorch
            scikitlearn
          ]))
          time
        ];
      };
    }
  );
}
