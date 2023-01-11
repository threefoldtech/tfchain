# 3. Third party service contract

Date: 2023-01-11

## Status

Accepted

## Context

See [here](https://github.com/threefoldtech/tfchain/issues/569) for more details.

## Decision

We decided to rework the twin Object, removing:

- Version
- IP

and Adding:

- Relay
- Pk (PublicKey)

We removed the version because we don't use that anymore. The IP becomes a relay address, this is some dns name where the twin can be contacted through. This will be set by RMB.
The Pk fields is a public key used for encrypting twin messages, with this public key anyone can verify the source.
