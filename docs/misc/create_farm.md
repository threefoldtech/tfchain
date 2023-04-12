# Creating a Farm on TFChain Devnet

## Prerequisites:

- you have to have an account (ID) on the polkadot TFChain.  We create a user account in the process below.  If you want to do that first, here's how you get [one](create_account.md)
- you have to have yggdrasil networking installed on your desktop/lapop.  Here's how you do [that](https://yggdrasil-network.github.io/installation.html)

With these two steps done - we're good to go and get a devnet 3.0 farm_id

## Step 1: Copy types to clipboard

Open https://raw.githubusercontent.com/threefoldtech/tfgrid-api-client/master/types.json and copy the whole content to your clipboard.

## Step 2: browse to Polkadot UI

Open the following URL based on your setup:
- Using a private node: https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/settings/developer
- Using a public node: https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Ftfchain.dev.threefold.io#/settings/developer

Paste the types in the box and hit `save`

![img](./assets/copy_types_1.png)

## Step 3: Create an account

Open the following URL based on your setup:
- Using a private node:https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/accounts
- Using a public node: https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Ftfchain.dev.threefold.io#/accounts

Click `Add account`, take note of the seed. Click on the `Advanced creation options` arrow, and select keypair crypto type: `Edwards(ed25519, alternative)`

Click `I have saved my mnemonic seed safely` and click next and fill in the required fields.

![account_creation](./assets/account_create_1.png)

## Step 4: Fund your account

On the same page, on the left top, hover over `Account` button and click on `Transfer`. First select account `Alice` and secondly select your newly created account from the list. Send any amount to your account (these are just tokens to play around with, they hold no real value).

![account_transfer](./assets/account_transfer_1.png)

## Step 5: Create a Twin

Open the following URL based on your setup:
- Using a private node: https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/extrinsics
- Using a public node: https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Ftfchain.dev.threefold.io#/extrinsics

and select your account from the list. First, sign some dummy terms and conditions by selecting `tfgridModule` -> `userAcceptTc(documentLink, documentHash)` and providing some dummy link and hash. Next, select `tfgridModule` -> `createTwin(ip)` from the list.

Fill in your [Yggdrasil](https://github.com/yggdrasil-network/yggdrasil-go) IPV6. And click on submit transaction and sign it with your account.

![create_twin](./assets/create_twin_1.png)

## Step 6: Create a Farm

Open the following URL based on your setup:
- Using a private node: https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/extrinsics
- Using a public node: https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Ftfchain.dev.threefold.io#/extrinsics

and select your account from the list. Next, select `tfgridModule` -> `createFarm(..)` from the list.

Fill in a name, select a certification type and leave `country_id` and `city_id` to 0. You can, if you want, set the country/city id values to a value from the https://explorer.devnet.grid.tf/graphql/ explorer. 

To find a country or city you can query all the available countries and cities in graphql.

Optionally you can also provide public ips on your farm.

![create_farm](./assets/create_farm_1.png)

## Step 7: query twin ID and Farm ID

### Query twin

Open the following URL based on your setup:
- Using a private node: https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/chainstate 
- Using a public node: https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Ftfchain.dev.threefold.io#/chainstate

and select `tfgridModule` -> scroll all the way down to `twinIdByAccountID(accountID):u32` and select your account ID from the list. Hit the PLUS symbol and you should see your twin ID.

![query_twin](./assets/query_twin_1.png)

### Query farm:

Open https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/chainstate 

and select `tfgridModule` -> scroll to `farmIdByName(bytes):u32` and search your farm ID based on your farm name. Hit the PLUS symbol and you should see your farm ID.

![query_farm](./assets/query_farm_1.png)
