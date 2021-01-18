# TF Grid db client

## Installation

TODO

## Initialization of the client

Parameters:

- url: url of the blockchain node.
- mneomic: a bip39 mnemonic. 

```js
const cli = new Client(url, mnemonic)

try {
  await cli.init()
} catch (err) {
  return err
}
```

### Example

Store an entity:

```js
const res = await client.createEntity(name, countryID, cityID, callback)
```

### Example with callback and listen for events

```js
const callback = ({ events = [], status }) => {
    console.log(`Current status is ${status.type}`)

    if (status.isFinalized) {
      console.log(`Transaction included at blockHash ${status.asFinalized}`)

      // Loop through Vec<EventRecord> to display all events
      events.forEach(({ phase, event: { data, method, section } }) => {
        console.log(`\t' ${phase}: ${section}.${method}:: ${data}`)
      })
      exit(1)
    }
}

const res = await client.createEntity(name, countryID, cityID, callback)
```