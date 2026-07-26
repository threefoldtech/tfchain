# Substrate funding service

A TFChain Wallet account requires a minimum balance to exist and function. New TFChain users will not automatically have any tokens (also not on stellar).
 Therefore an activation service for new TFChain wallets is created. It activates new TFChain wallet addresses by depositing a minimal amount of TFT, set by `ACTIVATION_AMOUNT`.


## Installing and running

create `.env` file with following content:

```
URL=wss://substrate01.threefold.io
MNEMONIC=substrate ed25519 private words
KYC_PUBLIC_KEY=kyc service 25119 public key
ACTIVATION_AMOUNT=1
```

`ACTIVATION_AMOUNT` is in whole TFT; decimals are accepted down to 1e-7 TFT. It
defaults to `0.1` when unset, which is the amount the service funded before the
variable was read at all — until then it was required but ignored, and the amount
was hardcoded to 0.1 TFT despite this file claiming 1 TFT. An amount below the
chain's existential deposit is rejected at startup, since such a transfer cannot
create an account.

An account whose balance has fallen below 0.0015 TFT is topped back up by that
amount instead of being funded in full.

Run backend

```
yarn
yarn start
```

## Endpoints

### Activate

`/activation/activate`

Activates a Substrate account and puts 500 tokens on it.

Example: Post to `localhost:3000/activation/activate`

```sh
curl --header "Content-Type: application/json" \
  --request POST \
  --data '{"substrateAccountID": "some_id"}' \
  http://localhost:3000/activation/activate
```

### Create Entity

`/activation/create-entity`

Creates an entity object in the griddb.

## KYC

The KYC signature is currently not validated 


## Deployment

Build the docker image and configure following environment variables:

```
MNEMONIC=mnemonic words for account that activates
URL=substrate websocket url
```
