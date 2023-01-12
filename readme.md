# TFchain &middot; ![Build & Tests](https://github.com/threefoldtech/tfchain/actions/workflows/build_test.yaml/badge.svg)

<p align="center">
  <img height="50%" width="50%" src="./substrate-node/.maintain/header.png">
</p>

Threefold blockchain serves as a registry for Nodes, Farm, Digital Twins and Deployment contracts.
It is the backbone of [ZOS](https://github.com/threefoldtech/zos) and other components.

## Structure

- Node: defines the way the node communicates with other nodes.
- Pallet: defines runtime behaviour, these are modules that can work together or independantly.
- Runtime: The runtime determines whether transactions are valid or invalid and is responsible for handling changes to the blockchain's state transition function. It also enabled configuration of pallets.

## Docs

see [docs](./docs/readme.md) for more information on how to work with this component.

## Feature list

- [Tfgrid Module](./substrate-node/pallets/pallet-tfgrid/readme.md): registry for Nodes / Farms / Twins
- [Smart Contract Module](./substrate-node/pallets/pallet-smart-contract/readme.md): node and rent contracts
- [Dao Module](<(./substrate-node/pallets/pallet-dao/readme.md)>): voting on proposals that impact the system for farmers. See [dao](./docs/misc/minimal_DAO.md) for more info.
- [Kvstore Module](./substrate-node/pallets/pallet-kvstore/readme.md): key value store for deployment information
- [Validator Set Module](./substrate-node/pallets/substrate-validator-set/readme.md): module for adding / removing authorities
- [TFT Price Module](./substrate-node/pallets/pallet-tft-price/readme.md): TFT price oracle. See [price](./docs/misc/price.md) for more info.
- other less mentionable modules..

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
