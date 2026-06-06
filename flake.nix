{
  description = "Minimal Rust development environment for gidames";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
        rustPackages = with pkgs; [
          cargo
          clippy
          pkg-config
          rust-analyzer
          rustc
          rustfmt
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
          packages = rustPackages ++ runtimeLibs ++ [ pkgs.vulkan-tools pkgs.xkeyboard_config ];

          shellHook = ''
            export XKB_CONFIG_ROOT="${xkbConfigRoot}"
            export LD_LIBRARY_PATH="${runtimeLibraryPath}:$LD_LIBRARY_PATH"
          '';
        };
      }
    );
}
