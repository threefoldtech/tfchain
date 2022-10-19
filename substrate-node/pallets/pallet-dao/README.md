# TFChain Pallet DAO

Pallet for DAO proposals and voting.

## Proposals

Proposals can be created by members of the council. A proposal can include the following properties:

- Extrinsic to execute (pricing change, connection price change, runtime upgrade, ..)
- Description of the proposals
- Link to a webpage / github markdown / wiki file, ...
- Threshold: amount of farmers that need to vote to have a valid proposal

When a "generic" proposal needs to be executed, the extrinsic `system.setRemark` needs to be selected. The remark itself is up to the council member to choose.
This is because a proposal needs to have an extrinsic attached.
A proposal duration is set by the config trait `MotionDuration` on this pallet, this value needs to be expressed in number of blocks.

A proposal can be closed either when threshold of votes is met or proposal duration ended.
Only a council member can close a proposal.
Based on the voting result the proposal can be either approved(executed) or dissaproved.

## Voting

Any farmer on chain can vote for proposals. Votes are weighted based on the farmers stake in the network. One vote by default is 1 weight.

If the farmers has nodes, the weight of the vote is calulcated as following:

- Sum of all nodes of the farmer: (node CU * 2 + node SU)

## Building

`cargo build`

## Testing

`cargo test`
