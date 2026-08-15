{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";

    git-hooks.url = "github:cachix/git-hooks.nix";
    git-hooks.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = {
    nixpkgs,
    rust-overlay,
    git-hooks,
    ...
  }: let
    forEachSystem = f:
      nixpkgs.lib.genAttrs ["aarch64-darwin" "x86_64-linux"]
      (system:
        f {
          inherit system;
          pkgs = import nixpkgs {
            inherit system;
            overlays = [(import rust-overlay)];
          };
        });

    # Entering the shell installs the hook, so the checkout needs no setup.
    preCommit = system:
      git-hooks.lib.${system}.run {
        src = ./.;
        hooks.rustfmt.enable = true;
      };
  in {
    formatter = forEachSystem ({pkgs, ...}: pkgs.alejandra);

    checks = forEachSystem ({system, ...}: {
      pre-commit = preCommit system;
    });

    devShells = forEachSystem ({
      pkgs,
      system,
    }: {
      default = pkgs.mkShell {
        inherit (preCommit system) shellHook;

        packages = with pkgs; [
          (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
          bacon
          cargo-edit
        ];
      };
    });
  };
}
