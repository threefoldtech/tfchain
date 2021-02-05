# TF Grid db on substrate

## Installation

### Node

You will need a specific version of rust nightly in order to compile:

`rustup install nightly-2020-10-05`

Wasm toolchain:

`rustup target add wasm32-unknown-unknown --toolchain nightly-2020-10-05`

Now you can build.

```
cd substrate-node
make build-debug
```

This will build the node binary in release mode, once built you can execute it by doing following:

`./target/release/node-template --dev --tmp --ws-external`

> You need the `ws-external` flag in order to connect from a zos node to substrate in a local setup.


### Client

Now you can build the client to interact with this node:

You need Yarn in order to continue.

```
cd cli-tool
yarn install
```

> The client uses the keys of **Bob** the sign for transactions in all the examples. The user **Bob** is a dummy user created when the chain starts.

So whenever objects (entities, twins, ..) are created, these will be linked to **Bob**.


### Graphql (optional)

If you want to query the data inside the blockchain with graphql you can set this up locally.

```
cd graphql

yarn

yarn build
yarn db:up
yarn db:prepare
yarn db:migrate

docker-compose up
```

Now browse to localhost:4000/graphql

Example query: 

```
query {
    entities(limit: 5) {
        name
    }
}
```

### Client examples

Use the client inside `./cli-tool` for following examples

### Creating an entity

Parameters:

* **-n**: Name of the entity.
* **-c**: Country ID (id of the country in the db (TODO))
* **-t**: City ID (id of the city in the db(TODO)) 
* **-m**: Mnemonic to sign with, if you provide a mnemonic of your own, your substrate public address must be funded first.
* **-a**: API Url (if none is given, it goes to localhost), example: wss://tfgrid.tri-fold.com

`node index.js createEntity -n newEntity -c 0 -t 0`

### Fetching the entity's details.

entity ID's are incremented sequentially. If you create your first entity the ID will be 0, the second will be 1, etc...

`node index.js getEntity --id 0`

### Deleting the entity.

Delete your entity

`node index.js deleteEntity`

### Creating a twin

Twins are linked to your public key. You don't need to provide parameters.

`node index.js createTwin`

### Fetching a twin's details

`node index.js getTwin --id 0`

### Deleting a twin

Delete your twin.

`node index.js deleteTwin`

### Making a vesting call (BETA)

You can vest X amount of tokens for X amount of time:

Example:

- 12000 tokens
- 0.00228 tokens unlock every block (that is 1000 ish tokens each month, which means te vesting period is 1 year)
- 0.30 cent is the TFT unlock price

`node index.js vestedTransfer -l 12000 -p 0.00228 -s 8 -t 0.30`

Check your vested balance:

`node index.js getBalance`

You are now vesting! :)