## Releases

Releases are automated by [this workflow](.github/workflows/030_create_release.yaml). When a release should be created following things need to be done:

- substrate-node
    - Increment spec version in the runtime [lib.rs](../../substrate-node/runtime/src/lib.rs) 
    - Increment version in [Cargo.toml](../../substrate-node/Cargo.toml)
    - Increment package `version` filed in [Chart.yaml](../../substrate-node/charts/substrate-node/Chart.yaml)
- tfchainbridge
    - Increment chart `appVersion` filed in [Chart.yaml](../../bridge/tfchain_bridge/chart/tfchainbridge/Chart.yaml)
    - Increment chart ` version` filed in [Chart.yaml](../../bridge/tfchain_bridge/chart/tfchainbridge/Chart.yaml)
- activation-service
    - Increment chart `appVersion` filed in [Chart.yaml](../../activation-service/helm/tfchainactivationservice/Chart.yaml)
    - Increment chart `version` filed in [Chart.yaml](../../activation-service/helm/tfchainactivationservice/Chart.yaml)
    - Increment package `version` in [package.json](../../activation-service/package.json)
- Js TFchin Client
    - Increment package `version` in [package.json](../../clients/tfchain-client-js/package.json)
- Scripts
    - Increment package `version` in [package.json](../../scripts/package.json)
- Tools/fork-off-substrate
    - Increment package `version` in [package.json](../../tools/fork-off-substrate/package.json)

- Commit the changes
- Create a new tag with the version number prefixed with a `v` (e.g. `v1.0.0`, `v1.0.0-rc1` for release candidates) 
- Push the tag to the repository
- The workflow will create a release draft with the changelog and the binaries attached

A changelog will be generated based on the Pull requests merged, so having PRs with meaningful titles is important.

## Validate a runtime

See [validate](../misc/validating_runtime.md) for instructions on how to validate a runtime.

### Upgrade runtime

To upgrade the runtime for a network based on a release, download the runtime attached to the release (tfchain_runtime.compact.compressed.wasm)
and upload it to the network using a council proposal. The proposal should be a `set_code` proposal with the runtime as the code and majority of the council should vote in favor of the proposal.

