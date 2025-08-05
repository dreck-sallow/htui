{
  description = "A nix-flake htui development environment";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = { nixpkgs, ...}:
  let
    system = "x86_64-linux";
    pkgs = import nixpkgs { inherit system; };
  in {

    devShells.${system}.default  = pkgs.mkShell {
      packages = with pkgs; [ rustc cargo gcc rustfmt clippy rust-analyzer pkg-config openssl];

      shellHook = ''
        exec fish -c zellij a htui
      '';

      env = {
        RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
      };      
    };
  };
}
