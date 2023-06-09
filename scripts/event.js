// Import the API
const { ApiPromise, WsProvider } = require('@polkadot/api')

async function main() {
  const provider = new WsProvider('wss://tfchain.dev.grid.tf')
  // Create our API with a default connection to the local node
  const api = await ApiPromise.create({ provider })

  // Subscribe to system events via storage
  api.query.system.events((events) => {
    console.log(`\nReceived ${events.length} events:`)

    // Loop through the Vec<EventRecord>
    events.forEach((record) => {
      // Extract the phase, event and the event types
      const { event, phase } = record
      const types = event.typeDef

      // Show what we are busy with
      console.log(`\t${event.section}:${event.method}:: (phase=${phase.toString()})`)
      console.log(event)
      console.log(`\t\t${event.meta.documentation}`)

      // Loop through each of the parameters, displaying the type and data
      event.data.forEach((data, index) => {
        console.log(`\t\t\t${types[index].type}: ${data.toString()}`)
      })
    })
  })
}

main().catch((error) => {
  console.error(error)
  process.exit(-1)
})