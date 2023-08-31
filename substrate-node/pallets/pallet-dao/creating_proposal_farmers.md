# Creating a Proposal for farmers to vote on

Only a council member can create a proposal for farmers to vote on.

## Step 1: go to Polkadot UI

Open the Polkadot JS UI in your browser:

- devnet:  https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Ftfchain.dev.grid.tf%2Fws#/extrinsics
- qanet:   https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Ftfchain.qa.grid.tf%2Fws#/extrinsics
- testnet: https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Ftfchain.test.grid.tf%2Fws#/extrinsics
- mainnet: https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Ftfchain.grid.tf%2Fws#/extrinsics

## Step 2: open proposal

- Go to `Developer` -> `Extrinsics`
- Make sure the council member account is selected
- Select `dao` -> `propose()` extrinsic

## Step 3: fill proposal

The proposal must include the following arguments:

- threshold: minimal number of farmer votes required to be able to close proposal before its end.
- action: call/extrinsic to execute on chain. If there is no call to be executed (which is usually the case) then `system` -> `remark()` should be set.
- description: a small description of what the proposal is about.
- link: a link to a more elaborate explanation of the proposal.
- duration: optional duration of the proposal after beeing created (default is 7 days, max value is 30 days), expressed in number of blocks (1 block = 6 sec).

![create](./img/create.png)

## Step 4: submit proposal

Make sure you have enought funds for transaction fee and submit the proposal.

! Remark: Once a proposal is created it cannot be altered or removed !

![submit](./img/submit.png)

## Step 5: check proposal

You can check if proposal was created.

- Go to `Developer` -> `Chain state` -> `dao` -> `proposalList()` to get the hash list of active proposals
- With the given hash, go to  `Developer` -> `Chain state` -> `dao` -> `proposals()` to see proposal index/description/link
- With the given hash, go to  `Developer` -> `Chain state` -> `dao` -> `proposalOf()` to see proposal action

## Step 6: farmer voting 

Once proposal is created farmers can vote for it. 

- Go to `Developer` -> `Extrinsics`
- Make sure the farmer account is selected.
- Select `dao` -> `vote()` extrinsic

The vote must include the following arguments:

- farmId: the farm id of the farmer
- proposalHash: the hash of the proposal
- approve: `Yes` or `No` the farmer approves the proposal

Further considerations:
- Vote is per farm so in case farmer owns other farms he could repeat the process for all of them.
- Vote can be changed at any moment until the proposal is closed.  
- Farmer can also vote via TF Dashboard in DAO section.

## Step 7: council member veto 

// TODO

## Step 8: closing proposal

After the proposal ends or, before it, if number of votes reached Threshold, it can be manually closed by a council member.

- Go to `Developer` -> `Extrinsics`
- Make sure the council member account is selected
- Select `dao` -> `close()` extrinsic
- Fill proposal hash and index (can be found by listing active proposals, see step 5 above) and submit

Then the proposal is removed from list and the action, if any, is executed on chain in case of approval.
Approval is obtain when 




Closing a proposal after the proposal ended. This will approve / disapprove the proposal and execute the call on chain. 



Provided:



![close](./img/close.png)