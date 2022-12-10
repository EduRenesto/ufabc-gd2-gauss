{
  inputs = {
    cargo2nix.url = "github:cargo2nix/cargo2nix";
    flake-utils.follows = "cargo2nix/flake-utils";
    nixpkgs.follows = "cargo2nix/nixpkgs";
  };

  outputs = inputs: with inputs;
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [cargo2nix.overlays.default];
        };

        rustPkgs = pkgs.rustBuilder.makePackageSet {
          rustVersion = "1.61.0";
          packageFun = import ./Cargo.nix;
          packageOverrides = pkgs: pkgs.rustBuilder.overrides.all ++ [
            (pkgs.rustBuilder.rustLib.makeOverride {
              name = "expat-sys";
              overrideAttrs = drv: {
                nativeBuildInputs = drv.nativeBuildInputs or [] ++ (with pkgs; [
                  cmake
                ]);
              };
            })

            (pkgs.rustBuilder.rustLib.makeOverride {
              name = "freetype-sys";
              overrideAttrs = drv: {
                nativeBuildInputs = drv.nativeBuildInputs or [] ++ (with pkgs; [
                  cmake
                ]);
              };
            })

            (pkgs.rustBuilder.rustLib.makeOverride {
              name = "servo-fontconfig-sys";
              overrideAttrs = drv: {
                nativeBuildInputs = drv.nativeBuildInputs or [] ++ (with pkgs; [
                  fontconfig
                ]);
              };
            })
          ];
        };
      in rec {
        packages = {
          gauss = (rustPkgs.workspace.gauss {
            propagatedBuildInputs = with pkgs; [ xlibswrapper ];
          }).bin;
          default = packages.gauss;
        };

        devShells.default = rustPkgs.workspaceShell {
          packages = with pkgs; [
            rust-analyzer
            glslang

            xorg.libX11
            xorg.libXcursor
            xorg.libXrandr
            xorg.libXi

            cmake
            fontconfig

            SDL2
          ];
          LD_LIBRARY_PATH = "${pkgs.xorg.libX11.out}/lib:${pkgs.xorg.libXcursor.out}/lib:${pkgs.xorg.libXrandr.out}/lib:${pkgs.xorg.libXi.out}/lib";
        };
      }
    );
}
