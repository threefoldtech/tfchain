# Using POS for tfchain

Questions to resolve:

- Can we transition from aura/grandpa to POS?
- Test what happens if 90 % of the validators are suddenly offline (if they are run on the grid for example and there is a problem).

## POS in substrate

There is a default pallet in substrate for staking: [pallet_staking](https://paritytech.github.io/substrate/master/pallet_staking/index.html).

This pallet also supports nominated POS (NPOS), it actually relies on it. This has the benefit of not having too many validators and that delegated staking is immediately solved.

## Staking rewards

The staking pallet uses a yearly inflation curve where rewards are newly minted tokens.
While other reward schemes can be implemented, let's take a yearly inflation of 5%.

In the contract values payouts, 30% is burned, let's increase this with 5% to compensate for the pos.

The [transaction fees are also burned](https://github.com/threefoldtech/tfchain/issues/72). This causes a little of deflation that compensates this a bit a well.

## Using vested tokens to participate in POS

Vested tokens are locked on the Stellar network.

There is a possibility to have the vested tokens to be used in the POS.

We could issue the vested tokens on tfchain on an account with the proper security measures and nominate a validator.
Slashing might be a problem but this can be turned off for the validator of the vested accounts if needed.

I'm not going more in to detail on this topic as the other ones are more important at the moment.

## Stimulating community participation and decentralization

If we run high uptime validators, why would someone else run a validator and why would other people nominate the other validators?

Comission is the percentage the validator gets for being a validator and is taken from the payout to the nominators.

A simple way to stimulate community paricipation and decentralization is by setting the comission of our validators to a high value. This way other validators can be preferred if they set their comission lower than ours.
