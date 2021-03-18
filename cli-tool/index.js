const yargs = require('yargs')
const { exit } = require('yargs')
const {
  createEntity,
  updateEntity,
  getEntity,
  deleteEntity,
  getTwin,
  createTwin,
  deleteTwin,
  createFarm,
  getFarm,
  deleteFarm,
  addTwinEntity,
  removeTwinEntity,
  createNode,
  getNode,
  deleteNode,
  sign,
  getPrice,
  listEntities,
  listTwins,
  listFarms,
  listNodes,
  vestedTransfer,
  getBalance,
  signEntityCreation,
  getAvgPrice
} = require('./src/contracts')

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
    },
    signature: {
      description: 'Signature for entity creation',
      alias: 's',
      type: 'string'
    },
    target: {
      description: 'Target address to create an entity for',
      alias: 'f',
      type: 'string'
    },
    mnemonic: {
      description: 'Mnemonic to sign with',
      alias: 'm',
      type: 'string'
    },
    apiUrl: {
      description: 'Url of the api',
      alias: 'a',
      type: 'string'
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
    },
    mnemonic: {
      description: 'Mnemonic to sign with',
      alias: 'm',
      type: 'string'
    },
    apiUrl: {
      description: 'Url of the api',
      alias: 'a',
      type: 'string'
    }
  })
  .command('getEntity', 'Get a entity by ID', {
    id: {
      description: 'entity ID',
      alias: 'id',
      type: 'string'
    },
    apiUrl: {
      description: 'Url of the api',
      alias: 'a',
      type: 'string'
    }
  })
  .command('listEntities', 'Get all entities', {
    apiUrl: {
      description: 'Url of the api',
      alias: 'a',
      type: 'string'
    }
  })
  .command('createTwin', 'Create a twin', {
    ip: {
      description: 'ip',
      alias: 'ip',
      type: 'string'
    },
    mnemonic: {
      description: 'Mnemonic to sign with',
      alias: 'm',
      type: 'string'
    },
    apiUrl: {
      description: 'Url of the api',
      alias: 'a',
      type: 'string'
    }
  })
  .command('getTwin', 'Get a twin by ID', {
    id: {
      description: 'twin ID',
      alias: 'id',
      type: 'string'
    },
    apiUrl: {
      description: 'Url of the api',
      alias: 'a',
      type: 'string'
    }
  })
  .command('listTwins', 'Get all twins', {
    apiUrl: {
      description: 'Url of the api',
      alias: 'a',
      type: 'string'
    }
  })
  .command('addTwinEntity', 'Add an entity to a twin', {
    signature: {
      description: 'Signature of the entity id + the twin id',
      alias: 'sig',
      type: 'string'
    },
    twin_id: {
      description: 'Id of the twin',
      alias: 'twin',
      type: 'number'
    },
    entity_id: {
      description: 'Id of the entity',
      alias: 'entity',
      type: 'number'
    },
    mnemonic: {
      description: 'Mnemonic to sign with',
      alias: 'm',
      type: 'string'
    },
    apiUrl: {
      description: 'Url of the api',
      alias: 'a',
      type: 'string'
    }
  })
  .command('deleteTwinEntity', 'Delete twin entity by id', {
    twin_id: {
      description: 'Id of the twin',
      alias: 'twin',
      type: 'number'
    },
    id: {
      description: 'entity ID',
      alias: 'id',
      type: 'number'
    },
    mnemonic: {
      description: 'Mnemonic to sign with',
      alias: 'm',
      type: 'string'
    },
    apiUrl: {
      description: 'Url of the api',
      alias: 'a',
      type: 'string'
    }
  })
  .command('deleteTwin', 'Delete your twin', {
    twin_id: {
      description: 'Id of the twin',
      alias: 'twin',
      type: 'number'
    },
    mnemonic: {
      description: 'Mnemonic to sign with',
      alias: 'm',
      type: 'string'
    },
    apiUrl: {
      description: 'Url of the api',
      alias: 'a',
      type: 'string'
    }
  })
  .command('createFarm', 'Create a Farm', {
    name: {
      description: 'Name of the farm',
      alias: 'n',
      type: 'string'
    },
    entityID: {
      description: 'id the entity',
      alias: 'entityID',
      type: 'number'
    },
    twinID: {
      description: 'Id of twin',
      alias: 'twinID',
      type: 'number'
    },
    pricingPolicyID: {
      description: 'Id of pricing policy',
      alias: 'policyID',
      type: 'number'
    },
    certificationType: {
      description: 'Certification type (none, silver, gold)',
      alias: 'cert',
      type: 'string'
    },
    city_id: {
      description: 'Id of the city',
      alias: 'cityID',
      type: 'number'
    },
    country_id: {
      description: 'Id of the country',
      alias: 'countryID',
      type: 'number'
    },
    mnemonic: {
      description: 'Mnemonic to sign with',
      alias: 'm',
      type: 'string'
    },
    apiUrl: {
      description: 'Url of the api',
      alias: 'a',
      type: 'string'
    }
  })
  .command('listFarms', 'Get all farms', {
    apiUrl: {
      description: 'Url of the api',
      alias: 'a',
      type: 'string'
    }
  })
  .command('deleteFarm', 'Delete a farm by id', {
    id: {
      description: 'farm ID',
      alias: 'id',
      type: 'string'
    },
    mnemonic: {
      description: 'Mnemonic to sign with',
      alias: 'm',
      type: 'string'
    },
    apiUrl: {
      description: 'Url of the api',
      alias: 'a',
      type: 'string'
    }
  })
  .command('createNode', 'Create a node', {
    farmID: {
      description: 'farm ID',
      alias: 'farm',
      type: 'number'
    },
    twinID: {
      description: 'twin ID',
      alias: 'twin',
      type: 'number'
    },
    city_id: {
      description: 'Id of the city',
      alias: 'cityID',
      type: 'number'
    },
    country_id: {
      description: 'Id of the country',
      alias: 'countryID',
      type: 'number'
    },
    mnemonic: {
      description: 'Mnemonic to sign with',
      alias: 'm',
      type: 'string'
    },
    apiUrl: {
      description: 'Url of the api',
      alias: 'a',
      type: 'string'
    }
  })
  .command('getNode', 'Get a node by ID', {
    id: {
      description: 'node ID',
      alias: 'id',
      type: 'string'
    },
    apiUrl: {
      description: 'Url of the api',
      alias: 'a',
      type: 'string'
    }
  })
  .command('listNodes', 'Get all nodes', {
    apiUrl: {
      description: 'Url of the api',
      alias: 'a',
      type: 'string'
    }
  })
  .command('delete node', 'Delete a node by id', {
    id: {
      description: 'node ID',
      alias: 'id',
      type: 'string'
    },
    mnemonic: {
      description: 'Mnemonic to sign with',
      alias: 'm',
      type: 'string'
    },
    apiUrl: {
      description: 'Url of the api',
      alias: 'a',
      type: 'string'
    }
  })
  .command('sign', 'Sign an entity - twin id', {
    entityID: {
      description: 'entity ID',
      alias: 'entityid',
      type: 'number'
    },
    twinID: {
      description: 'twin ID',
      alias: 'twinid',
      type: 'number'
    },
    mnemonic: {
      description: 'Mnemonic to sign with',
      alias: 'm',
      type: 'string'
    },
    apiUrl: {
      description: 'Url of the api',
      alias: 'a',
      type: 'string'
    }
  })
  .command('getPrice', 'Get TFT price', {
    apiUrl: {
      description: 'Url of the api',
      alias: 'a',
      type: 'string'
    }
  })
  .command('getAvgPrice', 'Get Avg TFT price', {
    apiUrl: {
      description: 'Url of the api',
      alias: 'a',
      type: 'string'
    }
  })
  .command('vestedTransfer', 'vest an amount of tokens', {
    locked: {
      description: 'amount of tokens to lock',
      alias: 'l',
      type: 'number'
    },
    perBlock: {
      description: 'amount of tokens that unlock per block',
      alias: 'p',
      type: 'number'
    },
    startingBlock: {
      description: 'block to start the lock on',
      alias: 's',
      type: 'number'
    },
    tftPrice: {
      description: 'tft price that triggers unlock condition',
      alias: 't',
      type: 'number'
    },
    mnemonic: {
      description: 'Mnemonic to sign with',
      alias: 'm',
      type: 'string'
    },
    apiUrl: {
      description: 'Url of the api',
      alias: 'a',
      type: 'string'
    }
  })
  .command('getBalance', 'Get your accounts balance', {
    mnemonic: {
      description: 'Mnemonic to sign with',
      alias: 'm',
      type: 'string'
    },
    apiUrl: {
      description: 'Url of the api',
      alias: 'a',
      type: 'string'
    }
  })
  .command('SignEntityCreation', 'Sign an entity creation', {
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
    },
    mnemonic: {
      description: 'Mnemonic to sign with',
      alias: 'm',
      type: 'string'
    },
    apiUrl: {
      description: 'Url of the api',
      alias: 'a',
      type: 'string'
    }
  })
  .help()
  .alias('help', 'h')
  .argv

if (argv._.includes('createEntity')) {
  // if (!argv.n || !argv.c || !argv.t) {
  //   console.log(argv)
  //   console.log('Bad Params')
  //   exit(1)
  // }

  createEntity(argv.f, argv.n, argv.c, argv.t, argv.s, argv.m, argv.a, res => {
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

  updateEntity(argv.n, argv.c, argv.t, argv.m, argv.a, res => {
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

  getEntity(argv.id, argv.a)
    .then(entity => {
      console.log('\nentity: ')
      console.log(entity)
      exit(0)
    })
    .catch(err => {
      console.log(err)
      exit(1)
    })
}
if (argv._.includes('listEntities')) {
  listEntities(argv.a)
    .then(entities => {
      console.log('\nentities: ')
      console.log(entities)
      exit(0)
    })
    .catch(err => {
      console.log(err)
      exit(1)
    })
}
if (argv._.includes('deleteEntity')) {
  deleteEntity(argv.m, argv.a)
    .then(() => exit(0))
    .catch(err => {
      console.log(err)
      exit(1)
    })
}
if (argv._.includes('createTwin')) {
  createTwin(argv.ip, argv.m, argv.a, res => {
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
  }).catch(err => {
    console.log(err)
    exit(1)
  })
}
if (argv._.includes('addTwinEntity')) {
  addTwinEntity(argv.twin, argv.entity, argv.sig, argv.m, argv.a, res => {
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
  }).catch(err => {
    console.log(err)
    exit(1)
  })
}
if (argv._.includes('deleteTwinEntity')) {
  removeTwinEntity(argv.twin, argv.entity, argv.m, argv.a, res => {
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

  getTwin(argv.id, argv.a)
    .then(contract => {
      console.log('\n twin: ')
      console.log(contract)
      exit(0)
    })
    .catch(err => {
      console.log(err)
      exit(1)
    })
}
if (argv._.includes('listTwins')) {
  listTwins(argv.a)
    .then(twins => {
      console.log('\ntwins: ')
      console.log(twins)
      exit(0)
    })
    .catch(err => {
      console.log(err)
      exit(1)
    })
}
if (argv._.includes('deleteTwin')) {
  deleteTwin(argv.twin, argv.m, argv.a, res => {
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
  }).catch(err => {
    console.log(err)
    exit(1)
  })
}
if (argv._.includes('createFarm')) {
  // if (!argv.n || !argv.c || !argv.t) {
  //   console.log(argv)
  //   console.log('Bad Params')
  //   exit(1)
  // }

  createFarm(argv.name, argv.entityID, argv.twinID, argv.policyID, argv.cert, argv.cityID, argv.countryID, argv.m, argv.a, (res) => {
    if (res instanceof Error) {
      console.log(res)
      exit(1)
    }
    const { events = [], status } = res
    // console.log(res)
    console.log(`Current status is ${status.type}`)

    if (status.isFinalized) {
      console.log(`Transaction included at blockHash ${status.asFinalized}`)

      // Loop through Vec<EventRecord> to display all events
      events.forEach(({ phase, event: x }) => {
        const { data, method, section } = x
        console.log(`\t' ${phase}: ${section}.${method}:: ${data}`)
      })
      exit(1)
    }
  })
}
if (argv._.includes('getFarm')) {
  getFarm(argv.id, argv.a)
    .then(farm => {
      console.log('\n farm: ')
      console.log(farm)
      exit(0)
    })
    .catch(err => {
      console.log(err)
      exit(1)
    })
}
if (argv._.includes('listFarms')) {
  listFarms(argv.a)
    .then(farms => {
      console.log('\nfarms: ')
      console.log(farms)
      exit(0)
    })
    .catch(err => {
      console.log(err)
      exit(1)
    })
}
if (argv._.includes('deleteFarm')) {
  if (!argv.id) {
    console.log('Bad Params')
    exit(1)
  }

  deleteFarm(argv.id, argv.m, argv.a, res => {
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
  }).catch(err => {
    console.log(err)
    exit(1)
  })
}
if (argv._.includes('createNode')) {
  createNode(argv.farmID, argv.twinID, argv.cityID, argv.countryID, argv.m, argv.a, (res) => {
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
  }).catch(err => {
    console.log(err)
    exit(1)
  })
}
if (argv._.includes('getNode')) {
  getNode(argv.id, argv.a)
    .then(node => {
      console.log('\n node: ')
      console.log(node)
      exit(0)
    })
    .catch(err => {
      console.log(err)
      exit(1)
    })
}
if (argv._.includes('listNodes')) {
  listNodes(argv.a)
    .then(nodes => {
      console.log('\nnodes: ')
      console.log(nodes)
      exit(0)
    })
    .catch(err => {
      console.log(err)
      exit(1)
    })
}
if (argv._.includes('deleteNode')) {
  if (!argv.id) {
    console.log('Bad Params')
    exit(1)
  }

  deleteNode(argv.id, argv.m, argv.a, res => {
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
  }).catch(err => {
    console.log(err)
    exit(1)
  })
}
if (argv._.includes('sign')) {
  sign(argv.entityID, argv.twinID, argv.m, argv.a)
    .then(message => {
      console.log('\n message: ')
      console.log(message)
      exit(0)
    })
    .catch(err => {
      console.log(err)
      exit(1)
    })
}
if (argv._.includes('getPrice')) {
  getPrice(argv.a)
    .then(price => {
      console.log(`avg price: ${price}`)
      exit(0)
    })
    .catch(err => {
      console.log(err)
      exit(1)
    })
}
if (argv._.includes('getAvgPrice')) {
  getAvgPrice(argv.a)
    .then(price => {
      console.log(`avg price: ${price}`)
      exit(0)
    })
    .catch(err => {
      console.log(err)
      exit(1)
    })
}
if (argv._.includes('vestedTransfer')) {
  vestedTransfer(argv.l, argv.p, argv.s, argv.t, argv.m, argv.a, (res) => {
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
  }).catch(err => {
    console.log(err)
    exit(1)
  })
}
if (argv._.includes('getBalance')) {
  getBalance(argv.m, argv.a)
    .then(price => {
      console.log('\nbalance: ')
      console.log(price)
      exit(0)
    })
    .catch(err => {
      console.log(err)
      exit(1)
    })
}
if (argv._.includes('signEntityCreation')) {
  signEntityCreation(argv.n, argv.c, argv.t, argv.a, argv.m)
    .then(message => {
      console.log('\nsignature: ')
      console.log(message)
      exit(0)
    })
    .catch(err => {
      console.log(err)
      exit(1)
    })
}
