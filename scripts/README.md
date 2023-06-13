# Tfchain scripts

Some scripts to read/write data to tfchain.

## Installation

Requires node 16+ and yarn.

```bash
yarn
```

## Get events

- Network: dev / test / main

```bash
node events.js <network>
```

## Attach solution provider ID to contracts

- Network: dev / test / main
- ProviderID: ID of the solution provider
- Mnemonics: Mnemonics of the solution provider account which created the contracts

```bash
node attachContractToProvider.js <network> <providerID> <mnemonics>
```