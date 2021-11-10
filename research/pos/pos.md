# Using POS for tfchain

Questions to resolve:

- Can we use POS?
- Is delegated staking possible?
- Can we transition from aura/grandpa to POS?
- How can the participants in the POS be rewarded?
- Is it possible to use the vested tokens on Stellar for POS?

## POS in substrate

There is a default pallet in substrate for staking: [pallet_staking](https://paritytech.github.io/substrate/master/pallet_staking/index.html).

This pallet also supports nominated POS (NPOS), it actually relies on it. This has the benefit of not having too many validators and that delegated staking is immediately solved.

## Using vested tokens to paricipate in POS

Vested tokens are locked on the Stellar network.

There is a possibility to have the vested tokens to be used in the POS.

We could issue the vested tokens on tfchain on an account with the proper security measures that and nominate a validator.
Slashing might be a problem but this can be turned off for the validator of the vested accounts if needed.

I'm not going more in to detail on this topic as the other ones are more important at the moment.
