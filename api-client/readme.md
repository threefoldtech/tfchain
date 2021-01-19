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

Store an example entity:

```js
const name = "foobar"
const countryID = 1
const cityID = 1
```

```js
// This call wont block and will return the block where the tx is included
const block = await client.createEntity(name, countryID, cityID)
console.log(`Transaction included in block with hash: ${block.toHex()}`)
exit(1)
```

### Example with callback and listen for events

```js
// This call will block until status is Finalized and tx is included in a block and validated
await client.createEntity(name, countryID, cityID, (res) => {
  if (res instanceof Error) {
    console.log(res)
    exit(1)
  }

  const { events = [], status } = res
  console.log(`Current status is ${status.type}`)

  if (status.isFinalized) {
    console.log(`Transaction included at blockHash ${status.asFinalized}`)

    // Loop through Vec<EventRecord> to display all events
    events.forEach(({ phase, event: { data, method, section } }) => {
      console.log(`\t' ${phase}: ${section}.${method}:: ${data}`)
    })
    exit(1)
  }
})
```