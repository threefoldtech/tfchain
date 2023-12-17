# TFChain Bridge

TFChain bridge is a Chain-To-Chain bridge which mainly designed to support the movement of assets between two blockchains, TFChain and Stellar.
It is composed as a daemon that runs in the background scanning a Stellar vault address for deposits and TFChain Events for withdrawals and executes upon them. This allows cross-chain transfers between Stellar <> TFChain.

## what is a blockchain bridge?

Blockchain bridges connect two different blockchains, similar to how real-world bridges connect two different locations. Without a bridge, blockchains are siloed environments that cannot communicate with each other because each network has its own set of rules, governance mechanisms, native assets, and data that are incompatible with the other blockchains. However, with a bridge between two blockchains, it becomes possible to transfer crypto-assets and arbitrary data between them. Bridges are key for interoperability in the crypto ecosystem and are necessary to make different blockchain networks compatible with each other.

## Cross-Chain Mechanism

Bridges can be categorized by a number of characteristics. These include how they transfer information across chains which consider the most important factor.

Our bridge between Stellar TFT and TFChain TFT use a mechanism known as “locking” or “burning,” followed by either minting or withdrawing, respectively. let's describe how the mechanism works by using an example.
- the user begins by depositing the Stellar TFT version into a designated stellar address owned by the bridge and specifying the recipient on TFChain. This step is referred to as “locking.”.
- the bridge initiate a flow to “mints” or issues a version of the deposited asset on TFChain and credits it to the recipient account.
- When the user wants to move back to Stellar TFT, the TFChain token is simply “burned.” This allows the underlying asset on Stellar to be redeemed and sent to the specified recipient address.

## Development

In this document we will explain how the bridge works and how you can setup a local instance to develop against.
The local instance will consist of a connection between a tfchain that runs in development mode and Stellar Testnet.

See [architecture](./architecture.md) for more information on how the bridge works.

## Setup

### Development setup

Refer to [development](./development.md) for more information on how to setup a development instance.

### Production setup

Refer to [production](./production.md) for more information on how to setup a production instance.

### Bridging

When you have setup the bridge in either development or production mode you can start bridging.

See [bridging](./bridging.md) for more information on how to bridge.

## Log schema
Bridge validators use simple event log for the sake of improving observability and perform tracing on workflows and data.
you can find more about the log schema and how it can improve the observability of the system in [the bride observability document](./observability.md).
