# Building the bridge with docker

## Build a docker image

To build a docker image with the latest git tag as version:

Note: this assumes you are in this directory (bridge/tfchain_bridge)

```sh
cd ../../
docker build -t tftchainstellarbridge:$(git describe --abbrev=0 --tags | sed 's/^v//')  . -f bridge/tfchain_bridge/Dockerfile
```

## Note

We rely on ../../client/tfchain-go-client in this bridge project. So this is the reason why we need to build this bridge from the root of the repository.