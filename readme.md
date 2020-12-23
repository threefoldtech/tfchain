# TF Grid db on substrate

## Installation

```
cd substrate-node
make build
```

This will build the node binary in release mode, once built you can execute it by doing following:

`./target/release/node-template --dev --tmp --ws-external`

> You need the `ws-external` flag in order to connect from a zos node to substrate in a local setup.

Now you can build the client to interact with this node:

You need Yarn in order to continue.

```
cd client
yarn install
```

> The client uses the keys of **Bob** the sign for transactions in all the examples. The user **Bob** is a dummy user created when the chain starts.

## Creating an entity

Parameters:

* **-n**: Name of the entity.
* **-c**: Country ID (id of the country in the db (TODO))
* **-t**: City ID (id of the city in the db(TODO)) 

`node index.js create -n newEntity -c 0 -t 0`

## Fetching the entity's details.

entity ID's are incremented sequentially. If you create your first entity the ID will be 0, the second will be 1, etc...

`node index.js get --id 0`