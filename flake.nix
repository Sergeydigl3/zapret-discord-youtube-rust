{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    naersk.url = "github:nix-community/naersk";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, naersk, flake-utils, ... }:
  let
    baseModule = import ./nix/nixos-module.nix;
  in
    flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs { inherit system; };
      naerskLib = pkgs.callPackage naersk {};

      zapret = naerskLib.buildPackage {
        src = ./.;
        nativeBuildInputs = [ pkgs.makeWrapper ];
        postInstall = ''
          wrapProgram $out/bin/zapret-rust \
            --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.nftables pkgs.polkit ]}
        '';
        meta = {
          description = "Zapret-Rust TUI for DPI bypass";
          homepage = "https://github.com/Sergeydigl3/zapret-discord-youtube-rust";
          license = pkgs.lib.licenses.mit;
          mainProgram = "zapret-rust";
          platforms = pkgs.lib.platforms.linux;
        };
      };
    in {
      packages.default = zapret;

      devShells.default = pkgs.mkShell {
        buildInputs = with pkgs; [
          cargo rustc rustfmt clippy rust-analyzer
          nftables polkit
        ];

        env.RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
      };
    }) // {
      nixosModules.zapret-rust-base = baseModule;

      nixosModules.zapret-rust = { config, lib, pkgs, ... }: {
        imports = [ baseModule ];
        services.zapret-rust.package = lib.mkDefault (
          self.packages.${pkgs.stdenv.hostPlatform.system}.default
        );
      };

      nixosModule = self.nixosModules.zapret-rust;
    };
}
