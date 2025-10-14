build:
    nix build .#container

build-darwin:
    nix build --builders 'linux-builder x86_64-linux /etc/nix/builder_ed25519' .#packages.x86_64-linux.container

load:
    docker load < result

run-darwin: build-darwin load
    docker run -p 8080:8080 cook-server:dirty

push-image:
    skopeo copy --dest-creds x:"$FLY_ACCESS_TOKEN" docker-archive:./result "docker://$REGISTRY:$TAG"

deploy:
    flyctl deploy -i $REGISTRY:$TAG

just lint:
    cooklint --dir ./recipes
