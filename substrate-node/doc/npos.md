# NPOS for tfchain

Tfchain uses Nominated Proof Of Stake (NPOS) for the consensus. This allows everyone to participate in securing the chain.
The implementation is pallet_staking from [Threefoldtech/tfchain_pallets](https://github.com/threefoldtech/tfchain_pallets). This is a modification of the [default substrate pallet_staking](https://github.com/paritytech/substrate/tree/v3.0.0/frame/staking).

## Overview

The Staking module is the means by which a set of network maintainers (known as validators) are chosen based upon those who voluntarily place funds under deposit. Under deposit, those funds are rewarded under normal operation but are held at pain of slash (expropriation) should the staked maintainer be found not to be discharging its duties properly.

## Block creation and finalization

Pallet_staking relies on Babe consensus for block creation and Grandpa for finalization. At the end of each era the new validators are selected and set to participate in the babe and grandpa consensus during the next era.

## Staking rewards

5% of the payed contract values go to a `npos_reward` account and at each payout time, 1% (minimizes variance) of the pos_reward account is distributed amongst the validators and nominators.

The npos_reward account account also gets the transaction fees spent in tfchain.

## pos reward account

The pos reward account is `5CNposRewardAccount11111111111111111111111111FSU`, 5C being the network indicator  and the last 2 bytes are the checksum.

It is obviously an address created just to look nice, is human readable and has no known private key.

## Stimulating community paricipation and decentralization

If we run high uptime validators, to make sure the chain progresses, why would someone else run a validator and why would other people nominate the other validators?

Comission is the percentage the validator gets for being a validator and is taken from the payout to the nominators.

A simple way to stimulate community paricipation and decentralization is by setting the comission of our validators to a high value. This way other validators can be preferred if they set their comission lower than ours.

## Terminology

- Staking: The process of locking up funds for some time, placing them at risk of slashing (loss) in order to become a rewarded maintainer of the network.
- Validating: The process of running a node to actively maintain the network, either by producing blocks or guaranteeing finality of the chain.
- Nominating: The process of placing staked funds behind one or more validators in order to share in any reward, and punishment, they take.
- Stash account: The account holding an owner's funds used for staking.
- Controller account: The account that controls an owner's funds for staking.
- Era: A (whole) number of sessions, which is the period that the validator set (and each validator's active nominator set) is recalculated and where rewards are paid out.
- Slash: The punishment of a staker by reducing its funds.

## Slashing

More information on slashing (penalty for bad validators), unresponsiveness and slashing calculations can be found on the [polkadot wiki](https://wiki.polkadot.network/docs/learn-staking#slashing).

Tfchain did not apply any modifications to the slashing logic besides the fact that slashed amounts go to a foundation account, this reduces the incentive for validators to attack each other.
