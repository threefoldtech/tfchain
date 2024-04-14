# Multinode setup

In this document we will explain how you can setup a local multinode instance of the bridge and develop against it.

## Step 1: Run tfchain

See [tfchain](https://github.com/threefoldtech/tfchain/blob/development/docs/development)

Create a twin on your local chain:

- Open https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/extrinsics
- Select `Alice` as source account
- Select `TfgridModule` -> `UserAcceptTc` and input ("somelink", "somehash")
- Select `TfgridModule` -> `CreateTwin` and submit the default values

## Step 2: Run bridge daemons

In this example we generated 3 keypairs using this [tool](https://github.com/threefoldfoundation/tft/tree/main/bsc/bridges/stellar/utils)

```sh
./stellar-utils generate bridge 3
generating 3 accounts and setting account options ...

Bridge master address: GAYJSBPBQ3J32CZZ72OM3GZP646KSVD3V5QB3WBJSSGPYHYS5MZSS4Z6
Bridge master secret: SDUHCIXQGCPQ2535FM2QK6KX43XH7RLCJ33RGYOKUYLA7E5W7LPBNEV7

Signer 1 address: GBFJM4HM4O7LTR2XLIIJMFPETKRQSRV6BF65DAUNR43WFE325FTLCURQ
Signer 1 secret: SCMEF63H2BA7WZDDBNGUP6YSGVW3O356CFOE4SHEGLQX3TXULEGEIXSG

Signer 2 address: GDCD6UY43MSGHMMYMXN4BGCJT75XMKZDL5INMTMS3YAKZ435WK3NZ5YH
Signer 2 secret: SBFMRNGJQ5NMVXJKDMSBHDDCHVXOJE4E7A62A4MHAD4A5DH5RU5ONWVK
```

The bridge master address is the address that will essentially vault the TFT. The other signer addresses / secrets are used to perform multisig on the master bridge wallet address. Running the above tool will set all the generated signer addresses as a signer on the bridge master address.

Following predefined tfchain keys can be used to start a bridge daemon:

- `quarter between satisfy three sphere six soda boss cute decade old trend` (bridge validator 1)
- `employ split promote annual couple elder remain cricket company fitness senior fiscal` (bridge validator 2)
- `remind bird banner word spread volume card keep want faith insect mind` (bridge validator 3)

By default, only 1 bridge validator is inserted in the tfchain runtime. In this example we will run a 3 node bridge setup, so we need to add 2 keys to the bridge validators on tfchain.

- Open: https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/extrinsics
- Select "Alice" account
- Select `Sudo` -> `Call` -> `tftBridgeModule` -> `addBridgeValidator` and input: `5CGQ6zra7qXw4RNVwUYM4bxHjxdj9VZH7DKsDwhfBCgVPmUZ`
- Do the same for: `5CVLaAHnvdX1CBsaKFKgfyfb91y5ApNP7FLSaczKginip1ho`

We inserted 2 addresses as authorized bridge validators, these addresses map to the last 2 of the predefined keys listed above

See:

```sh
subkey inspect "employ split promote annual couple elder remain cricket company fitness senior fiscal"
Secret phrase `employ split promote annual couple elder remain cricket company fitness senior fiscal` is account:
  Secret seed:      0xda84de3116bf3d9036c0a376e90c1b8a5a84ef52ce85a732fa0ffd0ab2699be6
  Public key (hex): 0x08eb2ee1d96b113f489dcda348b296fe74e4c54706287ff874ac587da7ca0c1f
  Account ID:       0x08eb2ee1d96b113f489dcda348b296fe74e4c54706287ff874ac587da7ca0c1f
  SS58 Address:     5CGQ6zra7qXw4RNVwUYM4bxHjxdj9VZH7DKsDwhfBCgVPmUZ
```

And

```sh
subkey inspect "remind bird banner word spread volume card keep want faith insect mind"
Secret phrase `remind bird banner word spread volume card keep want faith insect mind` is account:
  Secret seed:      0x354c820c3a4b8b4e6b4df6e9f6e0223f88ad15b3275b26c8837cc43cb6c42039
  Public key (hex): 0x12c97c156fea423e3b0a35361a7c8c9925dafdd20c5451507c4725c546224722
  Account ID:       0x12c97c156fea423e3b0a35361a7c8c9925dafdd20c5451507c4725c546224722
  SS58 Address:     5CVLaAHnvdX1CBsaKFKgfyfb91y5ApNP7FLSaczKginip1ho
```

> For every stellar keypair, we should have a tfchain keypair. As you can see above, we generated 3 bridge wallets and we have 3 tfchain keys.

Now lets run the bridge: change directory to the tfchain_bridge daemon (Make sure tfchain is running on your host)

```
cd tfchain_bridge
```

Now open a first terminal pane and execute:

```sh
./tfchain_bridge --secret SDUHCIXQGCPQ2535FM2QK6KX43XH7RLCJ33RGYOKUYLA7E5W7LPBNEV7 --tfchainurl ws://localhost:9944 --tfchainseed "quarter between satisfy three sphere six soda boss cute decade old trend" --bridgewallet GAYJSBPBQ3J32CZZ72OM3GZP646KSVD3V5QB3WBJSSGPYHYS5MZSS4Z6 --persistency ./signer1.json --network testnet
```

See we use the bridge master secret and wallet generated above in the run command.

Now Open a second terminal pane and execute:

```sh
./tfchain_bridge --secret SCMEF63H2BA7WZDDBNGUP6YSGVW3O356CFOE4SHEGLQX3TXULEGEIXSG --tfchainurl ws://localhost:9944 --tfchainseed "employ split promote annual couple elder remain cricket company fitness senior fiscal" --bridgewallet GAYJSBPBQ3J32CZZ72OM3GZP646KSVD3V5QB3WBJSSGPYHYS5MZSS4Z6 --persistency ./signer2.json --network testnet
```

> See we used the second generated steller secret but we still use the bridge master address as bridge wallet address.

Now open a third teminal pane and execute:

```sh
./tfchain_bridge --secret SBFMRNGJQ5NMVXJKDMSBHDDCHVXOJE4E7A62A4MHAD4A5DH5RU5ONWVK --tfchainurl ws://localhost:9944 --tfchainseed "employ split promote annual couple elder remain cricket company fitness senior fiscal" --bridgewallet GAYJSBPBQ3J32CZZ72OM3GZP646KSVD3V5QB3WBJSSGPYHYS5MZSS4Z6 --persistency ./signer3.json --network testnet
```

If all goes well, you should see something similar to following output:

```sh
5:25PM INF required signature count 3
5:25PM INF account GAYJSBPBQ3J32CZZ72OM3GZP646KSVD3V5QB3WBJSSGPYHYS5MZSS4Z6 loaded with sequence number 4328279062347778
5:25PM INF starting stellar subscription...
5:25PM INF starting tfchain subscription...
5:25PM INF awaiting signal
5:25PM INF fetching stellar transactions account=GAYJSBPBQ3J32CZZ72OM3GZP646KSVD3V5QB3WBJSSGPYHYS5MZSS4Z6 cursor=0 horizon=https://horizon-testnet.stellar.org/
5:25PM INF fetching events for blockheight ID=6
5:25PM INF received transaction on bridge stellar account hash=7a8c181f5e738ffeb68dda6518adf3ce4cf99777a4bd98a43dfed38ca0f99912
5:25PM INF received transaction on bridge stellar account hash=a8609616ac84f57eeadae8e6cde88025d0ab2ecbe8f1c70c7162b7548f20ae9a
5:25PM INF received transaction on bridge stellar account hash=a2eef986828038570a56314801626ee53e141408f0fc3c3eb96cca62fae81436
5:25PM INF fetching stellar transactions account=GAYJSBPBQ3J32CZZ72OM3GZP646KSVD3V5QB3WBJSSGPYHYS5MZSS4Z6 cursor=4328296242225152 horizon=https://horizon-testnet.stellar.org/
5:25PM INF fetching events for blockheight ID=7
5:25PM INF fetching events for blockheight ID=8
5:25PM INF fetching stellar transactions account=GAYJSBPBQ3J32CZZ72OM3GZP646KSVD3V5QB3WBJSSGPYHYS5MZSS4Z6 cursor=4328296242225152 horizon=https://horizon-testnet.stellar.org/
5:25PM INF fetching events for blockheight ID=9
5:25PM INF fetching stellar transactions account=GAYJSBPBQ3J32CZZ72OM3GZP646KSVD3V5QB3WBJSSGPYHYS5MZSS4Z6 cursor=4328296242225152 horizon=https://horizon-testnet.stellar.org/
5:25PM INF fetching events for blockheight ID=10
5:25PM INF fetching events for blockheight ID=11
```

## Step 3: Setup a personal wallet

First generate another account which you can use to interact with the bridge:

Use the same tool we used above to generate the keypairs: [tool](https://github.com/threefoldfoundation/tft/tree/main/bsc/bridges/stellar/utils)

```sh
./stellar-utils generate plain

New Account address: GCNFIHEN7LQZ4BVA4ADXIXEHPUXSMM6WHNKRR6MD3BYOBJZ3ADUW44TK
New Account secret: SDGRCA63GSP4MSASFAWX5FORTS6ATQMK63YL6ZMF7YIFEJVBTLJDJA3M
```

Now, request some Testnet TFT by doing a swap on the stellar dex using the same tool:

```sh
./stellar-utils faucet --secret SDGRCA63GSP4MSASFAWX5FORTS6ATQMK63YL6ZMF7YIFEJVBTLJDJA3M
```

Given this command did not give an error, your account you just generated now has 100 TFT.

## Step 4: Deposit TFT to the bridge

Make sure that

- Tfchain is running
- The bridge daemons are running
- You generated a personal wallet and have TFT on it

You can start doing a deposit on the bridge using the same tool again

First identify the bridge master address you generated above, in this example the address is: `GAYJSBPBQ3J32CZZ72OM3GZP646KSVD3V5QB3WBJSSGPYHYS5MZSS4Z6`.

Now construct a memo message indicating which twin you will deposit to: "twin_TWINID" (you should have created a twin in the above steps).

```sh
./stellar-utils transfer 50 "twin_1" GAYJSBPBQ3J32CZZ72OM3GZP646KSVD3V5QB3WBJSSGPYHYS5MZSS4Z6 --secret SDGRCA63GSP4MSASFAWX5FORTS6ATQMK63YL6ZMF7YIFEJVBTLJDJA3M
```

Now you should have received the tokens minus the depositfee on your account on tfchain (the default depositfee is 10 TFT).
