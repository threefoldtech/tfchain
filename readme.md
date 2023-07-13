# TFchain &middot; ![Build & Tests](https://github.com/threefoldtech/tfchain/actions/workflows/010_build_and_test.yaml/badge.svg)

<p align="center">
  <img height="50%" width="50%" src="./substrate-node/.maintain/header.png">
</p>

Threefold blockchain serves as a registry for Nodes, Farms, Digital Twins and Deployment contracts.
It is the backbone of [ZOS](https://github.com/threefoldtech/zos) and other components.

## Docs

see [docs](./docs/readme.md) for more information on how to work with this component.

## Modules list

- [Tfgrid Module](./substrate-node/pallets/pallet-tfgrid/readme.md): registry for Nodes / Farms / Twins
- [Smart Contract Module](./substrate-node/pallets/pallet-smart-contract/readme.md): node and rent contracts
- [Dao Module](<(./substrate-node/pallets/pallet-dao/readme.md)>): voting on proposals that impact the system for farmers. See [dao](./docs/misc/minimal_DAO.md) for more info.
- [Kvstore Module](./substrate-node/pallets/pallet-kvstore/readme.md): key value store for deployment information
- [Validator Set Module](./substrate-node/pallets/substrate-validator-set/readme.md): module for adding / removing authorities
- [TFT Price Module](./substrate-node/pallets/pallet-tft-price/readme.md): TFT price oracle. See [price](./docs/misc/price.md) for more info.
- other less mentionable modules..

## Bridge

See [bridge](./bridge/README.md) for more information on the bridge between Tfchain TFT and Stellar TFT.

## Scripts

See [scripts](./scripts/README.md) for more information on how to use the scripts.
## Deployed instances

- Development network:

  - Polkadot UI: [https://polkadot.js.org/apps/?rpc=wss%3A%2F%2F/tfchain.dev.grid.tf#/explorer](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2F/tfchain.dev.grid.tf#/explorer)
  - Websocket url: wss://tfchain.dev.grid.tf

- Qa testing network:

  - Polkadot UI: [https://polkadot.js.org/apps/?rpc=wss%3A%2F%2F/tfchain.qa.grid.tf#/explorer](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2F/tfchain.qa.grid.tf#/explorer)
  - Websocket url: wss://tfchain.qa.grid.tf

- Test network

  - Polkadot UI: [https://polkadot.js.org/apps/?rpc=wss%3A%2F%2F/tfchain.test.grid.tf#/explorer](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2F/tfchain.test.grid.tf#/explorer)
  - Websocket url: wss://tfchain.test.grid.tf

- Production network

  - Polkadot UI: [https://polkadot.js.org/apps/?rpc=wss%3A%2F%2F/tfchain.grid.tf#/explorer](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2F/tfchain.grid.tf#/explorer)
  - Websocket url: wss://tfchain.grid.tf

## Commit messages

In this repository [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/) are used.

## Releases

Releases are automated by [this workflow](.github/workflows/030_create_release.yaml). When a release should be created following things need to be done:

- Increment spec version in the [runtime](./substrate-node/runtime/src/lib.rs) 
- Increment version in [Cargo.toml](./substrate-node/Cargo.toml)
- Increment version in [Chart.yaml](./substrate-node/charts/substrate-node/Chart.yaml)
- Increment version in [Chart.yaml](./bridge/tfchain_bridge/chart/tfchainbridge/Chart.yaml)
- Commit the changes
- Create a new tag with the version number prefixed with a `v` (e.g. `v1.0.0`, `v1.0.0-rc1` for release candidates) 
- Push the tag to the repository
- The workflow will create a release draft with the changelog and the binaries attached

A changelog will be generated based on the Pull requests merged, so having PRs with meaningful titles is important.

### Upgrade runtime

See [upgrade](./docs/production/upgrade_process.md) for more information on how to upgrade the runtime.