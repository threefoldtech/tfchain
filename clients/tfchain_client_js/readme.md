# TF Grid API client

[![JavaScript Style Guide](https://img.shields.io/badge/code_style-standard-brightgreen.svg)](https://standardjs.com)
[![NPM version](https://img.shields.io/npm/v/tfgrid-api-client.svg?style=flat)](https://npmjs.org/package/tfgrid-api-client)

Client provides all the tooling around [tfchain](github.com/threefoldtech/tfchain) and its modules, including creating entities, twins, farms.. etc

## Installation

```sh
npm install tfgrid-api-client
```

## API Definition

Available methods:

### **init**

inits the client and returns a promise

```js
const Client = require("tfgrid-api-client");

const url = "urlToWebsocket";
const mnemonic = "some words";

const client = new Client(url, mnemonic);

try {
  await client.init();
} catch (err) {
  return err;
}
```

### **createEntity**

Creates an entity based on following information:

- name: name of the entity.
- countryID: ID of the country where the entity is located
- cityID: ID of the city where the entity is located
- callback: optional callback

```js
const name = "foobar";
const countryID = 1;
const cityID = 1;
```

```js
// This call wont be blocking and will return the block where the tx is included
const block = await client.createEntity(name, countryID, cityID, callback: optional)
console.log(`Transaction included in block with hash: ${block.toHex()}`)
```

Note: An entity is always linked to a private keypair, only one entity can be created per keypair.

### **updateEntity**

updates an entity based on following information:

- name: name of the entity.
- countryID: ID of the country where the entity is located
- cityID: ID of the city where the entity is located
- callback: optional callback

```js
// This call wont be blocking and will return the block where the tx is included
const block = await client.updateEntity(name, countryID, cityID, callback: optional)
console.log(`Transaction included in block with hash: ${block.toHex()}`)
```

### **getEntityByID**

Fetches an entity from storage based on an ID.

```js
const entity = await client.getEntityByID(1);
```

### **listEntities**

Fetches all entities from storage.

```js
const entity = await client.listEntities();
```

### **deleteEntity**

Deletes the entity linked to the private key.

```js
await client.deleteEntity(callback: optional)
```

### **createTwin**

Creates a twin based on following information:

- relay: a relay server address (optional)
- pk: the public key of the twins encryption key (optional)
- callback: optional callback

```js
const relay = "relay.dev.grid.tf" || null;
const pk = "somepublickey" || null;
```

```js
// This call wont be blocking and will return the block where the tx is included
const block = await client.createTwin(relay, pk, callback: optional)
console.log(`Transaction included in block with hash: ${block.toHex()}`)
```

Note: A twin is by default anonymous, check addTwinEntity to add an entity to a twin.

### **getTwinByID**

Fetches twin from storage based on an ID.

```js
const twin = await client.getTwinByID(1);
```

### **listTwins**

Fetches all twins from storage.

```js
const entity = await client.listTwins();
```

### **deleteTwin**

Deletes a twin from storage based on an ID. Only the creator of this twin can delete this twin.

```js
await client.deleteTwin(1);
```

### **addTwinEntity**

Add an entity to a twin. The entity that is being added must sign a message composed of the twinID and entityID. Only the twin's owner can add an entity to it's twin.

- entityID: entity ID to add.
- twinID: twin ID to update.
- signature: signature signed by private key of entity
- callback: optional callback

example:

```js
const entityID = 0;
const twinID = 0;

// the entity that owns this entity can sign this with his private key
const signedMessage = await client.sign(entityID, twinID);

// This call wont be blocking and will return the block where the tx is included
const block = await client.addTwinEntity(
  twinID,
  entityID,
  signedMessage,
  callback
);
console.log(`Transaction included in block with hash: ${block.toHex()}`);
```

If the signature of the `signedMessage` is valid, this entity id will be added to this twin.

### **removeTwinEntity**

Removes an entity from a twin. Only the twin's owner can remove an entity from it's twin.

- entityID: entity ID to remove.
- twinID: twin ID to update.
- callback: optional callback

example:

```js
// This call wont be blocking and will return the block where the tx is included
const block = await client.removeTwinEntity(twinID, entityID, callback);
console.log(`Transaction included in block with hash: ${block.toHex()}`);
```

### **sign**

Sign an entityID and twinID combination and returns a signed message.

- entityID: entity ID.
- twinID: twin ID.

```js
const signedMessage = await client.sign(entityID, twinID);
```

### **vest**

Vest an amount of tokens for a specific duration, if the tft price provided is equal to the real tft price. It unlocks the current and previous vesting months.

locked, perBlock, startingBlock, tftPrice

- locked: amount of tokens to lock
- perBlock: amount of tokens that unlock every block (1 block = 6 seconds)
- startingBlock: block number to start the vesting on
- tftPrice: price of tft that will trigger unlock condition (decimal number eg: 0.50)
- callback: optional callback

example:

```js
// This call wont be blocking and will return the block where the tx is included
const block = await client.vest(
  locked,
  perBlock,
  startingBlock,
  tftPrice,
  callback
);
console.log(`Transaction included in block with hash: ${block.toHex()}`);
```

### **getPrice**

Fetches the TFT Price.

```js
const price = await client.getPrice();
```

### **getBalance**

Fetches your account's balance.

```js
const balance = await client.getBalance();
```

### **tfStoreGet**

Set a value in tf key-value store

```js
await client.tfStoreSet("name", "Ashraf", (res) => {
  if (res instanceof Error) {
    console.log(res);
  }
});
```

### **tfStoreGet**

```js
console.log(await client.tfStoreGet("name"));
```

### Example callback function

```js
// This call will block until status is Finalized and tx is included in a block and validated
await client.createEntity(name, countryID, cityID, (res) => {
  if (res instanceof Error) {
    console.log(res);
    exit(1);
  }

  const { events = [], status } = res;
  console.log(`Current status is ${status.type}`);

  if (status.isFinalized) {
    console.log(`Transaction included at blockHash ${status.asFinalized}`);

    // Loop through Vec<EventRecord> to display all events
    events.forEach(({ phase, event: { data, method, section } }) => {
      console.log(`\t' ${phase}: ${section}.${method}:: ${data}`);
    });
    exit(1);
  }
});
```
