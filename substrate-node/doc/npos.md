# NPOS for tfchain

Tfchain uses Nominated Proof Of Stake (NPOS) for the consensus. This allows everyone to participate in securing the chain.
The implementation is pallet_staking from [Threefoldtech/tfchain_pallets](https://github.com/threefoldtech/tfchain_pallets). This is a modification of the default substrate pallet_staking.

## Block creation and validators

Pallet_staking relies on Babe consensus for block creation and Grandpa for finalization. At the end of each era the new validators are selected and set to participate in the babe and grandpa consensus during the next era.

## Staking rewards

5% of the payed contract values go to a `npos_reward` account and at each payout time, 1% (minimizes variance) of the pos_reward account is distributed amongst the validators and nominators.

The npos_reward account account also gets the transaction fees spent in tfchain.

### Slashing

When slashing occurs( penalty for bad validators), the penalty goes to a foundation account.

## pos reward account

The pos reward account is `5CNposRewardAccount11111111111111111111111111FSU`, 5C being the network indicator  and the last 2 bytes are the checksum.

It is obviously an address created just to look nice, is human readable and has no known private key.

## Stimulating community paricipation and decentralization

If we run high uptime validators, to make sure the chain progresses, why would someone else run a validator and why would other people nominate the other validators?

Comission is the percentage the validator gets for being a validator and is taken from the payout to the nominators.

A simple way to stimulate community paricipation and decentralization is by setting the comission of our validators to a high value. This way other validators can be preferred if they set their comission lower than ours.
