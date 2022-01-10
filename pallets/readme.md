# TFChain Pallets

This repository contains a set of Runtime modules for the Threefold Chain (TFChain).

## Importing a Runtime module in a Runtime

### Pallet TFGrid

[readme](./pallet-tfgrid/readme.md)

Modify `runtime/Cargo.toml` with:

```toml
[dependencies.pallet-tfgrid]
default-features = false
git = "https://github.com/threefoldtech/tfchain_pallets"
package = "pallet-tfgrid"
branch = "development"
```

### Pallet Smart Contract

[readme](./pallet-smart-contract/readme.md)

Modify `runtime/Cargo.toml` with:

```toml
[dependencies.pallet-smart-contract]
default-features = false
git = "https://github.com/threefoldtech/tfchain_pallets"
package = "pallet-smart-contract"
branch = "development"
```
