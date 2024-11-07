{
  description = "Development environment flake";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
  }: let
    supportedSystems = ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"];
    forEachSupportedSystem = f:
      nixpkgs.lib.genAttrs supportedSystems (system:
        f {
          pkgs = import nixpkgs {
            inherit system;
            # crossSystem = "x86_64-w64-mingw32";
            overlays = [rust-overlay.overlays.default self.overlays.default];
          };
        });
  in {
    overlays.default = final: prev: {
      rustToolchain = let
        rust = prev.rust-bin;
      in
        if builtins.pathExists ./rust-toolchain.toml
        then rust.fromRustupToolchainFile ./rust-toolchain.toml
        else throw "rust-toolchain.toml file not found. This file is required for the build.";
    };

    devShells = forEachSupportedSystem ({pkgs}: {
      default = pkgs.mkShell rec {
        nativeBuildInputs = with pkgs; [pkg-config];
        buildInputs = with pkgs; [
          rustToolchain
          udev
          alsa-lib
          vulkan-loader
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
          libxkbcommon

          # wayland optional (linux only)
          wayland

          # webgl builds
          binaryen
          wasm-bindgen-cli

          # for packaging the source code
          zip
        ];
        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
      };
    });
  };
}
