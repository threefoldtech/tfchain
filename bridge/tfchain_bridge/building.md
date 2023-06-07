# Building the bridge

## local build

This is a normal go project so just execute `go build`.

## Build a docker image

To build a docker image with the latest git tag as version:

```sh
docker build -t tftchainstellarbridge:$(git describe --abbrev=0 --tags | sed 's/^v//')  .
```
