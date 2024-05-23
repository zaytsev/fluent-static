{
  description = "Basic Rust dev environment";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    flake-compat = {
      url = github:edolstra/flake-compat;
      flake = false;
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    flake-compat,
    fenix,
  }:
    flake-utils.lib.eachDefaultSystem
    (
      system: let
        overlays = [fenix.overlays.default];
        pkgs = import nixpkgs {inherit system overlays;};

        projectConfig = builtins.fromTOML (builtins.readFile ./Cargo.toml);

        package = let
          toolchain = fenix.packages.${system}.stable.toolchain;
        in
          (pkgs.makeRustPlatform {
            cargo = toolchain;
            rustc = toolchain;
          })
          .buildRustPackage {
            pname = projectConfig.package.name;
            version = projectConfig.package.version;
            src = ./.;

            doCheck = false;

            nativeBuildInputs = with pkgs; [
            ];

            buildInputs = with pkgs; [
            ];

            cargoLock = {
              lockFile = ./Cargo.lock;
            };
          };
      in {
        packages = flake-utils.lib.flattenTree {
          main = package;
        };
        defaultPackage = package;

        devShell =
          pkgs.mkShell
          {
            buildInputs = with pkgs; [
              (
                with fenix.packages.${system};
                  combine [
                    stable.rustc
                    stable.cargo
                    stable.rust-src
                    stable.rustfmt
                  ]
              )
              cargo-edit
              cargo-update
              cargo-geiger
              cargo-outdated
              cargo-audit

              cocogitto
            ];
          };
      }
    );
}
