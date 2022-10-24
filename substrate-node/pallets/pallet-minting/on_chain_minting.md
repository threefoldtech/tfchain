# On chain minting

The [current minting code] is run as a separate binary run against an archive node.This code iterates over every block in the `period` (a predefined amount of time), to get a view of all events on chain, and finally calculates how much tokens are awarded for each node. A receipt is also created with this amount, plus some additional details about the node (such as the capacity of said node). Finally, the tokens are paid out on the stellar blockchain, using the hash of the receipt as memo (in a `TX_HASH` memo), with the stellar address to pay to extracted from the blockchain. The time has come to properly do the entirety of the minting on chain.

## Minting history

Minting has been going on for years now, and went through quite some changes. The concept has however been the same since the beginning. Essentially, there exists a list of "some `nodes`", having "some `capacity`", resulting in "some amount of tokens" being paid. Different versions of minting have done this slightly differently. The original minting, which worked with zos V1 and the original tfchain, calculated the amount of tokens for every node, then used special `Coin Outputs` on tfchain to mint completely new tokens. When we transitioned to zos V2 and stellar, a new minting repo was used, this time sending tokens on stellar. While no tokens are explicitly created, the issuing account of an asset on stellar can't hold any of said asset, but it can transmit an unrestricted amount of them to other accounts (provided they have a trustline to the asset in question). In the [current minting code] we still use this approach to mint on stellar, As the existing tfchain uses a bridge to convert stellar TFT assets to native TFT's on tfchain, and this bridge vaults on stellar side. This means that minting on tfchain still requires an equal mint on stellar, which has been a showstopper up until know.

Regardless of the actual method of payout, there has always been a set of rules (the `tokenomics`) which decide how much tokens a node gets. These rules have changed over time and are currently partially dynamic, as part of a `farming policy`.

## Periods

As said earlier, the minting happens per `period`. This is a concept which was introduced in the original minting. While `periods` are roughly similar to months, there are subtle differences. The duration of a `period` was chosen such that there were exactly 12 per year in the original 5 years of the minting (at this point, in the v1, nodes would only mint tokens for 5 years).

As such, the duration of a period is calculated as:

`60 seconds / minute * 60 minutes / hour * 24 hours / day * (365 days / year * 3 years / 5 year period + 366 days / leap year * 2 leap years / 5 year period) / 60 months / 5 year period`

This equals `2_630_880 seconds`.

The original `period` started at unix timestamp `1522501000` (`Sat Mar 31 02:56:40 PM CEST 2018`). This coincides with the creation time of the [genesis block of the original tfchain].

More info on working with periods can be found in the [period module] of the [current minting code].

## Current minting flow

The general flow of minting has not changed over the years. Essentially it comes down to:

- Checking the resources in a node
- Verifying the uptime during the period
- Calculating the node payout

Next to this, we also track utilization of a node over the period. While not actively used right now, the current `tokenomics` want to lock funds for 2 years unless 30% of the node capacity is utilized. Furthermore, network and IP addresses are rewarded based on "utilization" on/by a node.

Because we want to track this as it changed during the period, we currently fetch all nodes from the chain, as well as all farms, payout addresses (which are linked to the farm), and contracts. Then, we iterate over all events of every block in the period. At every block, we handle certain events which impact the minting. These are the new nodes (since we can scale payout if we know the node was not there at the start of the minting, as well as the required uptime to achieve the SLA, for which we need to know the point of connection), the node update, as we only payout the lowest amount of resources in a node during the period, uptime received events, which are used to calculate uptime, contract resources used events, which set how much resources a contract use on a node and is used to track static utilization, and nru consumption reports, which inform the chain that a contract used some amount of public network traffic (which is both billed to the user and rewarded in the minting to the node), and that the contract is live in general (thus allowing is to increment the used resources based on the resources set on the contract).

After this, we pull another 2 hours worth of blocks, since the uptime events which are needed to cover the last portion of the `period` will not be received until after this `period` ended.

With all this data aggregated, for every node, we calculate the `CU` and `SU` based on the amount of `cru`, `mru`, `hru` and `sru`. We also check the uptime, to make sure it is above the required SLA, as per the `farming policy` attached to the node. Currently this SLA is actually ignored, and payout is scaled according to actual achieved uptime. We then also add the reward for `NU` and `IP usage`. This allows us to calculate the node payout in `USD`, which we finally convert to an amount of `TFT` based on the `TFT price on node connect`. We then generate a receipt with required info, as well as a csv file with payment info for all nodes. The payment address is found by looking it up based on the farm in which the node resides at the end of the `period`.

## Current problems we want to solve

First, a minor problem is that we require a stellar address on tfchain linked to every farm. This address is used for the actual payout, and needs to be provided by the user. As such, there are some farms which have either no stellar address, or a wrong one (not every stellar address is a valid payment address). This results in the nodes in that farm not receiving tokens. Due to issues like these in the past, we implemented a mitigation which will trigger a `retry` of a payment for a node if the hash of the receipt is not found on stellar by the new minting. While this works and allows farmers to correct their info, it introduces a hard dependency on the stellar blockchain, which is not super ideal. By contract, if we mint on chain, we can simply mint to the address associated with the farmer twin, which is known to be valid and active as it was used to set up this twin.

Secondly, the current minting code is an external tool. Since upgrades on the chain are not backward compatible, this means that we usually maintain 2 versions of the client with a bunch of duct tape in between them to allow iteration over the history. This iteration, used to build state, is also just a comparison of 2 consecutive events for a node, which means that we can just keep track of a very small state and modify it in place (this is already what happens). In other words, the current approach is just a very large hassle which can be avoided by moving to the chain. While we could actively track every block as it happens, this also introduces problems, as updates are not clean on chain (i.e. we can't easily swap out external clients), and keeping track of a state in such a scenario is prone to data corruption.

## Technical details

Since the minting mainly operates based on events happening on chain, we can do this pretty straightforward through the introduction of a new `pallet`, and some event handlers for these functions. The `minting pallet` maintains a small amount of state, required for the payout calculations. We keep the state separate from the node object, since it will be frequently mutated, and not an actual property of the node. In any case, these hooks should do a minimal amount of work as to not overload the existing functionality on chain.

### Period management

While it is possible to do a continuous mint every time a node sends uptime, we will stick to minting per `period` as is currently the case. The reason for this is the `SLA` requirement. This is defined over a `period`, and if we mint every time there is an uptime event, it would not even make sense anymore. One decision to make is whether all nodes are using a fixed period, or if we track periods per node. The
later seems to be the better scenario, as it avoids a massive storm of payments all at once, and instead distributes them over time. By tying the periods to the node itself, instead of fixed points in time, we can also avoid the issue where we currently scale the first `period`, since the node always comes online for the first time at the start of its own `period`.

### Tracking uptime over a period

Tracking an uptime will be the same as the [current minting code], but instead of calculating it after a while we keep track of the current uptime during the `period`, and reset it once the `period` threshold has been crossed. For calculating actual uptime, we need 2 data points: the previous one, and the current one. In both cases, a data point contains the uptime as reported by the node, and the chain time the report was processed (which is requested from the runtime). For a healthy, on-line node, we expect the difference between the reported uptimes to be equal to the difference between the timestamps of the report. Note first that the block time is 6 seconds, so there can be a slight mismatch here. Secondly, there might be some network latency. As such, it is required to implement a small window for which the report will be accepted, in which case the uptime increment is equal to the difference of uptimes. A valid window could for example be 1 minute. In other words, if the difference of uptime is `Δuptime` and the difference of report times is `Δr`, then a report is
valid if `Δr - 60 <= Δuptime <= Δr + 60`. We choose `Δuptime` as the amount to increment uptime because this is the amount reported by the node and should be free of any latency issues w.r.t. network or block production.

It is also possible for a node to reboot. In this case, the node sends an uptime report, at some point reboots, then sends another uptime report. This case is signaled by `uptime < Δr`. Notice that we don't care about the uptime difference here. Since we claim that nodes shouldn't reboot on the grid, we consider a reboot an abnormal situation and the time which we don't know about will be considered downtime. It is the job of `zos` to properly send a final uptime report before a planned reboot to keep this time to a minimum. In this case, we can consider `Δr - uptime` as downtime for the node, and `uptime` as uptime (with `uptime` being the uptime reported by the node).

Other cases are generally not possible and point to faulty or fake nodes. The principal case is `Δuptime > Δr + 60`, which indicates time on the node would be flowing faster than time on the chain. This is not possible, as latency or block production stalls would lead to the inverse, i.e. time on chain advanced more than the difference in uptime. The only scenario where this could happen is if a previous uptime report was considerably delayed, and the node sends on a fixed timer. However, we will reject these reports for minting purposes, and require the node to send a new one which is valid. As such, we will penalize a node in this case.

The other case where `Δuptime < Δr - 60` likely means a delay in uptime reporting. We observed this behavior due to network latency already in the past. There is also the possibility of a block production stall while the transaction is in the transaction pool. Since we can't rule out intermediate issues (or issues on the chain end), we simply require the node to send uptime again. In an ideal world we would check if it is only limited nodes which have issues (indicating network issues which would mandate slashing of minting rewards), or the majority of nodes (which would mean a chain problem which we will ignore). Since this is a pretty stand-alone expansion, we will ignore this for now and leave it as a "future improvement".

As explained, we keep track of last reported uptime and last report timestamp. Next to this we keep track of the start of the `period` (this is needed to know when a new `period` begins), and the current accumulated uptime. We don't need to explicitly keep track of downtime as this is easily calculated as `last uptime report - period start - uptime`.

The `period` ends when an uptime report is accepted (i.e. one which is not lagging behind), and current time on chain is bigger than the recorded period start of the node + period duration. If this happens, we calculate downtime and uptime as usual, and check how many seconds are left in the "old" `period`. We then subtract this value from downtime of the report first (as downtime necessarily predates uptime),  and if that is not sufficient, subtract from uptime as well. At this point we have the full uptime of the node over the `period` and can mint. There is 2 options, either we mint immediately, which is fine because the current code is still called as part of a submitted extrinsic (and as such will just be retried if the current block is too heavy), or we collect and store this data for a future dedicated extrinsic
call to trigger the actual minting. This means we use a bit more storage space, but at the flip side we don't encumber the uptime report extrinsic too much. While the logic is similar in both cases, this doc will for now assume option 2: save a snapshot of data for a later "Mint" extrinsic to process. In either case, the minting info which is tracked is updated, such that period start is advanced by period duration, accumulated uptime is reset, and incremented with the remainder of the calculated uptime from the report, and last reported uptime is set as usual.

This part mostly handled the uptime report logic. Notice that this is currently part of the `Tfgrid module`, but only serves as an extrinsic emitting an event for the current minting logic. Therefore it would be best to move this extrinsic to the new minting pallet. To to this, we implement the extrinsic here, and add a hook to the existing function which calls this extrinsic. This way existing nodes can gracefully
migrate to the new extrinsic without any downtime or failure. In a future update the existing extrinsic can then be removed.

### Node updates

When a node is updated, this has impact on the minting. Specifically, the minting cares about changes in resources in the node. Unfortunately, this is not covered by the current tokenomics, so until that is resolved, we will take a best effort approach which is deemed
reasonable. Keeping in mind that minting wants to reward "usable" capacity, we will reward the minimum amount of capacity a node had
during a `period`, as this is the capacity which was available at all time. This is also what the current minting does. There is however a
more subtle issue. As per the tokenomics, a node has a "connection price". This actually refers to the hardware of the node at that time.
For instance, if we connect one HDD at 0.08 USD, then connect a new HDD while the connection price is 0.16 USD, the second HDD should not get rewarded similarly to the first HDD. While we could keep an array of capacity connected at certain points, the problem then shifts to capacity being removed. For example, in the above scenario we have 2 HDD's, one connected at 0.08 and one at 0.16. Now imagine one of them is removed or breaks down. Capacity will be reduced to 1 HDD. But we have no idea which one is removed. It can be the original HDD, which would mean more tokens are removed due to the lower connection price, or the second one, which leaves the original HDD in the minting which pays more, while this might not be the actual case.

The straightforward solution to this would be to take make a copy of the capacity of a node for the minting, and never allow more than this capacity to be rewarded. This means upgrading a node effectively becomes pointless for farmers (in the current situation). As a middle road, we can allow upgrades as long as the current connection price on chain is equal to the connection price as registered by the node. At this point, it would allow a farmer to upgrade his node anyhow by removing the old one, adding capacity, and booting it as a new node getting a new node ID.

Putting this together, the minting will track 2 sets of resources for nodes. The first one will be the "maximum" resources the node can be
rewarded for. The second one will be the lowest resources in the period. Resources here are the collection of `cru`, `mru`, `sru`, and `hru`. Technically it is possible for old hardware connected at a lower price (part of the maximum resources) to break, and new hardware which replaces it to be connected at a higher connection price, which means it should get less tokens, but this seems like a fair trade off here. Notice that we reward the minimal capacity during the `period`, meaning that we fetch the actual capacity again from the node itself when a `period` is finished and a new one starts (see above). By extension, the maximum capacity can only ever increase as result of the node updated handler, and the minimal capacity can only ever decrease (as the reset is triggered by a different handler).

The one thing to think about here is that the reset of the minimal capacity needs to account for the maximal capacity, such that every part of the minimal capacity is lower or equal to the maximal capacity, and gets reduced to this level should this not be the case. Similarly the maximum capacity can only increase if the current connection price is lower or equal to the connection price of the node, as explained above (it is assumed the DAO will never reduce connection price for new nodes to protect the network, hence equal should be sufficient, though it is technically possible for this to happen, therefore less than or equal is the better choice here).

When the period ends and the minting info is saved, the minimal resources are saved as part of it. These will be used to calculate how
much a node is rewarded.

### CU and SU calculation

`CU` and `SU` are the units which are actually rewarded, and are calculated from the raw resources. Calculation happens as part of the actual minting call based on the info saved previously (the minimal resources during the `period`). By delaying calculation to the actual minting call, we save a bit of weight on the report uptime extrinsic, and can group as much of the actual minting code as possible together.

For reference, the formulas to calculate these values are as follows:

- `CU = MIN(cru * 4 / 2, (mru - 1) / 4, sru / 50)`
- `SU = hru / 1200 + sru * 0.8 / 200`

IMPORTANT: in the above, `mru`, `hru`, and `sru` are expressed in `GiB` notation. This is required to keep in line with the existing minting. `1 GiB = 1024 ** 3 == 2 ** 30 == 1 << 30`

### Node resource usage and IP usage + NU

As part of the minting we also calculate the utilization of a node over the `period`. This is required for the locking of tokens (which is part
of the tokenomics but not part of the current minting implementation as this lives on stellar currently), and potentially in the future for the
implementation of boosters. Because the "utilization of the node" is not defined further in the tokenomics, we won't bother with that now, and instead just keep track of the utilization of the raw capacity. To do this, we calculate the `unit seconds` that resources are available and have been used. The current minting does this by keeping a counter for resources used by contracts. At every NRUConsumptionReport, the contract resources are looked up for said contract, multiplied by the window of
the report, and added to the used unit seconds. At the end of the `period`, utilization for every resource is then simply calculated as
`used resource seconds / (node resources * period duration)`. Notice that there is an issue here. We only care about the actual resources of the node at the end of the `period`. This means that the available resources of a node can be reduced at the end of the period to artificially inflate the calculated utilization, at the cost of getting less tokens for the `period`. This currently does not make sense, but could become interesting if locks are implemented which unlock earlier if a given utilization is achieved, depending on market circumstances.

We also can't compare against maximum capacity, as this would punish farmers who don't replace broken hardware in their node. The solution thus is to keep track of the total capacity in a node, again expressed as `unit seconds`, by tracking it as the node reports its uptime. This way we can follow reduction in capacity of the nodes as they happen, and we can have an accurate utilization of resources by workloads as they are available. This approach also has a drawback though. If only capacity is accounted for while the node is online, we don't correctly represent the reduction in usage which follows from a node being offline. This can easily be resolved however by adding capacity for offline nodes as the same capacity the node has when it is offline. It does not matter that this might be lower than the actual capacity, as we won't credit utilization for the downtime anyway, due to lack of reports of contracts.

In a similar fashion, we keep track of the IP usage on the node. To do this, when a contract sends an NRUConsumptionReport which is eventually handled by the minting, we check the amount of public IP's on the contract, and multiply those by the contract window, which is then added to a counter for the node in the minting code. This is just a running counter.

Finally, public traffic network consumption is reported by the nodes. Again we can just add this value as is.

At the end of the `period`, all these counters are saved on the minting info for the actual minting to happen. The nodes currently report
network consumption as `bytes`, whereas we want an `NU` for the minting. The conversion is simply `1 NU = 1GiB of public traffic`. Similarly, the minting awards "hours of public IP used" on a node, here we can just divide the previously mentioned counter by 3600 to get the amount of "IP hours used". This value can be more than the duration of the `period` if multiple IP's are assigned on a node, this is completely intentional.

Wrapping things up, the following data is tracked (and added to the minting info when a period ends):

- The total capacity of a node in `unit seconds`
- The actually used capacity of a node in `unit seconds`
- The amount of IP addresses assigned to the node in `unit seconds`
        (conversion can be done in the mint extrinsic)
- The amount of bytes of public traffic used on the node (conversion to
        `NU` can be done in the mint extrinsic)

#### Rent contracts resource usage

Rent contracts technically mean a whole node, and thus all of its resources are rented. We can't just blindly add all resources of the
node for the window if the contract happens to be a rent contract, as regular contracts on top of the rent contract would then inflate the
usage above 100%. While this is possible for CPU, as that is overprovisioned, it makes no sense for the other resources. This means
we need to prevent regular contracts from adding consumption if a rent contract is used.

This is technically challenging, as the proposed design means every NRUConsumptionReport would require checking if the node has a rent contract on it, and if so ignore this contract. Instead, we can keep a flag which tracks if a rent contract is present on the node, and rely on the fact that only a single rent contract can ever be available per node. We then hook the contract create and delete functions, to toggle this flag at the right time. If the flag is present, regular contract reports can be completely ignored, and only account for the rent
contract. If the rent contract gets a consumption report, the node resources are loaded, and the appropriate amount of consumption is
credited on the node. This does mean that we won't ever achieve more than 100% CPU utilization, but that is fine.

This approach could also be used to keep track of the total sum of resources reserved by contracts on a node. In that case we can simply increase the consumption counters on uptime reports. However this approach has the downside that we will sometimes credit too much (we lose track of when the contract started exactly, so the initial uptime report will erroneously add utilized capacity for the time between the previous uptime report and the contract creation), and sometimes too little (we won't credit the last part of utilization if a contract is deleted, the part between the last uptime report and contract deletion). The latter can be fixed by adding the missing utilization when the contract delete is handled, though handling the other case is harder and will add too much complexity, so we won't take this route.

### Farming policy

Because the minting info is only saved once the next uptime report for the node is received, it is technically possible that the farming policy for the node changes after the `period` has technically ended, but before the minting info is saved. As such, we also keep track of the farming policy ID on the minting info, and update it accordingly. If the policy ID is updated after the `period` finished (i.e. `current chain time > period_start + period duration`), we ignore it. Once the minting data is saved and reset for the following `period`, we fetch the existing farming policy ID from the node and set that. This means we will always mint with the latest farming policy attached to the node in the `period`.

### Punishment for misbehaving nodes

While not touched in the tokenomics, a node could be misbehaving or actively malicious. While detecting that is currently out of scope of
the minting, there is one thing which could be detected, being faulty uptime reports as laid out above. The current minting code simply flags this, and reports a "violation". It then also does not pay the node for the `period` where this happens. On chain we can do a similar thing, where we track an optional violation. If one is found, the mint extrinsic can report this violation (as an error or some kind of mint
failed because x/y/z event). Other than this, it is up to the tokenomics to define.

### VM's

Perhaps the most simple of cases, if a node is considered to be virtualized (which we already track), there is no payout. In this case,
we also don't need to do any work or track anything, as there is no point in it.

### Carbon offset TFT's

At some point it was decided that we would mint additional TFT based on the capacity of nodes and their uptime, to offset the carbon footprint of these nodes. These TFT's are minted next to the actual minting, and sent to a specific wallet, to eventually be sent to a third party (TAG?). The wallet in question is [GDIJY6K2BBRIRX423ZFUYKKFDN66XP2KMSBZFQSE2PSNDZ6EDVQTRLSU]. Currently
sitting on a casual 2 million TFT, there does not seem to be any activity. If this partnership is not active, we can refrain from implementing this. Until such time where this is confirmed though, the calculation for tokens received by this wallet is:

`(CU * ίCU + SU * ίSU) / node connection price * uptime / period duration`

Note that the amount is _always_ scaled to the actual uptime of the node. The constants are given as:

`ίCU = 354` and `ίSU = 122`.

### Actual minting

With the concerns of the minting laid out, we eventually produce "minting info" objects which we store on chain, containing all the info
above. The one thing missing is the farm ID, which also needs to be saved. Technically this can be done in the same way as the farming
policy ID. The reason this needs to be saved is that a node can be moved between farms, and in a case similarly to farming policy ID's, we want to know the last farm the node was in before the `period` ended. We only need this so we can get the address from the farm twin, as this is the wallet which will receive the tokens. We could instead opt to store this address, though that is slightly more work as we need to follow the indirection from farm ID to farm to twin. Also, by only saving the farm, we will pay out to the _current_ twin address, which might be different than the one at the end of the `period`.

The mint itself is fairly straightforward. We [calculate CU and SU](###CU-and-SU-calculation), calculate the reward in `USD` according to the attached farming policy, and finally convert this to the appropriate amount of tokens according to the connection price of the node. This amount is then transferred out of the void to the address of the twin of the farm. All while an event is emitted giving full details of the mint: raw stats, calculated stats, and eventually calculated payout.

The final thing is accounting for uptime. The original spec claims all nodes _must_ adhere to some SLA. Since the policy includes the SLA, we can easily check that `uptime * 1000 / period duration >= policy SLA`. Note times thousand as SLA in the policy is expressed permill instead of percent (also note that policy 1 and 2 still have the faulty percent indication, which needs to be changed). However currently the minting does a scaled payout. If that is still required when this launches, we will need to hard code an exception on policy 1 and 2 which instead applies uptime as a scaling factor. This is fairly trivial though.

### Payment trigger

The flow provides for the chain to collect all required data, separate for each node, while still adhering to the concept of `periods`. Calculation of payout is fully independent of any user input. As such, the mint extrinsic can be callable by anyone. This means we can do multiple things for the actual payment trigger, ranging from controlled by us, to automated, to user claims. Next to this, observe that not all nodes will get the minting info and thus their tokens at the same time. For existing nodes this will be largely batched, for new nodes this will depend on when exactly they come online. For payment triggers, a non exhaustive list follows:

1. Maintain control by the foundation. Currently 5 people sign the payouts after they are generated. If this is to be done on chain, we can change the mint extrinsic to not do a calculation, and instead just take an off line calculated amount based on the stored input. We also require multiple parties to input the same amount (e.g. 5 as currently out of X which are allowed to give input). The extrinsic
merely saves the input amount, irrespective of the actual data, together with the address (i.e. key) of the submitter). Of course
only authorized addresses can call this. Then, after every submission, the list of amounts is checked, unique entries summed
per amount, and if an amount has sufficient entries, the mint is executed for this amount. In other words, submitting an amount
equals voting for this amount to be paid out. This improves the current situation, as "signers" don't need to sign the output of the
previous person, and can sign independent of one another. It also means that bugs in the payout calculation can be fixed off chain
quickly.
2. Similar to the previous option, but the mint extrinsic calculates the value of the mint itself, and only keeps track of votes which have an amount equal to the self-calculated one. This means if there is a bug to on chain payment calculation, it will still not result in any faulty payments, but it will be harder to fix as we need to update chain code. On the flip side, the community has a guarantee that the signers can't collaborate to give them lower payouts.
3. We [leave the mint extrinsic as described](###Actual-minting). Because there is no user input, everyone can call it. We run a small service ourselves which listens for events of minting info saved (the minting `period` for a node ended, info was saved for the mint extrinsic to be called, and new `period` started). Minting happens almost immediately, we pay a small transaction fee for every person. This service can be an off chain worker as well.
4. We don't do anything, instead relying on users to claim their own funds. Either a community member can run a service as described in the previous option, or we add functionality to the dashboard and TF connect to alert farmers that they have pending mints, at which point they can claim them. Note that the farmer twin needs to have funds to do this, but those should be there anyway since they are needed to create a farm. After the first payment, sufficient TFT should be present anyway. If there turns out to be a problem, we can still force the mint on our end.

### Smooth transition from current flow

The current minting has 2 things which we need to account for. These are fixed `periods`, and `receipts`. Receipts will be replaced by elaborate events when the minting happens, which will contain all the data the current receipts have. These events can then easily be extracted from graphql by whoever wants, possibly to build a UI on top.

The fixed `periods` also make it easy to transition from one system to the other. Once the code is complete, we decide when to transition. This will be at the boundary of a `period`. We then add checks in the minting code on chain to ignore most events from before this timestamp, only updating the last report received and last reported uptime fields. After the timestamp we handle extrinsics as described. In a future update, the checks can be removed. We then do a final mint off chain while the minting code is running and takes over.

### Caveats

Calculations regarding utilization won't be completely accurate. The cause of this is that we track this as part of contract NRU consumption events, which span the `period` boundaries. In essence, some consumption or utilization will be reported too early or too late, depending on how things go. The current minting also has this issue, but we choose to ignore it as the effect is marginal (around 0.1%) of the contract utilization. Also, the issue is only present if you check individual periods, as the consumption and utilization will be correct when amortized over multiple periods which span the duration of the contract. This means that __all__ public traffic and public IP used will be rewarded, and all consumption of resources will be visible, but a marginal portion might be credit to the wrong `period`. For our interest however this does not matter

## Problems to properly do minting on chain

The main issue is token issuance. While we can perfectly mint tokens on tfchain, the bridge between stellar and tfchain vaults on the stellar side. This means that all tokens are present on stellar, and the tft on tfchain is essentially just "something which represents a token vaulted by the bridge". Since these tokens can be bridged back to stellar at any time, it is important that the stellar bridge vault always has sufficient tokens available to allow this. There are multiple solutions to this problem:

- Create a small separate service which monitors the chain and listens for `Mint` events which would be emitted when tokens get minted for nodes. This would allow the service to transfer the exact amount of tokens to the bridge vault. The problem here is that we need to create this service, though that should be trivial, and most importantly that it needs to keep running, as missed events or other failures will cause a discrepancy in the amount of vaulted tokens. A link to the actual mint can be inserted in the memo.
- Do a periodic "fund" of the vault based on minted tokens in some past time. This can be a mostly manual process, as we just need to list and sum all `Mint` events to determine the amount. The problem here is that it is hard to link the transfer to the vault to the actual `Mint` events.
- Just send arbitrary amount of tokens to the vault and accept that the vault and tfchain won't be in sync, but ensure there are always sufficient tokens for withdraws. This means we lose the 1 to 1 relationship, but avoids a bunch of other hassle.

Aside from this, we also lose a bunch of flexibility for changes in the minting. Specifically, implementation of the boosters is going to be more of a hassle, though we wanted to do on chain minting for a long time now, and overall this fully automated solution will still be a massive quality of life improvement over the current situation.

## Future work

As mentioned, locking is a part of the tokenomics currently in use. As it is not currently done, and since adding it won't require changes in the given flow, it was not discussed. Technically, locking on tfchain is fairly easy, though we have to keep in mind that the existing locking infrastructure is not suitable (it overlaps rather than stacks). Still, adding locks if and when they happen should not be a huge bother. On a technical level, we don't even have to lock, we could also opt for a delayed payout of mints. All in all, how to do that is currently outside the scope of this story.

[current minting code]: https://github.com/threefoldtech/minting_v3
[genesis block of the original tfchain]: https://explorer2.threefoldtoken.com/block.html?height=0
[period module]: https://github.com/threefoldtech/minting_v3/blob/master/src/period.rs
[GDIJY6K2BBRIRX423ZFUYKKFDN66XP2KMSBZFQSE2PSNDZ6EDVQTRLSU]: https://stellar.expert/explorer/public/account/GDIJY6K2BBRIRX423ZFUYKKFDN66XP2KMSBZFQSE2PSNDZ6EDVQTRLSU