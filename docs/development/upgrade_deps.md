# Upgrading substrate dependencies

Since substrate is still in active development, we need to upgrade our dependencies from time to time. 

It's easy to track which release of substrate is used in production (Polkadot) here: https://github.com/paritytech/polkadot/releases

## Comparing between latest substrate and tfchain

Check what is changed in the latest polkadot release and see if any of the pallets we use are affected.

If for example our version is: 0.9.30 and the latest version is 0.9.35 you need to check the last 5 releases and see if anything major was changed.
Anything major requires a migration and sometimes migration are removed between 0.9.x versions. Usually you can find this information in the release notes.
If anything major was changed and the migration was removed in the next version, you need to either:

- Add the migration yourself manually
- Upgrade to the version with that migration and then upgrade to the latest version

## Upgrading substrate

If you want to upgrade substrate, you can do so by changing the version in the `substrate-node/Cargo.toml` file.

Find and replace the version number with the latest version number. Build and test the node.