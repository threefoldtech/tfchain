# Tfchain substrate Node

## Development

Local builds and running a single node development chain is explained in the [development doc](./development.m).

## Multi-Node local chain

Everything needed in order to create multi-node network chain is explained in the official documentation of Substrate.

[start a private network here](https://substrate.dev/docs/en/tutorials/start-a-private-network/)

### Key generation

Keys are generated using [subkey](https://substrate.dev/docs/en/knowledgebase/integrate/subkey).

For eack Aurablockproducer generate a key:

```sh
subkey generate --scheme sr2551
```

If you want it to work as a GRANDPA validator, create the ed25519 public key and address:

```sh
subkey inspect --scheme ed25519 "<seed from the previous command>"
```
