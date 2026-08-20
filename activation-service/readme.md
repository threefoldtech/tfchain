# Substrate funding service

A TFChain Wallet account requires a minimum balance to exist and function. New TFChain users will not automatically have any tokens (also not on stellar).
 Therefore an activation service for new TFChain wallets is created. It activates new TFChain wallet addresses by depositing a minimal amount of TFT, set by `ACTIVATION_AMOUNT`.


## Installing and running

create `.env` file with following content:

```
URL=wss://substrate01.threefold.io
MNEMONIC=substrate ed25519 private words
ACTIVATION_AMOUNT=0.1
```

All three are required and the service refuses to start without them, so there is
no effective default — `lib/config.js` falls back to `0.1` only for callers that
bypass `bin/www`, such as the tests.

`ACTIVATION_AMOUNT` is in **whole TFT**, not base units; decimals are accepted down
to 1e-7 TFT. `0.1` is the recommended value: it is what the Helm chart defaults to
and what the service has actually funded since 2022, roughly 98 extrinsics' worth
of fees at current prices. An amount below the chain's existential deposit is
rejected at startup, since such a transfer cannot create an account.

The unit is the same however the service is deployed — it is read from the
environment, so Helm, `docker run -e`, and a local `.env` all take whole TFT.
Historically the variable was required but never read, and the amount was
hardcoded to 0.1 TFT while this file claimed 1 TFT.

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

Funds a Substrate account with `ACTIVATION_AMOUNT` if it is empty, or tops it back
up to the minimum extrinsic cost if its balance has fallen below that.

Returns 400 for an account id that is not a valid 32 byte public key, and 5xx if
the transfer fails on chain — including when the funding account is itself empty.

Example: Post to `localhost:3000/activation/activate`

```sh
curl --header "Content-Type: application/json" \
  --request POST \
  --data '{"substrateAccountID": "some_id"}' \
  http://localhost:3000/activation/activate
```

## KYC

The KYC signature is not validated. The verification code has been commented out
for years, and `POST /activation/create-entity` — its only caller — was removed in
#1103, having been an unauthenticated route that signed a fee-paying extrinsic from
the service wallet. `KYC_PUBLIC_KEY` is no longer read or required; re-add it if
the checks come back.

## Deployment

The image sets no environment variables of its own, so every deployment has to
supply all three. `ACTIVATION_AMOUNT` is in whole TFT here exactly as it is
everywhere else.

```sh
docker run -d -p 3000:3000 \
  -e URL=wss://tfchain.grid.tf/ws \
  -e MNEMONIC='...' \
  -e ACTIVATION_AMOUNT=0.1 \
  ghcr.io/threefoldtech/tfchain_activation_service:latest
```

For Kubernetes the same values come from the chart, where `activation_amount` maps
to this variable — see
[helm/tfchainactivationservice/values.yaml](helm/tfchainactivationservice/values.yaml).
