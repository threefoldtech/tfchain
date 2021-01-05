const yargs = require('yargs')
const { exit } = require('yargs')
const { createEntity, updateEntity, getEntity, deleteEntity, getTwin, createTwin, deleteTwin } = require('./src/contracts')

const argv = yargs
  .command('createEntity', 'Create a an entity', {
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
  .command('updateEntity', 'Update a an entity', {
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
  .command('getEntity', 'Get a entity by ID', {
    id: {
      description: 'entity ID',
      alias: 'id',
      type: 'string'
    }
  })
  .command('createTwin', 'Create a twin')
  .command('getTwin', 'Get a twin by ID', {
    id: {
      description: 'twin ID',
      alias: 'id',
      type: 'string'
    }
  })
  .command('deleteTwin', 'Delete your twin')
  .help()
  .alias('help', 'h')
  .argv

if (argv._.includes('createEntity')) {
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
if (argv._.includes('updateEntity')) {
  // if (!argv.n || !argv.c || !argv.t) {
  //   console.log(argv)
  //   console.log('Bad Params')
  //   exit(1)
  // }

  updateEntity(argv.n, argv.c, argv.t, ({ events = [], status }) => {
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
if (argv._.includes('getEntity')) {
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
if (argv._.includes('deleteEntity')) {
  deleteEntity(({ events = [], status }) => {
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
if (argv._.includes('createTwin')) {
  createTwin(({ events = [], status }) => {
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
if (argv._.includes('getTwin')) {
  if (!argv.id) {
    console.log('Bad Params')
    exit(1)
  }

  getTwin(argv.id)
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
if (argv._.includes('deleteTwin')) {
  deleteTwin(({ events = [], status }) => {
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
