{
  inputs = {
    nixpkgs = {
      url = "github:nixos/nixpkgs/master";
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
