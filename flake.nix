{
  description = "Cross-compiling a Rust project to aarch64-linux";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      
    };

  };
      nixConfig = {
        extra-substituters = [
          "https://nix-community.cachix.org"
        ];
        extra-trusted-public-keys = [
          "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
        ];
      };    

  

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [
          (import rust-overlay)
          (final: prev: {
            opencv = prev.opencv.override {
              enableJPEG = false;
              enablePNG = false;
              enableTIFF = false;
              enableWebP = false;
              enableEXR = true;
              enableJPEG2000 = false;
              enableEigen = false;
              enableBlas = false;
              enableVA = true;
              enableContrib = false;

              enableCuda = false;
              enableCublas = false;
              enableCudnn = false; # NOTE: CUDNN has a large impact on closure size so we disable it by default
              enableCufft = false;

              enableLto = false;
              enableFfmpeg = false;
              enableGStreamer = true;
            };
          })
        ];

        # Host packages: used for tooling like cargo, rustc, etc.
        hostPkgs = import nixpkgs {
          inherit system overlays;
        };

        # Cross packages: for the target platform (aarch64)
        crossPkgs = import nixpkgs {
          inherit overlays;
          system = system;
          crossSystem = {
            config = "aarch64-unknown-linux-gnu";
            rustc.config = "aarch64-unknown-linux-gnu";
          };
        };

        target = "aarch64-unknown-linux-gnu";

        rustToolchain = hostPkgs.rust-bin.stable.latest.default.override {
          targets = [ target ];
        };

        rustPlatform = crossPkgs.makeRustPlatform {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };


      in {
        packages.default = rustPlatform.buildRustPackage {
          pname = "dashboard";
          version = "0.1.0";
          src = ./.;

          nativeBuildInputs = [
            crossPkgs.pkg-config
            hostPkgs.pkg-config
            rustToolchain
            crossPkgs.stdenv.cc  # provides aarch64-unknown-linux-gnu-gcc
            hostPkgs.installShellFiles

            # hostPkgs.clang hostPkgs.llvm hostPkgs.llvmPackages.libclang hostPkgs.lld
            crossPkgs.clang crossPkgs.llvm crossPkgs.llvmPackages.libclang crossPkgs.lld
            crossPkgs.clangStdenv
          ];


          buildInputs = with crossPkgs; [
            libgit2
            openssl
            udev
            expat
            fontconfig
            libGL
            xorg.libX11
            xorg.libXcursor
            xorg.libXi
            xorg.libXrandr
            xorg.libxcb

            opencv
            ];

          postFixup = ''
            patchelf $out/bin/dashboard \
              --add-rpath ${
                hostPkgs.lib.makeLibraryPath [
                  crossPkgs.fontconfig
                  crossPkgs.libGL
                  crossPkgs.opencv
                ]
              }
          '';

          cargoDeps = rustPlatform.importCargoLock {
            lockFile = ./Cargo.lock;
          };

          cargoLock.lockFile = ./Cargo.lock;

          # Prevent cargo from building a native binary by default
          buildPhase = ''
            export LIBCLANG_PATH="${hostPkgs.llvmPackages.libclang.lib}/lib";
            cargo build --release --target ${target}
          '';

          # Install only the cross-compiled binary
          installPhase = ''
            mkdir -p $out/bin
            cp target/${target}/release/dashboard $out/bin/
          '';

          doCheck = false;

          # Cross-compilation configuration for cargo
          postPatch = ''
            mkdir -p .cargo
            cat > .cargo/config.toml <<EOF
            [target.${target}]
            linker = "${crossPkgs.stdenv.cc.targetPrefix}gcc"
            EOF
                      '';
        };

        # devShells.${system}.default = hostPkgs.mkShell {

        #   nativeBuildInputs = [
        #     crossPkgs.pkg-config
        #     hostPkgs.pkg-config
        #     rustToolchain
        #     crossPkgs.stdenv.cc  # provides aarch64-unknown-linux-gnu-gcc
        #     hostPkgs.installShellFiles
        #   ];

        #   buildInputs = with crossPkgs; [
        #     # openssl
        #     udev
        #   ];

        #   shellHook = ''
        #     mkdir -p .cargo
        #     cat > .cargo/config.toml <<EOF
        #     [target.${target}]
        #     linker = "${crossPkgs.stdenv.cc.targetPrefix}gcc"
        #     EOF
        #               '';
          

        # };

        # Optional: Alias under the actual target system
        packages.${target} = self.outputs.packages.${system}.default;
      });
}

