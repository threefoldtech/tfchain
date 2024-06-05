# Releases

Releases are automated by [this workflow](.github/workflows/030_create_release.yaml).
When a release should be created following things need to be done:

## Version bump

This step can be done using Makefile or manually.

### Makefile

see makefile [here](../../Makefile)

#### Usage

```bash
type=patch make version-bump # incrment the patch version
type=minor make version-bump # incrment the minor version
type=major make version-bump # incrment the major version
```

This function will take care also of branch and commit creation.
Review the changes and push them, then follow the steps 3 and 4 below to finish the release.

Important: This function will also incrment the spec version in the runtime.
If you already did this in another commit you need to undo spec_version changes to avoid double incrment.

### Manually

1 - Create a new branch for the release, increment the version for all components in the monorepo.
Here is a list of the files that need to be changed to make the release:

* substrate-node
  * Increment spec version in the runtime [lib.rs](../../substrate-node/runtime/src/lib.rs)
  * Increment version in [Cargo.toml](../../substrate-node/Cargo.toml)
  * Increment chart `version` filed in [Chart.yaml](../../substrate-node/charts/substrate-node/Chart.yaml)
  * Increment chart `appVersion` filed in [Chart.yaml](../../substrate-node/charts/substrate-node/Chart.yaml)

* tfchainbridge
  * Increment chart `version` filed in [Chart.yaml](../../bridge/tfchain_bridge/chart/tfchainbridge/Chart.yaml)
  * Increment chart `appVersion` filed in [Chart.yaml](../../bridge/tfchain_bridge/chart/tfchainbridge/Chart.yaml)

* activation-service
  * Increment chart `version` filed in [Chart.yaml](../../activation-service/helm/tfchainactivationservice/Chart.yaml)
  * Increment chart `appVersion` filed in [Chart.yaml](../../activation-service/helm/tfchainactivationservice/Chart.yaml)
  * Increment package `version` in [package.json](../../activation-service/package.json)

* Js TFChain Client
  * Increment package `version` in [package.json](../../clients/tfchain-client-js/package.json)

* Scripts
  * Increment package `version` in [package.json](../../scripts/package.json)

* Tools/fork-off-substrate
  * Increment package `version` in [package.json](../../tools/fork-off-substrate/package.json)

2 - Commit the changes

3 - Create a new tag with the version number prefixed with a `v` (e.g. `v1.0.0`)

4 - Push the tag to the repository

The workflow will create a release draft with the changelog and the binaries attached

The generated changelog will be based on the merged Pull requests, so having PRs with meaningful titles is important.

## Validate a runtime

See [validate](../misc/validating_runtime.md) for instructions on how to validate a runtime.

## Upgrade runtime

To upgrade the runtime for a network based on a release, download the runtime attached to the release (tfchain\_runtime.compact.compressed.wasm) and upload it to the network using a council proposal.
The proposal should be a `set_code` proposal with the runtime as the code and majority of the council should vote in favor of the proposal.
