# Upgrading runtime

## Check first

- Create a release first, runtime binary will be uploaded to the release on github.
- Make sure CI (build & tests) passes
- Test migrations (if any) with the tools described in [try-runtime](../misc/try_runtime.md) and [tools](../../tools/fork-off-substrate/README.md)

## Download the latest release from github

Download the latest release from github tfchain repository release page.

## Propose a runtime upgrade using the Council

Note: this can only be done if you are a council member. See [council](../misc/council.md).

Browse to the Polkadot UI and connect to the Node, select the `Governance` -> `Council` page 

Select `Motions` on top and click `Propose Motion` on the right.

Now select:

- Account (your account)
- Threshold (Super Majority)
- Proposal: `runtimeUpgrade` -> `setCode` (upload wasm file)

Now click `Propose`

[example](../assets/propose.png)

## IMPORTANT

If a type change occurred in the modified runtime pallets, all objects stored under that type will be invalidated unless a runtime storage migration is executed.

There are several ways to work around that but none are advised. More information on runtime upgrades can be found [here](https://substrate.dev/docs/en/knowledgebase/runtime/upgrades).

### Upgrade grapqhl

[graphql upgrade documentation](../graphql/docs/readme.md)
