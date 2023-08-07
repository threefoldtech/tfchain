## Production

This document will explain how you can run tfchain in production

## Compose a chain specification file

See this very thorough guide on substrate: https://docs.substrate.io/tutorials/get-started/add-trusted-nodes/#before-you-begin

## Releases

See [releases](./releases.md) for instructions on how to create / validate a release.

### Upgrading runtime

See [process](./upgrade_process.md)

### Inspecting a runtime

Install subwasm

```sh
cargo install --locked --git https://github.com/chevdor/subwasm --tag v0.19.1
```

Verify installation 

`subwasm -v`

Download a runtime from any of our live networks like:

```sh
subwasm get wss://tfchain.dev.grid.tf:443 -o runtime.wasm
```

Inspect:

```sh
subwasm info runtime.wasm
```