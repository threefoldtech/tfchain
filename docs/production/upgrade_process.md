# Upgrading runtime

## Check first

- Make sure CI passes and it compiles locally
- Test migrations (if any) with the tools described in [try-runtime](../misc/try_runtime.md) and [tools](../../tools/fork-off-substrate/README.md)

## Increment spec version

Open `substrate-node/runtime/src/lib.rs` and increment the spec version with the current spec version. You can see what spec version a node is running on the polkadot ui.
In the top left you should see something like: `substrate-threefold/1` which means it's running the runtime `substrate-threefold` with spec version `1`.

## Compile a runtime wasm build

You can pass an optional release flag.

```
cargo build [--release] -p tfchain-runtime
```

## Upload the wasm build to the Node using Polkadot UI

Browse to the Polkadot UI and connect to the Node, select `Developer` -> `Extrinsics`.

Now select a `Root` account to upgrade the runtime with, if the node is running in `Dev`, the default `Root` account is Alice.

[example](./doc/upgrade_runtime.png)

Select `Sudo` -> `sudoUncheckedWeight` -> `System` -> `setCode` -> Toggle `file upload` and click the box.

Navigate to `tfchain/substrate-node/target/release/wbuild/tfchain-runtime/tfchain_runtime.compact.wasm` and upload that file.

Submit the transaction, wait a couple of seconds and the runtime version should be incremented and the code will be live.

## IMPORTANT

If a type change occurred in the modified runtime pallets, all objects stored under that type will be invalidated unless a runtime storage migration is executed.

There are several ways to work around that but none are advised. More information on runtime upgrades can be found [here](https://substrate.dev/docs/en/knowledgebase/runtime/upgrades).

### Upgrade grapqhl

[graphql upgrade documentation](../graphql/docs/readme.md)
