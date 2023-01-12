## Production

This document will explain how you can run tfchain in production

### Upgrading runtime

See [process](./substrate-node/upgrade_process.md)

### Client

You can use the client to interact with the chain, [read more](./cli-tool/readme.md)

### Data Cleanup:

To wipe data run:

```
./target/release/tfchain purge-chain --dev
```
