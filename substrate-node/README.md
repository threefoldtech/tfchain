# Tfchain substrate Node

## Development

Local builds and running a single node development chain is explained in the [development doc](./development.m).

## Build container image

```sh
docker build -t tfchainnode:$(git describe --abbrev=0 --tags | sed 's/^v//') .
```

Add `--no-cache` if a newer rust toolchain is required.

## Multi-Node chain

Everything needed in order to create multi-node network chain is explained in the official documentation of Substrate.

[start a private network here](https://substrate.dev/docs/en/tutorials/start-a-private-network/)

### Key generation

Keys are generated using [subkey](https://substrate.dev/docs/en/knowledgebase/integrate/subkey) or use  `tfchain key`.

For each Aura blockproducer generate a key:

```sh
subkey generate --scheme sr25519
```

If you want it to work as a GRANDPA validator, create the ed25519 public key and address:

```sh
subkey inspect --scheme ed25519 "<seed from the previous command>"
```

For bootnodes, it is is best to generate a nodekey so the bootnode address is predictive:

```sh
subkey generate-node-key
```

This prints the p2p nodeId to stderr and the nodekey to stdout.

### Creating a custom chain spec

First build the tfchain binary.

Export the local chain spec to json:

```sh
./target/release/tfchain build-spec --disable-default-bootnode --chain local > chainspecs/<name>/chainSpec.json
```

Change the `genesis/runtime/palletAura/authorities` and  `genesis/runtime/palletGrandpa/authorities` with SS58 public keys generated in the above explained key generation section.

Distributing a raw spec ensures that each node will store the data at the proper storage keys.so convert it to raw chain spec:

```sh
./target/release/tfchain build-spec --chain=chainspecs/<name>/chainSpec.json --raw --disable-default-bootnode > chainspecs/<name>/chainSpecRaw.json
```
