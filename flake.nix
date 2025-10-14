{
  description = "Recipes served by cook-cli";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs =
    {
      self,
      nixpkgs,
    }:
    let
      forEachSystem =
        fn:
        nixpkgs.lib.genAttrs [
          "x86_64-linux"
          "aarch64-linux"
          "x86_64-darwin"
          "aarch64-darwin"
        ] (system: fn system nixpkgs.legacyPackages.${system});
      tag = self.rev or "dirty";
      port = "8080";
    in
    {
      packages = forEachSystem (
        system: pkgs: {
          container = pkgs.dockerTools.buildLayeredImage {
            name = "cook-server";
            tag = tag;
            contents = [
              pkgs.cook-cli
              ./recipes
            ];
            config = {
              Entrypoint = [ "cook" ];
              Cmd = [
                "server"
                "--host"
                "--port"
                port
              ];
              WorkingDir = ./recipes;
              ExposedPorts = {
                port = { };
              };
            };
          };

          cooklint = pkgs.rustPlatform.buildRustPackage {
            name = "cooklint";
            src = ./cooklint;
            cargoHash = "sha256-UFMb4i+F87wbpAxe2MSTm53k+UfWSz1sMA4oSbF1cko=";
          };
        }
      );

      devShells = forEachSystem (
        system: pkgs: {
          default = pkgs.mkShell {
            buildInputs = with pkgs; [
              cook-cli
              skopeo
              flyctl
              just
            ];
          };
          lint = pkgs.mkShell {
            buildInputs = with pkgs; [
              just
              self.packages.${system}.cooklint
            ];
          };
        }
      );
    };
}
