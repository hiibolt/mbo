{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };
  outputs = { self, nixpkgs, rust-overlay, flake-utils }: 
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
        rustVersion = pkgs.rust-bin.stable.latest.default;

        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustVersion;
          rustc = rustVersion;
        };

        backendBuildInputs = with pkgs; [
          (rustVersion.override { extensions = ["rust-src"]; })
          pkg-config
          cargo
          gcc
          rustfmt
          clippy
          openssl.dev
          sqlite
        ];

        frontendBuildInputs = with pkgs; [
          bun
        ];

        sharedBuildInputs = with pkgs; [
          jq
        ];

        backendPackage = rustPlatform.buildRustPackage rec {
          pname = "mbo-backend";
          version = "0.0.1";
          src = ./mbo-backend;
          cargoBuildFlags = "";

          cargoLock = {
            lockFile = ./mbo-backend/Cargo.lock;
          };

          nativeBuildInputs = backendBuildInputs;

          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
          OPENSSL_LIB_DIR = pkgs.openssl.out + "/lib";
        };
      in
      {
        defaultPackage = backendPackage;

        packages = {
          backend = backendPackage;
        };

        devShell = pkgs.mkShell {
          buildInputs = backendBuildInputs ++ frontendBuildInputs ++ sharedBuildInputs;
          
          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
          OPENSSL_LIB_DIR = pkgs.openssl.out + "/lib";
        };
    });
}
