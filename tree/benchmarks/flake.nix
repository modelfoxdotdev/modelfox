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
          cargo
          rustc
          time
          (python38.withPackages(p: with p; [
            catboost
            lightgbm
            numpy
            pandas
            scikitlearn
            xgboost
          ]))
        ];
      };
    }
  );
}
