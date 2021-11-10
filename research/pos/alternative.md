# Staking alternative

Staking in an npos network relies heavily on inflation of tokens. Meaning the stakers will get rewards based on the amount of tokens created every block. For the Threefold network this not the case, we only create (mint) tokens when capacity is added on the network. Meaning we cannot distribute rewards to validators/nominators in an npos network based on inflation. Reverse engineering this concept can introduce flaws in the staking concept. As the rewards for regular staking always outweigh the cost of hardware, since running a validator requires 100% uptime and top notch hardware in order to not get "slashed".

## Proposal

We keep the current permissioned network and don't allow outsiders to run validators since the Threefold tokenomics don't account for having staking rewards. Rather it rewards farmers for adding capacity.

A staking module can be implemented as following in order to reward a user for "locking" up tokens in a staking wallet whilst still getting a decent amount of rewards for it.

Staking rewards could be generated from the usage of the grid:

- % of the farming rewards
- % of the cultivation rewards
- % of the transaction fees generated

These rewards could be sent to a defined staking pool.

All these rewards combined could be distributed to the stakers in the staking pool based on their amount of "stake". The amount of tokens in the staking pool would go up drastically if the usage of the grid becomes larger. More rewards would be funneled to the staking pool making staking a very nice incentive for users.

Given this concept a user would not need to run a validator on expensive hardware whilst still benefiting on the amount of tokens he is willing to stake.
