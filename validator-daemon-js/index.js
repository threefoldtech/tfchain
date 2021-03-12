const yargs = require('yargs')
const { monitorBridge } = require('./src/bridge')
const { monitorVesting } = require('./src/vesting')

const argv = yargs
  .command('bridge', 'Start your bridge validator node', {
    mnemonic: {
      description: 'Mnemonic to sign with',
      alias: 'm',
      type: 'string'
    },
    apiUrl: {
      description: 'Url of the api',
      alias: 'a',
      type: 'string'
    },
    account: {
      description: 'Account to monitor',
      alias: 'c',
      type: 'string',
      default: 'GDIVGRGFOHOEWJKRR5KKW2AJALUYARHO7MW3SPI4N33IJZ4PQ5WJ6TU2'
    }
  })
  .command('vesting', 'Start your vesting validator node', {
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

if (argv._.includes('bridge')) {
  monitorBridge(argv.m, argv.a)
} else if (argv._.includes('vesting')) {
  monitorVesting(argv.m, argv.a)
}
