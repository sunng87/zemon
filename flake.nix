{
  description = "A Rust system monitor TUI application";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, fenix, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ fenix.overlays.default ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.fenix.stable.withComponents [
          "cargo"
          "rustc"
          "rust-src"
          "rustfmt"
          "clippy"
          "rust-analyzer"
        ];

        buildInputs = with pkgs; [
          # Add any system dependencies your app needs here
        ];

        nativeBuildInputs = with pkgs; [
          pkg-config
        ];

        cargoToml = pkgs.lib.importTOML ./Cargo.toml;

      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = buildInputs ++ [ rustToolchain ];
          inherit nativeBuildInputs;

          shellHook = ''
            echo "Rust development environment loaded"
            echo "Rust version: $(rustc --version)"
            echo "Cargo version: $(cargo --version)"
          '';
        };

        packages.default = pkgs.rustPlatform.buildRustPackage rec {
          pname = cargoToml.package.name;
          version = cargoToml.package.version;

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];
          inherit buildInputs;

          meta = with pkgs.lib; {
            description = cargoToml.package.description;
            license = licenses.mit;
            maintainers = [ ];
          };
        };

        apps.default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/zemon";
        };
      });
}
