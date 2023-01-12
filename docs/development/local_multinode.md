# Local tfchain network

This document will explain how to run a local multi node tfchain network.

## Compile release build

```
cd substrate-node
cargo build --release
```

## Start the network

In a terminal window execute the following command:

```
./target/release/tfchain \
    --base-path /tmp/alice \
    --chain local \
    --alice \
    --port 30333 \
    --ws-port 9945 \
    --rpc-port 9933 \
    --node-key 0000000000000000000000000000000000000000000000000000000000000001 \
    --validator
```

In a second terminal window executing the following command:

```
./target/release/tfchain \
    --base-path /tmp/bob \
    --chain local \
    --bob \
    --port 30334 \
    --ws-port 9946 \
    --rpc-port 9934 \
    --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
    --validator \
    --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp
```

## Verify that they are creating blocks

After you start the second node, the nodes should connect to each other as peers and start producing blocks.

To verify blocks are being finalized:

Verify that you see lines similar to the following in the terminal where you started the first node:

```
2022-08-16 15:32:33 discovered: 12D3KooWBCbmQovz78Hq7MzPxdx9d1gZzXMsn6HtWj29bW51YUKB /ip4/127.0.0.1/tcp/30334
2022-08-16 15:32:33 discovered: 12D3KooWBCbmQovz78Hq7MzPxdx9d1gZzXMsn6HtWj29bW51YUKB /ip6/::1/tcp/30334
2022-08-16 15:32:36 🙌 Starting consensus session on top of parent 0x2cdce15d31548063e89e10bd201faa63c623023bbc320346b9580ed3c40fa07f
2022-08-16 15:32:36 🎁 Prepared block for proposing at 1 (5 ms) [hash: 0x9ab34110e4617454da33a3616efc394eb1ce95ee4bf0daab69aa4cb392d4104b; parent_hash: 0x2cdc…a07f; extrinsics (1): [0x4634…cebf]]
2022-08-16 15:32:36 🔖 Pre-sealed block for proposal at 1. Hash now 0xf0869a5cb8ebd0fcc5f2bc194ced84ca782d9749604e888c8b9b515517179847, previously 0x9ab34110e4617454da33a3616efc394eb1ce95ee4bf0daab69aa4cb392d4104b.
2022-08-16 15:32:36 ✨ Imported #1 (0xf086…9847)
2022-08-16 15:32:36 💤 Idle (1 peers), best: #1 (0xf086…9847), finalized #0 (0x2cdc…a07f), ⬇ 1.0kiB/s ⬆ 1.0kiB/s
2022-08-16 15:32:41 💤 Idle (1 peers), best: #1 (0xf086…9847), finalized #0 (0x2cdc…a07f), ⬇ 0.6kiB/s ⬆ 0.6kiB/s
2022-08-16 15:32:42 ✨ Imported #2 (0x0d5e…2a7f)
2022-08-16 15:32:46 💤 Idle (1 peers), best: #2 (0x0d5e…2a7f), finalized #0 (0x2cdc…a07f), ⬇ 0.6kiB/s ⬆ 0.6kiB/s
2022-08-16 15:32:48 🙌 Starting consensus session on top of parent 0x0d5ef31979c2aa17fb88497018206d3057151119337293fe85d9526ebd1e2a7f
2022-08-16 15:32:48 🎁 Prepared block for proposing at 3 (0 ms) [hash: 0xa307c0112bce39e0dc689132452154da2079a27375b44c4d94790b46a601346a; parent_hash: 0x0d5e…2a7f; extrinsics (1): [0x63cc…39a6]]
2022-08-16 15:32:48 🔖 Pre-sealed block for proposal at 3. Hash now 0x0c55670e745dd12892c9e7d5205085a78ccea98df393a822fa9b3865cfb3d51b, previously 0xa307c0112bce39e0dc689132452154da2079a27375b44c4d94790b46a601346a.
2022-08-16 15:32:48 ✨ Imported #3 (0x0c55…d51b)
2022-08-16 15:32:51 💤 Idle (1 peers), best: #3 (0x0c55…d51b), finalized #1 (0xf086…9847), ⬇ 0.7kiB/s ⬆ 0.9kiB/s
```

In these lines, you can see the following information about your blockchain:

- The second node identity was discovered on the network (12D3KooWBCbmQovz78Hq7MzPxdx9d1gZzXMsn6HtWj29bW51YUKB).
- The node has a one peer (1 peers).
- The nodes have produced some blocks (best: #3 (0x0c55…d51b)).
- The blocks are being finalized (finalized #1 (0xf086…9847)).

## Review the command-line options

Before moving on, have a look at how the following options are used to start the node.

- --base-path Specifies the directory for storing all of the data related to this chain.
- --chain local Specifies the chain specification to use. Valid predefined chain specifications include local, development, and staging.
- --alice Adds the predefined keys for the alice account to the node's keystore. With this setting, the alice account is used for block production and finalization.
- --port 30333 Specifies the port to listen on for peer-to-peer (p2p) traffic. Because this tutorial uses two nodes running on the same physical computer to simulate a network, you must explicitly specify a different port for at least one account.
- --ws-port 9945 Specifies the port to listen on for incoming WebSocket traffic. The default port is 9944. This tutorial uses a custom web socket port number (9945).
- --rpc-port 9933 Specifies the port to listen on for incoming RPC traffic. The default port is 9933.
- --node-key <key> Specifies the Ed25519 secret key to use for libp2p networking. You should only use this option for development and testing.
- --telemetry-url Specifies where to send telemetry data. For this tutorial, you can send telemetry data to a server hosted by Parity that is available for anyone to use.
- --validator Specifies that this node participates in block production and finalization for the network.

## More information

Can be found here: https://docs.substrate.io/tutorials/get-started/simulate-network/
