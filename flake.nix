{
  description = "A Nix-flake-based Rust development environment";

  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forEachSupportedSystem =
        f:
        inputs.nixpkgs.lib.genAttrs supportedSystems (
          system:
          f {
            pkgs = import inputs.nixpkgs {
              inherit system;
              overlays = [
                inputs.rust-overlay.overlays.default
                inputs.self.overlays.default
              ];
            };
          }
        );
    in
    {
      overlays.default = final: prev: {
        rustToolchain =
          let
            rust = prev.rust-bin;
          in
          if builtins.pathExists ./rust-toolchain.toml then
            rust.fromRustupToolchainFile ./rust-toolchain.toml
          else if builtins.pathExists ./rust-toolchain then
            rust.fromRustupToolchainFile ./rust-toolchain
          else
            rust.stable.latest.default.override {
              extensions = [
                "rust-src"
                "rustfmt"
              ];
            };
      };
      packages = forEachSupportedSystem (
        { pkgs }:
        let
          runtime-stuff = with pkgs; [
            vulkan-loader
            libGL
            libxkbcommon
            wayland
          ];
          libpath = pkgs.lib.makeLibraryPath runtime-stuff;
        in
        rec {
          client = pkgs.rustPlatform.buildRustPackage {
            pname = "client";
            version = "0.1.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            buildInputs = runtime-stuff;
            nativeBuildInputs = [ pkgs.makeWrapper ];
            postInstall = ''
              wrapProgram $out/bin/client --set LD_LIBRARY_PATH ${libpath}
            '';
          };
          default = client;
          server = pkgs.rustPlatform.buildRustPackage {
            pname = "server";
            version = "0.1.0";
            src = ./.;
            cargoBuildFlags = [
              "--package"
              "server"
            ];
            cargoLock.lockFile = ./Cargo.lock;
            buildInputs = runtime-stuff;
          };

        }
      );

      devShells = forEachSupportedSystem (
        { pkgs }:
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              rustToolchain
              openssl
              pkg-config
              cargo-deny
              cargo-edit
              cargo-watch
              rust-analyzer
            ];

            env =
              let
                libPath =
                  with pkgs;
                  lib.makeLibraryPath [
                    libGL
                    libxkbcommon
                    wayland
                    vulkan-tools
                    vulkan-loader
                    vulkan-validation-layers
                    vulkan-extension-layer
                  ];
              in
              {
                # Required by rust-analyzer
                LD_LIBRARY_PATH = libPath;
                RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";
              };
          };
        }
      );
    };
}
