{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.11";
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    crane,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      craneLib = crane.lib.${system};
      pkgs = import nixpkgs {inherit system;};
    in {
      packages.default = craneLib.buildPackage {
        pname = "edlearn_tui";
        version = "0.0.1";

        src = craneLib.cleanCargoSource (craneLib.path ./.);
        doCheck = false;
        buildInputs = with pkgs; [pkg-config openssl];
      };
      devShells.default = pkgs.mkShell {
        buildInputs = with pkgs; [openssl.dev pkg-config gnumake];
      };
    });
}
