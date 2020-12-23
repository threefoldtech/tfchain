const yargs = require('yargs')
const { exit } = require('yargs')
const { createEntity, getEntity } = require('./src/contracts')

const argv = yargs
  .command('create', 'Create a an entity', {
    name: {
      description: 'Name of the entity',
      alias: 'n',
      type: 'string'
    },
    country_id: {
      description: 'Id of the country',
      alias: 'c',
      type: 'number'
    },
    city_id: {
      description: 'Id of the city',
      alias: 't',
      type: 'number'
    }
  })
  .command('get', 'Get a entity by ID', {
    id: {
      description: 'entity ID',
      alias: 'id',
      type: 'string'
    }
  })
  .help()
  .alias('help', 'h')
  .argv

if (argv._.includes('create')) {
  // if (!argv.n || !argv.c || !argv.t) {
  //   console.log(argv)
  //   console.log('Bad Params')
  //   exit(1)
  // }

  createEntity(argv.n, argv.c, argv.t, ({ events = [], status }) => {
    console.log(`Current status is ${status.type}`)

    if (status.isFinalized) {
      console.log(`Transaction included at blockHash ${status.asFinalized}`)

      // Loop through Vec<EventRecord> to display all events
      events.forEach(({ phase, event: { data, method, section } }) => {
        console.log(`\t' ${phase}: ${section}.${method}:: ${data}`)
      })
      exit(1)
    }
  }).catch(err => {
    console.log(err)
    exit(1)
  })
}
if (argv._.includes('get')) {
  if (!argv.id) {
    console.log('Bad Params')
    exit(1)
  }

  getEntity(argv.id)
    .then(contract => {
      console.log('\n entity: ')
      console.log(contract)
      exit(0)
    })
    .catch(err => {
      console.log(err)
      exit(1)
    })
}
