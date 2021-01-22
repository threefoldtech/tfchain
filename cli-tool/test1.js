const Client = require('tfgrid-api-client')
const { exit } = require('yargs')

async function main () {
  const cli = new Client('', 'industry dismiss casual gym gap music pave gasp sick owner dumb cost')

  try {
    await cli.init()
  } catch (err) {
    console.log(err)
    exit(1)
  }
}

main()
