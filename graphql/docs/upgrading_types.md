# Upgrading graphql

Scenarios:

- Custom Runtime Type has changed
- Custom Runtime Type has been added
- Custom Runtime Type was removed

## Process

### Update api client

Repo: https://github.com/threefoldtech/tfgrid-api-client

In any case, first start by editing the api-client library https://github.com/threefoldtech/tfgrid-api-client/blob/master/types.json

Align library code with changes made to the types, check `lib/`

### Increment api client version and publish to npm

Increment version in `package.json` and push to master branch on github. Run `npm publish` afterwards, you will need the npm credentials for this. Contact @dylanverstraete.

When the new version is pushed to npm you can continue.

### Update typescript types library

Repo: https://github.com/threefoldtech/tfchain_graphql_ts_types

Given a change in either

- [smartContractModule types](https://github.com/threefoldtech/tfchain_pallets/blob/development/pallet-smart-contract/src/types.rs)
- [tfgridModule types](https://github.com/threefoldtech/tfchain_pallets/blob/development/pallet-tfgrid/src/types.rs)

Open the respective directory:

`src/smartContractModule/definitions.ts` or `src/tfgridModule/definitions.ts`

Make the change in types accordingly to the change you made in the api client library.

#### Build changes

If you made the type change and you are testing on a local substrate node you will need to change: https://github.com/threefoldtech/tfchain_graphql_ts_types/blob/c5107f1e633f3abc581ca3b05d918b0d4e8a1a5d/package.json#L12

To contain the url that points to your local node.

Apply the change, and build the types

```
yarn
yarn build
```

Push the changes to github master branch, increment the package.json version and publish to npm with `npm publish`.



Note the version for both the api-client and the typescript library

Next up: [make the type change in graphql](./graphql_upgrade_types.md)