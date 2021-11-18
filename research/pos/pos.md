# Using POS for tfchain

Questions to resolve:

- Can we transition from aura/grandpa to POS?
- Test what happens if 90 % of the validators are suddenly offline (if they are run on the grid for example and there is a problem).

## POS in substrate

There is a default pallet in substrate for staking: [pallet_staking](https://paritytech.github.io/substrate/master/pallet_staking/index.html).

This pallet also supports nominated POS (NPOS), it actually relies on it. This has the benefit of not having too many validators and that delegated staking is immediately solved.

## Staking rewards

The staking pallet uses a yearly inflation curve where rewards are newly minted tokens. For tfchain this is not an option.

Instead 5% of the payed contract vales should go to a `npos_reward` account and at each payout time, 1% of the pos_reward account is distributed (minimizes variance).

**This is no default pallet_staking functionality.**
But it should be possible by overriding the [make_payout function](https://github.com/paritytech/substrate/blob/755569d202b4007179cc250279bad55df45b5f7d/frame/staking/src/pallet/impls.rs#L223).

Currently [transaction fees are also burned](https://github.com/threefoldtech/tfchain/issues/72). This causes a little of deflation. The transaction fees should also go to the npos_reward pool.

## Using vested tokens to paricipate in POS

Vested tokens are locked on the Stellar network.

There is a possibility to have the vested tokens to be used in the POS.

We could issue the vested tokens on tfchain on an account with the proper security measures and nominate a validator.
Slashing might be a problem but this can be turned off for the validator of the vested accounts if needed.

I'm not going more in to detail on this topic as the other ones are more important at the moment.

## Stimulating community paricipation and decentralization

If we run high uptime validators, why would someone else run a validator and why would other people nominate the other validators?

Comission is the percentage the validator gets for being a validator and is taken from the payout to the nominators.

A simple way to stimulate community paricipation and decentralization is by setting the comission of our validators to a high but acceptable value. This way other validators can be preferred if they set their comission lower than ours.

## npos reward pool adress

It would be nice if the npos reward pool account address would be a human readable one that obviously has no possible private key associated with it.

Besides the fact that this is very clear, no private key that can leak or kept secret is present.

The Base58 alphabet consists of the following characters:
123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz

`NposRewardPoolAccount11....` should as such be possible, prefixed with the network type and the checksum at the end that should match according to the [SS58 format](https://docs.substrate.io/v3/advanced/ss58/).
