# Upgrading runtime

## Update runtime pallets

If you updated the pallets functionality and pushed it's default branch on github you can update the dependency as following:

```
cargo update -p pallet-tfgrid
cargo update -p pallet-smart-contract
```

Alternatively you can compile runtime modules in your runtime from a local path. You can change the git dependency to a local path dependency like:

```
[dependencies.pallet-smart-contract]
default-features = false
#path = "somepath/tfchain_pallets/pallet-smart-contract"
```

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

