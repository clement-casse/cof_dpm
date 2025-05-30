{
  description = "Flake for packaging the development and build environment of COF DPM";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };

  outputs = { nixpkgs, utils, rust-overlay, crane, ... }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        inherit (pkgs) lib;

        # Get Rust Toolchain version from ./rust-toolchain.toml
        rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        nativeBuildInputs = with pkgs; [
          pkg-config

          grpc-gateway
          protobuf
          protoc-gen-doc
          protoc-gen-tonic
        ];

        buildInputs = with pkgs; [
          openssl
        ] ++ lib.optionals stdenv.buildPlatform.isDarwin [
          libiconv
        ];

        src = lib.fileset.toSource {
          root = ./.;
          fileset = lib.fileset.unions [
            (craneLib.fileset.commonCargoSources ./.)
            (lib.fileset.fileFilter (
              file:
              lib.any file.hasExt [
                "sql"
                "proto"
              ]
            ) ./.)
          ];
        };

        commonArgs = {
          inherit src nativeBuildInputs buildInputs;
          pname = "cof-dpm-workspace";
          strictDepts = true;
          doCheck = false; # NB: we disable tests since we'll run them all via cargo-nextest
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
      in
      with pkgs;
      {
        formatter = nixpkgs-fmt;

        devShells.default = craneLib.devShell {
          inputsFrom = [ cargoArtifacts ];

          packages = with pkgs; nativeBuildInputs ++ buildInputs ++ [
            buf
            taplo
            sqlx-cli

            cargo-audit
            cargo-binutils
            cargo-nextest
            cargo-tarpaulin
            cargo-watch
          ];

          RUST_LOG = "trace";
          DATABASE_URL = "postgres://postgres:welcome@localhost/database";
        };
      });
}
