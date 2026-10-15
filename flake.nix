{
  description = "Intern Judge Bevy development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    { nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rustfmt" "clippy" ];
        };

        linuxLibraries = with pkgs; [
          # Bevy default audio and gamepad support
          alsa-lib
          libudev-zero

          # Vulkan and windowing
          vulkan-loader
          libx11
          libxcursor
          libxi
          libxrandr
          libxkbcommon
          wayland
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [
            rustToolchain
            pkgs.pkg-config
          ];

          buildInputs = pkgs.lib.optionals pkgs.stdenv.isLinux linuxLibraries;

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";

          LD_LIBRARY_PATH = pkgs.lib.optionalString pkgs.stdenv.isLinux (
            pkgs.lib.makeLibraryPath linuxLibraries
          );
        };
      }
    );
}
