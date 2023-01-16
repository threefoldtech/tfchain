### Key generation

Keys are generated using [subkey](https://substrate.dev/docs/en/knowledgebase/integrate/subkey) or use `tfchain key`.

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
