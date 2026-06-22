{
  pkgs ? import <nixpkgs> { },
}:

pkgs.mkShell {
  buildInputs = [
    pkgs.cargo-llvm-cov
    pkgs.libGL
    pkgs.libxkbcommon
    pkgs.mesa
    pkgs.openssl
    pkgs.pkg-config
    pkgs.rust-analyzer
    pkgs.rustup
    pkgs.seatd
    pkgs.udev
    pkgs.wayland
    pkgs.wayland-protocols

    pkgs.xorg.libX11
    pkgs.xorg.libXcursor
    pkgs.xorg.libXi
    pkgs.xorg.libXrandr
    pkgs.xorg.libxcb
  ];

  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
    pkgs.libGL
    pkgs.libxkbcommon
    pkgs.mesa
    pkgs.openssl
    pkgs.seatd
    pkgs.udev
    pkgs.wayland

    pkgs.xorg.libX11
    pkgs.xorg.libXcursor
    pkgs.xorg.libXi
    pkgs.xorg.libXrandr
    pkgs.xorg.libxcb
  ];
}
