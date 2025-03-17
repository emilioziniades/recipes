{
  description = "Recipes served by cook-cli";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = {
    self,
    nixpkgs,
  }: let
    forEachSystem = fn:
      nixpkgs.lib.genAttrs [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ] (system: fn nixpkgs.legacyPackages.${system});
  in {
    packages = forEachSystem (pkgs: {
      container = pkgs.dockerTools.buildLayeredImage {
        name = "cook-server";
        tag = "latest";
        contents = [
          pkgs.cook-cli
          "${self}/recipes"
        ];
        config = {
          Entrypoint = ["cook"];
          Cmd = ["server" "--host"];
          WorkingDir = "${self}/recipes";
          ExposedPorts = {
            "9080" = {};
          };
        };
      };
    });

    devShells = forEachSystem (pkgs: {
      default = pkgs.mkShell {
        buildInputs = with pkgs; [
          cook-cli
        ];
      };
    });
  };
}
