# Requirements for devnet with NPoS enabled

We need atleast 5 Tfchain validator nodes on devnet for it to be safeguarded against possible downtimes on validators.

## Initial staked balance

Since the staking module documents that the ideal stake is 50% of all the circulating supply then the stake for each validator should be 400 million devnet TFT's (given that we run 5 validators).

So the Stash account of each validator should container 400 million TFT, the controller account should have 1 TFT (to manage the staking operations).

## Different locations

We should run these validators on 3 different locations in order to guarantee uptime.