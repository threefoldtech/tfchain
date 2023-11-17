# Council

The Council is a group of people that have collective interests in the Threefold Chain. The council is responsible for the following:

- Proposing & Approving runtime upgrades
- Approving chain configuration changes
- Approving new council members
- Approving new validators
- ..

## Council members

Council members are configurable at runtime, members can be added and removed by the council itself.

### Adding / Removing members

Browse to the Polkadot UI and connect to the Node, select the `Governance` -> `Council` page

Select `Motion` on top and click `Propose Motion` on the right.

Now select `councilMembership` as proposal and select `addMember` or `removeMember` as extrinsic.

Use the `id` of the account address (SS58) you want to add or remove.

## Proposals

Proposals are created by council members and can be voted on by other council members. Proposals can be created as following:

Browse to the Polkadot UI and connect to the Node, select the `Governance` -> `Council` page 

Select `Motions` on top and click `Propose Motion` on the right.

Now select any motion (any extrinsic from any pallet can be proposed) and click `Propose`

## Voting

Voting can be done by council members, the voting process is as following:

Browse to the Polkadot UI and connect to the Node, select the `Governance` -> `Council` page

View any open motion and click `Vote` (Select Aye or Nay)

## Closing motions

When a motion is closed, the motion is executed on the chain. This means that the extrinsic is executed and the state of the chain is changed.

To close a motion, browse to the Polkadot UI and connect to the Node, select the `Governance` -> `Council` page

View any open motion and click `Close`