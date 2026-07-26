const { Client } = require('@threefold/tfchain_client')

const { activationAmount } = require('./config')

const { MNEMONIC, URL } = process.env

const client = new Client({
  url: URL,
  mnemonicOrSecret: MNEMONIC,
  keypairType: 'sr25519'
})

/**
 * Refuse to start if the configured activation amount cannot create an account.
 * A transfer below the chain's existential deposit fails, so booting with such a
 * value would produce a service that errors on every activation — fail at startup
 * rather than once per request.
 */
function checkActivationAmount () {
  const amount = activationAmount()
  const existentialDeposit = client.api.consts.balances.existentialDeposit.toBigInt()

  if (BigInt(amount) < existentialDeposit) {
    throw new Error(
      `ACTIVATION_AMOUNT is ${amount} base units, below the chain's existential ` +
      `deposit (${existentialDeposit}); transfers of this size cannot create an account`
    )
  }
}

async function init () {
  // connect() validates the mnemonic, loads the keypair and, unlike the previous
  // client, rejects when the chain is unreachable instead of retrying forever.
  await client.connect()
  checkActivationAmount()
}

module.exports = {
  client,
  init
}
