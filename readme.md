# Ledger Chain

A Substrate-based blockchain that handles node registration, smart contract management, billing, and consensus for decentralized infrastructure operations. It records the canonical state of nodes, deployments, and token transactions across the network.

## What this is

Ledger Chain is a purpose-built blockchain that serves as the coordination and settlement layer for a decentralized grid of compute, storage, and network resources. It maintains registries of nodes, farms, and twins; manages smart contracts between users and node operators; handles billing based on resource consumption; and provides a governance mechanism for network evolution. The chain also supports cross-chain transfers with Bitcoin, Binance Smart Chain, and Stellar.

## What this repository contains

- **Substrate node** — The core blockchain node implementation based on `substrate-node-template`
  - `node` — Application for participating in the blockchain network
  - `runtime` — Runtime crate specifying pallets and their configuration
  - `pallets` — Custom runtime modules (DAO, KV store, smart contract, grid registry, TFT bridge, TFT price oracle, validator management, burning)
  - `support` — Shared types and configuration used across the runtime, pallets, and node
  - `tests` — Integration tests
  - `chainspecs` — Genesis specifications for dev, QA, test, and main networks
  - `charts` — Kubernetes deployment charts
- **Bridge** — Background service for cross-chain transfers between Ledger Chain TFT and Stellar TFT
- **Clients** — Language-specific libraries for interacting with Ledger Chain
  - `tfchain-client-js` — JavaScript client
  - `tfchain-client-rs` — Rust client (in progress)
  - `tfchain-client-go` — Go client
- **Activation service** — Account activation service for new users
- **Tools** — Development and testing utilities, including a fork-off-substrate tool for runtime migration testing
- **Scripts** — JavaScript utilities for event subscription and chain interaction
- **Documentation** — Development, production, and architecture documentation

## Role in the stack

Ledger Chain is the ledger and coordination layer of the infrastructure stack. It is the source of truth for:

- **Node registry** — Which nodes exist, their capabilities, and their operational status
- **Farm registry** — Collections of nodes operated by independent providers
- **Smart contracts** — Agreements between users and nodes for resource allocation
- **Billing** — Periodic billing based on actual resource consumption
- **Governance** — On-chain proposals and voting for protocol changes

Nodes run ZOS / Zero-OS and report their state to the chain. Users interact with the chain to create contracts, and the chain ensures that node operators are compensated for the resources they provide.

## ZOS / Zero-OS

ZOS, also known as Zero-OS, is the operating system layer used to run and manage nodes. It provides the low-level runtime environment for workloads, networking, storage, and automation. Nodes running ZOS register themselves on Ledger Chain and receive deployment instructions through the chain.

## Relation to ThreeFold

This technology is used within the ThreeFold ecosystem and was first deployed on the ThreeFold Grid. The component itself is designed as reusable infrastructure technology and should be understood by its technical function first, independent of any specific deployment.

## Ownership

This repository is owned and maintained by TF-Tech NV, a Belgian company responsible for the development and maintenance of this technology.

## Runtime architecture

The Ledger Chain runtime is built using FRAME and consists of pallets from Substrate and custom pallets in `pallets/`.

### Substrate pallets

- **System** — Core functionality all other pallets depend on
- **Utility** — Derivative accounts and batched calls
- **Balances** — TFT token balance tracking
- **Timestamp** — On-chain notion of time
- **Transaction Payment** — Pre-dispatch transaction fee computation
- **Scheduler** — Dispatch scheduling at specified block numbers or periods
- **Session** — Session key management and rotation
- **Aura** — Aura consensus with offline reporting
- **Grandpa** — GRANDPA finality authority set management
- **Authorship** — Block author tracking
- **Collective** — Collective dispatch from member sets
- **Membership** — Membership control for account ID sets

### Custom pallets

- **pallet-dao** — On-chain governance by stakeholders
- **pallet-kvstore** — Key-value data storage for network participants
- **pallet-runtime-upgrade** — Configurable-origin wrapper for `set_code` extrinsics
- **pallet-smart-contract** — Creation, billing, and deletion of smart contracts
- **pallet-tfgrid** — Registry for nodes, farms, and twins
- **pallet-tft-bridge** — Bridging between Ledger Chain TFT and Stellar TFT
- **pallet-tft-price** — TFT price oracle
- **substrate-validator-set** — Authority addition and removal
- **pallet-validator** — Validator management (deprecated)
- **pallet-burning** — TFT burning (deprecated)

## Docs

See [docs](./docs/readme.md) for more information on how to work with Ledger Chain.

## Bridge

See [bridge](./bridge/README.md) for more information on the bridge between Ledger Chain TFT and Stellar TFT.

## Scripts

See [scripts](./scripts/README.md) for more information on how to use the scripts.

## Deployed instances

- **Development network**
  - Polkadot UI: [https://polkadot.js.org/apps/?rpc=wss%3A%2F%2F/tfchain.dev.grid.tf#/explorer](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2F/tfchain.dev.grid.tf#/explorer)
  - WebSocket: `wss://tfchain.dev.grid.tf`

- **QA testing network**
  - Polkadot UI: [https://polkadot.js.org/apps/?rpc=wss%3A%2F%2F/tfchain.qa.grid.tf#/explorer](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2F/tfchain.qa.grid.tf#/explorer)
  - WebSocket: `wss://tfchain.qa.grid.tf`

- **Test network**
  - Polkadot UI: [https://polkadot.js.org/apps/?rpc=wss%3A%2F%2F/tfchain.test.grid.tf#/explorer](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2F/tfchain.test.grid.tf#/explorer)
  - WebSocket: `wss://tfchain.test.grid.tf`

- **Production network**
  - Polkadot UI: [https://polkadot.js.org/apps/?rpc=wss%3A%2F%2F/tfchain.grid.tf#/explorer](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2F/tfchain.grid.tf#/explorer)
  - WebSocket: `wss://tfchain.grid.tf`

## Commit messages

This repository uses [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/).

## Releases

See [releases](./docs/production/releases.md) for more information on how to create or validate a release.

## License

This project is licensed under the Apache License 2.0 — see the [LICENSE](LICENSE) file for details.
Copyright (c) TFTech NV.
