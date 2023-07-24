# DAO Module

Pallet for a DAO (decentralised autonomous organisation) proposals and voting. This pallet is tightly coupled with a configured [collective](https://github.com/paritytech/substrate/tree/master/frame/collective) and with [Tfgrid module](../pallet-tfgrid/readme.md).

## Overview

The DAO module provides functions for:

- Creating Proposals
- Voting on Proposals
- Closing Proposals
- Vetoing Proposals (X amount of veto votes will close and dissaprove a proposal)

## Terminology

- Council member: a member of the [collective](https://github.com/paritytech/substrate/tree/master/frame/collective)
- Proposal: A proposal is a request to execute an extrinsic on chain. A proposal can be created by a [council member](../../../docs/misc/minimal_DAO.md). This proposal has a threshold, a link to a forum post, a duration specified in amount of blocks, a description and an extrinsic to execute.
- Vote: farmer can vote for a proposal. A vote has a weight based on the farmers stake in the network. One vote by default is 1 weight. If the farmer has nodes, the weight of the vote is calulcated as following: Sum of all nodes of the farmer: (node CU * 2 + node SU)
- Veto: council members can veto a proposal, a proposal is vetod when majority of council members vote to veto a proposal. A veto closes and dissaproves a proposal.

## Implementations

This pallet implements a trait `ChangeNode`. Specifically it implements:

- `node_changed`: registers or updates a node's voting weight in storage
- `node_deleted`: deleted a node's voting weight

This trait's functions can be called by other modules to trigger changes in the voting weights of nodes.

### Addendum on Proposals

Proposals can be created by members of the council. A proposal can include the following properties:

- Extrinsic to execute (pricing change, connection price change, runtime upgrade, ..)
- Description of the proposals
- Link to a webpage / github markdown / wiki file, ...
- Threshold: amount of farmers that need to vote to have a valid proposal
- Duration: specified in amount of blocks

When a "generic" proposal needs to be executed, the extrinsic `system.setRemark` needs to be selected. The remark itself is up to the council member to choose.
This is because a proposal needs to have an extrinsic attached.
A proposal default duration is set by the config trait `MotionDuration` on this pallet, this value needs to be expressed in number of blocks.

A proposal can be closed either when threshold of votes is met or proposal duration ended. Only a council member can close a proposal.
Based on the voting result the proposal can be either approved(executed) or dissaproved.

## Interface

- `propose` - Create a proposal
- `vote` - Vote for a proposal
- `veto` - Veto a proposal
- `close` - Close a proposal

