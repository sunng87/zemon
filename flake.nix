{
  description = "A Rust system monitor TUI application";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rustfmt" "clippy" ];
        };

        buildInputs = with pkgs; [
          # Add any system dependencies your app needs here
        ];

        nativeBuildInputs = with pkgs; [
          pkg-config
        ];

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
          pname = "zemon";
          version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];
          inherit buildInputs;

          meta = with pkgs.lib; {
            description = "A system monitor TUI application";
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
