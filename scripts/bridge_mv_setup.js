#!/usr/bin/env node
/**
 * bridge_mv_setup.js
 *
 * Configures TFChain for a multi-validator bridge dev environment:
 *   1. Create twins for all 3 validators (Alice, Bob, Charlie)
 *   2. Register all 3 as bridge validators
 *   3. Set the bridge Stellar wallet address
 *   4. Set the fee account (Ferdie)
 *   5. Set deposit and withdraw fees
 *
 * Usage:
 *   node scripts/bridge_mv_setup.js
 *   TFCHAIN_URL=ws://localhost:9944 BRIDGE_MV_ENV_FILE=/tmp/bridge_mv_env.sh node scripts/bridge_mv_setup.js
 */

'use strict'

const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api')
const fs = require('fs')

const TFCHAIN_URL = process.env.TFCHAIN_URL || 'ws://localhost:9944'
const ENV_FILE = process.env.BRIDGE_MV_ENV_FILE || '/tmp/bridge_mv_env.sh'
const DEPOSIT_FEE = process.env.DEPOSIT_FEE || '10000000'
const WITHDRAW_FEE = process.env.WITHDRAW_FEE || '10000000'

function log (msg) { console.log(`[mv-setup] ${msg}`) }
function die (msg) { console.error(`[mv-setup] ERROR: ${msg}`); process.exit(1) }

function loadEnv () {
  if (!fs.existsSync(ENV_FILE)) die(`Env file not found: ${ENV_FILE}. Run 'make bridge-mv-accounts' first.`)
  const lines = fs.readFileSync(ENV_FILE, 'utf8').split('\n')
  for (const line of lines) {
    const m = line.match(/^export\s+(\w+)="([^"]*)"/)
    if (m) process.env[m[1]] = m[2]
  }
}

function getEnv (key) {
  const val = process.env[key]
  if (!val) die(`Missing env var: ${key}`)
  return val
}

async function signAndWait (api, tx, signer) {
  return new Promise((resolve, reject) => {
    tx.signAndSend(signer, ({ status, dispatchError }) => {
      if (dispatchError) {
        if (dispatchError.isModule) {
          const d = api.registry.findMetaError(dispatchError.asModule)
          reject(new Error(`${d.section}.${d.name}`))
        } else {
          reject(new Error(dispatchError.toString()))
        }
        return
      }
      if (status.isInBlock) resolve(status.asInBlock.toString())
    })
  })
}

async function createTwinIfNeeded (api, signer, label) {
  try {
    await signAndWait(api, api.tx.tfgridModule.createTwin('::1'), signer)
    log(`Twin created for ${label}.`)
  } catch (e) {
    if (e.message && e.message.includes('TwinExists')) {
      log(`Twin already exists for ${label}, skipping.`)
    } else {
      throw e
    }
  }
}

async function addValidatorIfNeeded (api, sudoSigner, validatorAddress, label) {
  try {
    await signAndWait(
      api,
      api.tx.sudo.sudo(api.tx.tftBridgeModule.addBridgeValidator(validatorAddress)),
      sudoSigner
    )
    log(`${label} registered as bridge validator.`)
  } catch (e) {
    if (e.message && (e.message.includes('ValidatorExists') || e.message.includes('AlreadyValidator'))) {
      log(`${label} already registered, skipping.`)
    } else {
      throw e
    }
  }
}

async function main () {
  loadEnv()

  const bridgeAddress = getEnv('BRIDGE_ADDRESS')

  log(`Connecting to TFChain at ${TFCHAIN_URL}...`)
  const api = await ApiPromise.create({ provider: new WsProvider(TFCHAIN_URL) })
  const keyring = new Keyring({ type: 'sr25519' })

  const alice = keyring.addFromUri('//Alice')    // Val1 + sudo
  const bob = keyring.addFromUri('//Bob')        // Val2
  const charlie = keyring.addFromUri('//Charlie') // Val3
  const ferdie = keyring.addFromUri('//Ferdie')  // Fee account

  log(`Alice   (Val1): ${alice.address}`)
  log(`Bob     (Val2): ${bob.address}`)
  log(`Charlie (Val3): ${charlie.address}`)
  log(`Ferdie  (fees): ${ferdie.address}`)
  log(`Bridge wallet:  ${bridgeAddress}`)

  // 1. Create twins for all validators (needed for TFChain identity)
  log('Creating twins...')
  await createTwinIfNeeded(api, alice, 'Alice')
  await createTwinIfNeeded(api, bob, 'Bob')
  await createTwinIfNeeded(api, charlie, 'Charlie')

  // 2. Register all 3 as bridge validators (requires sudo/root)
  log('Registering validators...')
  await addValidatorIfNeeded(api, alice, alice.address, 'Alice')
  await addValidatorIfNeeded(api, alice, bob.address, 'Bob')
  await addValidatorIfNeeded(api, alice, charlie.address, 'Charlie')

  // 3. Set bridge wallet address and fee account
  log('Setting bridge wallet and fee account...')
  await signAndWait(api, api.tx.sudo.sudo(api.tx.tftBridgeModule.setFeeAccount(ferdie.address)), alice)
  await signAndWait(api, api.tx.sudo.sudo(api.tx.tftBridgeModule.setBridgeAddress(bridgeAddress)), alice)

  // 4. Set fees
  log('Setting fees...')
  await signAndWait(api, api.tx.sudo.sudo(api.tx.tftBridgeModule.setDepositFee(DEPOSIT_FEE)), alice)
  await signAndWait(api, api.tx.sudo.sudo(api.tx.tftBridgeModule.setWithdrawFee(WITHDRAW_FEE)), alice)

  // Verify
  const validators = await api.query.tftBridgeModule.validators()
  const feeAccount = await api.query.tftBridgeModule.feeAccount()
  const depositFee = await api.query.tftBridgeModule.depositFee()
  const withdrawFee = await api.query.tftBridgeModule.withdrawFee()

  log('=== TFChain Multi-Validator Bridge Configuration ===')
  log(`  Validators:   ${JSON.stringify(validators.toHuman())}`)
  log(`  Fee account:  ${feeAccount.toHuman()}`)
  log(`  Deposit fee:  ${Number(depositFee.toString()) / 1e7} TFT`)
  log(`  Withdraw fee: ${Number(withdrawFee.toString()) / 1e7} TFT`)
  log('Setup complete.')

  await api.disconnect()
}

main().catch(e => {
  console.error(`[mv-setup] FATAL: ${e.message || e}`)
  process.exit(1)
})
