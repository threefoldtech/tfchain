# Tfchain substrate Node

## Structure

- Node: defines the way the node communicates with other nodes.
- Pallet: defines runtime behaviour, these are modules that can work together or independantly.
- Runtime: The runtime determines whether transactions are valid or invalid and is responsible for handling changes to the blockchain's state transition function. It also enabled configuration of pallets.

## Development

Local builds and running a single node development chain is explained in the [development doc](../docs/development/development.md).

## Build container image

```sh
docker build -t tfchainnode:$(git describe --abbrev=0 --tags | sed 's/^v//') .
```

On an Apple Silicon chip, add `--platform linux/amd64`.

Add `--no-cache` if a newer rust toolchain is required.
