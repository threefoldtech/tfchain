const yargs = require('yargs')
const { exit } = require('yargs')
const { createEntity } = require('./src/contracts')

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
  // .command('get', 'Get a contract by ID', {
  //   contractID: {
  //     description: 'Contract ID',
  //     alias: 'id',
  //     type: 'string'
  //   }
  // })
  // .command('pay', 'Pay for a contract by ID', {
  //   contractID: {
  //     description: 'Contract ID',
  //     alias: 'id',
  //     type: 'string'
  //   },
  //   amount: {
  //     description: 'Amount to pay',
  //     alias: 'a',
  //     type: 'number'
  //   }
  // })
  // .command('accept', 'Accept a contract by ID', {
  //   contractID: {
  //     description: 'Contract ID',
  //     alias: 'id',
  //     type: 'string'
  //   },
  //   mnemonic: {
  //     description: 'Mnemonic to sign with',
  //     alias: 'm',
  //     type: 'string'
  //   }
  // })
  // .command('claim', 'Claim funds off a contract by ID', {
  //   contractID: {
  //     description: 'Contract ID',
  //     alias: 'id',
  //     type: 'string'
  //   },
  //   mnemonic: {
  //     description: 'Mnemonic to sign with',
  //     alias: 'm',
  //     type: 'string'
  //   }
  // })
  // .command('cancel', 'Cancel a contract by ID', {
  //   contractID: {
  //     description: 'Contract ID',
  //     alias: 'id',
  //     type: 'string'
  //   }
  // })
  .help()
  .alias('help', 'h')
  .argv

if (argv._.includes('create')) {
  if (!argv.n || !argv.t || !argv.s) {
    console.log('Bad Params')
    exit(1)
  }

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
// if (argv._.includes('get')) {
//   if (!argv.id) {
//     console.log('Bad Params')
//     exit(1)
//   }

//   getContract(argv.id)
//     .then(contract => {
//       console.log('\ncontract: ')
//       console.log(contract)
//       exit(0)
//     })
//     .catch(err => {
//       console.log(err)
//       exit(1)
//     })
// }
// if (argv._.includes('pay')) {
//   if (!argv.id || !argv.a) {
//     console.log('Bad Params')
//     exit(1)
//   }

//   payContract(argv.id, argv.a.toString(), ({ events = [], status }) => {
//     console.log(`Current status is ${status.type}`)

//     if (status.isFinalized) {
//       console.log(`Transaction included at blockHash ${status.asFinalized}`)

//       // Loop through Vec<EventRecord> to display all events
//       events.forEach(({ phase, event: { data, method, section } }) => {
//         console.log(`\t' ${phase}: ${section}.${method}:: ${data}`)
//       })
//       exit(1)
//     }
//   }).catch(err => {
//     console.log(err)
//     exit(1)
//   })
// }
// if (argv._.includes('accept')) {
//   if (argv.id === '' || !argv.m) {
//     console.log('Bad Params')
//     exit(1)
//   }

//   acceptContract(argv.id, argv.m, ({ events = [], status }) => {
//     console.log(`Current status is ${status.type}`)

//     if (status.isFinalized) {
//       console.log(`Transaction included at blockHash ${status.asFinalized}`)

//       // Loop through Vec<EventRecord> to display all events
//       events.forEach(({ phase, event: { data, method, section } }) => {
//         console.log(`\t' ${phase}: ${section}.${method}:: ${data}`)
//       })
//       exit(1)
//     }
//   }).catch(err => {
//     console.log(err)
//     exit(1)
//   })
// }
// if (argv._.includes('claim')) {
//   if (argv.id === '' || !argv.m) {
//     console.log('Bad Params')
//     exit(1)
//   }

//   claimContractFunds(argv.id, argv.m, ({ events = [], status }) => {
//     console.log(`Current status is ${status.type}`)

//     if (status.isFinalized) {
//       console.log(`Transaction included at blockHash ${status.asFinalized}`)

//       // Loop through Vec<EventRecord> to display all events
//       events.forEach(({ phase, event: { data, method, section } }) => {
//         console.log(`\t' ${phase}: ${section}.${method}:: ${data}`)
//       })
//       exit(1)
//     }
//   }).catch(err => {
//     console.log(err)
//     exit(1)
//   })
// }
// if (argv._.includes('cancel')) {
//   if (!argv.id) {
//     console.log('Bad Params')
//     exit(1)
//   }

//   cancelContract(argv.id, ({ events = [], status }) => {
//     console.log(`Current status is ${status.type}`)

//     if (status.isFinalized) {
//       console.log(`Transaction included at blockHash ${status.asFinalized}`)

//       // Loop through Vec<EventRecord> to display all events
//       events.forEach(({ phase, event: { data, method, section } }) => {
//         console.log(`\t' ${phase}: ${section}.${method}:: ${data}`)
//       })
//       exit(1)
//     }
//   }).catch(err => {
//     console.log(err)
//     exit(1)
//   })
// }
