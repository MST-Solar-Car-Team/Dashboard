{
  description = "Cross-compiling a Rust project to aarch64-linux";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        target = "aarch64-unknown-linux-gnu";

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          targets = [ target ];
        };

        # crossPkgs = import nixpkgs {
        #   system = "x86_64-linux"; # Assuming you're building on x86_64
        #   crossSystem = {
        #     config = target;
        #   };
        #   overlays = [ (import rust-overlay) ];
        # };
        #
        crossPkgs = pkgs.pkgsCross.aarch64-multiplatform;

      in {
        packages.default = crossPkgs.stdenv.mkDerivation {
          pname = "dashboard";
          version = "0.1.0";

          src = ./.;

          nativeBuildInputs = with crossPkgs; [
            rustToolchain
            pkg-config
            crossPkgs.stdenv.cc
          ];

          buildInputs = with crossPkgs; [
            openssl
          ];

          CARGO_BUILD_TARGET = target;

          buildPhase = ''
            cargo build --release --target ${target}
          '';

          installPhase = ''
            mkdir -p $out/bin
            cp target/${target}/release/dashboard $out/bin/
          '';


          # Ensure the .cargo/config.toml exists
          postPatch = ''
            mkdir -p .cargo
            cat > .cargo/config.toml <<EOF
          [target.aarch64-unknown-linux-gnu]
          linker = "${crossPkgs.stdenv.cc.targetPrefix}gcc"
          EOF
          '';
        };
      });
}
