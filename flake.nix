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
          packages = with pkgs; [
            alsa-lib
            cargo
            clippy
            libxkbcommon
            pkg-config
            rust-analyzer
            rustc
            rustfmt
            udev
            vulkan-loader
            vulkan-tools
            wayland
            xkeyboard_config
            libx11
            libxcursor
            libxi
            libxrandr
          ];

          shellHook = ''
            if [ -f /etc/os-release ] && grep -q '^ID=nixos$' /etc/os-release; then
              export XKB_CONFIG_ROOT="${xkbConfigRoot}"
              export LD_LIBRARY_PATH="${runtimeLibraryPath}:$LD_LIBRARY_PATH"
            fi
          '';
        };
      }
    );
}
