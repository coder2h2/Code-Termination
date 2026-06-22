{
  description = "Minimal Rust development environment for Code-Termination";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          targets = [ "x86_64-pc-windows-gnu" ];
        };
        rustPackages = [
          rustToolchain
          pkgs.pkg-config
          pkgs.rust-analyzer
        ];
        xkbConfigRoot = "${pkgs.xkeyboard_config}/share/X11/xkb";
        runtimeLibs = with pkgs; [
          alsa-lib
          libxkbcommon
          udev
          vulkan-loader
          wayland
          libx11
          libxcursor
          libxi
          libxrandr
        ];
        runtimeLibraryPath = pkgs.lib.makeLibraryPath runtimeLibs;
      in
      {
        devShells.default = pkgs.mkShell {
          packages = rustPackages;
        };

        devShells.nixos = pkgs.mkShell {
          packages = rustPackages ++ runtimeLibs ++ [ pkgs.vulkan-tools pkgs.xkeyboard_config pkgs.pkgsCross.mingwW64.stdenv.cc ];

          shellHook = ''
            export XKB_CONFIG_ROOT="${xkbConfigRoot}"
            export LD_LIBRARY_PATH="${runtimeLibraryPath}:$LD_LIBRARY_PATH"
            export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER="x86_64-w64-mingw32-gcc"
          '';
        };
      }
    );
}
