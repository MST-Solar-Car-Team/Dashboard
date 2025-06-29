{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  packages = [ pkgs.gdb pkgs.opencv pkgs.llvmPackages.libclang.lib];
  pkgs.opencv.overrideAttrs  =(finalAttrs: { 
              enableJPEG = false;
              enablePNG = false;
              enableTIFF = false;
              enableWebP = false;
              enableJPEG2000 = false;
              enableEigen = false;
              enableBlas = false;
              enableVA = false;
              enableContrib = false;

              enableCuda = false;
              enableCublas = false;
              enableCudnn = false; # NOTE: CUDNN has a large impact on closure size so we disable it by default
              enableCufft = false;

              enableLto = false;
              enableFfmpeg = false;
              enableGStreamer = true;
          });  
  buildInputs = with pkgs;[
    stdenv
    rustc cargo rust.packages.stable.rustPlatform.rustLibSrc
    udev pkg-config libxkbcommon
    rust-analyzer
    clang llvm llvmPackages.libclang lld 
    opencv

          
     ];
          


  
  
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
    # export RUST_SRC_PATH="${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
    #export LIBCLANG_PATH="${pkgs.llvmPackages.libclang}/lib";
    export LIBCLANG_PATH="${pkgs.llvmPackages.libclang.lib}/lib";

    export RUST_SRC_PATH="${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
    '';

}
