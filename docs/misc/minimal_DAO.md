# DAO on TFChain

Based on https://docs.google.com/document/d/1qK51Dsg4jj3FJ76CWeh-WdNftQPFV8CtBmpLTlqaFk4/edit#

The goal is to have a minimal DAO implemented on TFChain where farmers can vote on proposals. Only farmers can vote (see below), and only selected people (council, see below) can submit a proposal to be voted on.

## Council

A council member is a person with slightly elevated privileges, represented by an account on chain. It is intended that this person acts in the interest of the grid/chain. There can be at most 12 council members at once (though this should be configurable in the code). A council member has the following abilities on chain:

- Create a new voting proposal.
- Link an existing farming policy to a farm, with suggested limits (see farming policy section).
- Council only vote (nobody except council member can vote for this) to add or remove a member. For the result of this vote, strict majority is used, i.e. vote passes if `votes_for > votes_against`. There must also be a time limit on the vote, any council member who doesn't vote before this time implicitly abstains. Note that in case of removal, the member to be removed can vote against that (no special logic to prevent this).
- Set the certification level of a farm (see below).

## Certified node verifier

This is an entity (person, group, ...), represented on chain by an address, which can mark an existing node as certified. Adding addresses to this list is done through a regular vote. Any address in this group can mark any node as certified (and unmark a node as well). This means:

- A list needs to be kept with addresses which can certify nodes.
- The origin on the certify nodes call needs to be changed to allow any address in this list to call it, the current restricted origin can just be removed.

## Farming policy

A farming policy defines how farming rewards are handed out for nodes. Every node has a farming policy attached. A farming policy is either linked to a farm, in which case new nodes are given the farming policy of the farm they are in once they register themselves. Alternatively a farming policy can be a "default". These are not attached to a farm, but instead they are used for nodes registered in farms which don't have a farming policy. Multiple defaults can exist at the same time, and the most fitting should be chosen.

A farming policy has the following fields:

- id (used to link policies)
- name
- Default. This indicates if the policy can be used by any new node (if the parent farm does not have a dedicated attached policy). Essentially, a `Default` policy serves as a base which can be overridden per farm by linking a non default policy to said farm.
- Reward tft per CU, SU and NU
- Minimal uptime needed in percentage (can be decimal e.g. 99.8%)
- Policy end date (After this data the policy can not be linked to new farms any more)
- If this policy is immutable or not. Immutable policies can never be changed again

Additionally, we also use the following fields, though those are only useful for `Default` farming policies:

- Node needs to be certified
- Farm needs to be certified (with certification level, which will be changed to an enum).

In case a farming policy is not attached to a farm, new nodes will pick the most appropriate farming policy from the default ones. To decide which one to pick, they should be considered in order with most restrictive first until one matches. That means:

- First check for the policy with highest farming certification (in the current case gold) and certified nodes
- Then check for a policy with highest farming certification (in the current case gold) and non certified nodes
- Check for policy without farming certification but certified nodes
- Last check for a policy without any kind of certification

Important here is that certification of a node only happens after it comes live for the first time. As such, when a node gets certified, farming certification needs to be re-evaluated, but only if the currently attached farming policy on the node is a `Default` policy (as specifically linked policies have priority over default ones). When evaluating again, we first consider if we are eligible for the farming policy linked to the farm, if any.

### Limits on linked policy

When a council member attaches a policy to a farm, limits can be set. These limits define how much a policy can be used for nodes, before it becomes unusable and gets removed. The limits currently are:

- CU. Every time a node is added in the farm, its CU is calculated and deducted from this amount. If the amount drops below 0, the maximum amount of CU that can be attached to this policy is reached.
- SU. Every time a node is added in the farm, its SU is calculated and deducted from this amount. If the amount drops below 0, the maximum amount of SU that can be attached to this policy is reached.
- End date. After this date the policy is not effective anymore and can't be used. It is removed from the farm and a default policy is used.
- Certification. If set, only certified nodes can get this policy. Non certified nodes get a default policy.

Once a limit is reached, the farming policy is removed from the farm, so new nodes will get one of the default policies until a new policy is attached to the farm.

## Farm certification

Certification of a farm will be changed from simple boolean (certified or not) to an enum (representing a different lvl of certification. For now there will only be `NotCertified` and `Gold`.

### Gold farm certification

Gold farm certification will be defined as follows (from https://forum.threefold.io/t/gep-gold-certified-farming-specs/2925):

- Hardware purchased from a recognized vendor (approved by DAO - initially HPE & ThreeFold itself)
- Minimum 5 IPv4 addresses per server
- Two power supplies
- Two routers per rack
- Two Internet Service Provider connections
- Tier 3 or 4 data center certified to ISO 27001
- Uptime 99.8%
- Network connection at least 1GBit/sec
- Geographic decentralization - no more than one full datacenter rack per town unless and until utilization in that rack is > 50%.
- The 3Nodes need to be certified (can be done by TFTech or other actors on blockchain)

Since all of these requirements can not be checked on the chain itself, it is simply up to the council members to verify this and make sure this remains the case after certification. If not, certification is reverted.

Important: we will also reserve some space for some text in the certification status. This way we can (later) insert a link to a webpage where related documents proving this can be found (e.g. image of ISO 27001 certification).

## Treasury

Currently 50% of the contract cost goes to a "certified sales channel" which is just an account. This will be removed in favor of a treasury. The treasury is essentially an account (though it should be a special account with no private key so it can't be spent from). Spending from the treasury happens through means of a vote proposed by the council. The description in the proposal should indicate the reason of the spending.

## Solution reward

A "solution" is something running on the grid, created by a community member. This can be brought forward to the council, who can vote on it to recognize it as a solution. On contract creation, a recognized solution can be referenced, in which case part of the payment goes toward the address coupled to the solution. On chain a solution looks as follows:

- Description (should be some text, limited in length. Limit should be rather low, if a longer one is desired a link can be inserted. 160 characters should be enough imo).
- Up to 5 payout addresses, each with a payout percentage. This is the percentage of the payout received by the associated address. The amount is deducted from the payout to the treasury and specified as percentage of the total contract cost. As such, the sum of these percentages can never exceed 50%. If this value is not 50%, the remainder is paid to the treasure. Example: 10% payout percentage to addr 1, 5% payout to addr 2. This means 15% goes to the 2 listed addresses combined and 35% goes to the treasury (instead of usual 50). Rest remains as is. If the cost would be 10TFT, 1TFT goes to the address1, 0.5TFT goes to address 2, 3.5TFT goes to the treasury, instead of the default 5TFT to the treasury
- A unique code. This code is used to link a solution to the contract.

This means contracts need to carry an optional solution code. If the code is not specified (default), the 50% goes entirely to the treasury (as is always the case today).

Note that a solution can be deleted. In this case, existing contracts should fall back to the default behavior (i.e. if code not found -> default).

## Voting

### Voting weight

As mentioned anyone can vote. To calculate vote weight:

- Get all linked farms to the account
- Get all nodes per farm
- Get CU and SU per node
- Weight of a farm is 2 * (sum of CU of all nodes) + (sum of SU of all nodes)

Weight should be tracked per farm to keep it easy and traceable, so if an account has multiple farms the vote will be registered per farm.

### Voting process

First, a council member needs to suggest a proposal (see below for valid topics). Once a proposal is submitted it can be voted on by farmers (technically everyone can vote, but if an address attempts to vote with no attached farms, or no nodes in the farm i.e. weight 0, an error should be returned to indicate this). The exception are council members. 3 council "no" votes will veto the proposal. If a member of the council, without farm, votes no, the no vote needs to be stored. On 3 no votes the proposal is automatically closed. Should this happen, an event should be emitted declaring a council veto, with the address of the no voters. Otherwise, if the time limit on the proposal is reached, a simple majority decided (vote weight for > vote weight against). Anyone who does not vote before the proposal is closed is implied to abstain, and does not contribute. Lastly, a minimum number of votes can optionally be set. In this case, it is the number of farms that need to vote. If by the end of the vote the amount of votes is not reached, the vote fails due to insufficient interest. Only votes made by actual farm(er)s are counted for this.

Storage wise, this should look something like:

- a map to track the farms voting in favor with their weight.
- a map to track the farms voting against with their weight.
- a vector of council no vote addresses.

After the vote is closed (assuming it is not veto'd by the council), we first check if sufficient farms have voted. This is done by simply counting the farms voting in favor and the farms voting against, and checking if this sum is greater than or equal to the configured minimum votes in the proposal.
If it is a chain action, it should automatically be executed after the vote passes.
Importantly all proposals have a description.

## Possible proposals:

- Change pricing rules (i.e. change the active pricing policy).
- Add a new farming policy.
- Change an existing not immutable farming policy.
- Upgrade for the chain.
- Add a certified node validator.
- Remove a certified node validator.
- Spend from the treasury.
- Register a new solution
- Delete an existing solution
- "Generic" proposal, this is not reflected on the grid, but rather just something with a description.
