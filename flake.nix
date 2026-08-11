{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      fenix,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ fenix.overlays.default ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Latest nightly with the complete component set for warnings and Miri
        rust-nightly = pkgs.fenix.complete.toolchain;

        # A known good pinned stable with needed components
        rust-pinned = pkgs.fenix.fromToolchainFile {
          file = ./pinned-toolchain.toml;
          sha256 = "sha256-A1abGIbOtcBSdrUMhDGrER3pRM1hQP4fp9gh3Y4PKc8=";
        };

        # Uses the crate MSRV with minimal profile
        msrv = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.rust-version;
        rust-msrv =
          (pkgs.fenix.toolchainOf {
            channel = msrv;
            sha256 = "sha256-VZZnlyP69+Y3crrLHQyJirqlHrTtGTsyiSnZB8jEvVo=";
          }).minimalToolchain;

        commonTools = with pkgs; [
          just
          cargo-show-asm
          cargo-nextest
          cargo-sort
          cargo-machete
          ripgrep
          jq
          nixd
          nixfmt
        ];

        mkShell =
          rust:
          pkgs.mkShell {
            nativeBuildInputs = [
              pkgs.pkg-config
              pkgs.clang-tools
            ];
            hardeningDisable = [ "fortify" ];
            buildInputs = [ rust ] ++ commonTools;
          };
      in
      {
        devShells = {
          default = mkShell rust-pinned;
          msrv = mkShell rust-msrv;
          nightly = mkShell rust-nightly;
        };
      }
    );
}
