# Tfchain TFT bridge

Bridge TFT between a TFchain and the Stellar network.

There are 2 components that make up this bridge:

- A pallet that needs to be included in the [tfchain](https://github.com/threefoldtech/tfchain) runtime: [pallet-tft-bridge](../substrate-node/pallets/pallet-tft-bridge)
- Bridge daemons: [tfchain_bridge](./tfchain_bridge)

See more about [architecture](./docs/architecture.md)

## Running Bridge instances

### Mainnet

Bridge account on mainnet: `GBNOTAYUMXVO5QDYWYO2SOCOYIJ3XFIP65GKOQN7H65ZZSO6BK4SLWSC`

Can be interacted with on: https://dashboard.grid.tf

### Testnet

Bridge account on testnet `GA2CWNBUHX7NZ3B5GR4I23FMU7VY5RPA77IUJTIXTTTGKYSKDSV6LUA4`

Can be interacted with on: https://dashboard.test.grid.tf

### Devnet

Bridge account on devnet: `GDHJP6TF3UXYXTNEZ2P36J5FH7W4BJJQ4AYYAXC66I2Q2AH5B6O6BCFG`

Can be interacted with on: https://dashboard.dev.grid.tf

## Development

See [development](./docs/readme.md) to learn more.
