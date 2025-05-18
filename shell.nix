{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  packages = [ pkgs.gdb ];
  buildInputs = with pkgs;[ stdenv rustc cargo udev pkg-config libxkbcommon ];

  # { pkgs ? import <nixpkgs> {} }:  
  
# pkgs.mkShell {  
  # buildInputs = with pkgs; [  
  #   xorg.libX11  
  #   xorg.libXcursor  
  #   xorg.libXrandr  
  #   xorg.libXinerama  
  #   xorg.libXi  
  #   xorg.libXext  
  #   xorg.libXft  
  #   xorg.libXrender  
  #   mesa  
  #   mesa.drivers  

  #   stdenv
  #   rustc
  #   cargo
  #   pkg-config
  # ];  


  nativeBuildInputs = [ pkgs.fontconfig ];
  shellHook = ''
    # export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:${pkgs.wayland}/lib:${pkgs.libxkbcommon}/lib
    export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:${pkgs.xorg.libX11}/lib:${pkgs.xorg.libXcursor}/lib:${pkgs.xorg.libXrandr}/lib:${pkgs.xorg.libXi}/lib:${pkgs.libxkbcommon}/lib
    export WINIT_UNIX_BACKEND=X11
    '';
}