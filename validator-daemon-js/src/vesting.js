const { getClient } = require('./subclient')
const chalk = require('chalk')

async function monitorVesting (mnemonic, url) {
  const client = await getClient(url, mnemonic)
  console.log(chalk.green.bold('✓ starting vesting validator daemon...'))

  client.api.query.system.events((events) => {
    events.forEach((record) => {
      const { event, phase } = record
      const types = event.typeDef

      if (event.section === 'tfgridModule') {
        console.log(event.method)

        console.log(
          `${event.section}:${event.method}:: (phase=${phase.toString()})`
        )

        event.data.forEach((data, index) => {
          console.log(`${types[index].type}: ${data.toString()}`)
        })
      }
    })
  })
}

module.exports = {
  monitorVesting
}
