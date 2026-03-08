#!/usr/bin/env node
/**
 * bridge_setup.js
 *
 * Configures TFChain for a local bridge dev environment:
 *   1. Create twin for the bridge validator (Alice)
 *   2. Register Alice as a bridge validator
 *   3. Set the bridge Stellar wallet address
 *   4. Set the fee account (Ferdie)
 *   5. Set deposit and withdraw fees
 *
 * Reads account details from BRIDGE_ENV_FILE (default: /tmp/bridge_local_env.sh)
 * or from individual env vars.
 *
 * Usage:
 *   node scripts/bridge_setup.js
 *   TFCHAIN_URL=ws://localhost:9944 node scripts/bridge_setup.js
 */

'use strict'

const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api')
const fs = require('fs')

const TFCHAIN_URL = process.env.TFCHAIN_URL || 'ws://localhost:9944'
const ENV_FILE = process.env.BRIDGE_ENV_FILE || '/tmp/bridge_local_env.sh'

// Fees: 10_000_000 base units = 1 TFT (7 decimal places)
const DEPOSIT_FEE = process.env.DEPOSIT_FEE || '10000000'
const WITHDRAW_FEE = process.env.WITHDRAW_FEE || '10000000'

function log (msg) { console.log(`[setup] ${msg}`) }
function die (msg) { console.error(`[setup] ERROR: ${msg}`); process.exit(1) }

function loadEnv () {
  if (!fs.existsSync(ENV_FILE)) {
    die(`Env file not found: ${ENV_FILE}. Run 'make accounts' first.`)
  }
  const lines = fs.readFileSync(ENV_FILE, 'utf8').split('\n')
  for (const line of lines) {
    const m = line.match(/^export\s+(\w+)="([^"]*)"/)
    if (m) process.env[m[1]] = m[2]
  }
}

function getEnv (key) {
  const val = process.env[key]
  if (!val) die(`Missing required env var: ${key}. Run 'make accounts' first.`)
  return val
}

async function signAndWait (api, tx, signer) {
  return new Promise((resolve, reject) => {
    tx.signAndSend(signer, ({ status, dispatchError, events }) => {
      if (dispatchError) {
        if (dispatchError.isModule) {
          const decoded = api.registry.findMetaError(dispatchError.asModule)
          reject(new Error(`${decoded.section}.${decoded.name}: ${decoded.docs}`))
        } else {
          reject(new Error(dispatchError.toString()))
        }
        return
      }
      if (status.isInBlock) resolve(status.asInBlock.toString())
    })
  })
}

async function main () {
  loadEnv()

  const bridgeAddress = getEnv('BRIDGE_ADDRESS')

  log(`Connecting to TFChain at ${TFCHAIN_URL}...`)
  const api = await ApiPromise.create({ provider: new WsProvider(TFCHAIN_URL) })
  const keyring = new Keyring({ type: 'sr25519' })

  const alice = keyring.addFromUri('//Alice')
  const ferdie = keyring.addFromUri('//Ferdie')

  log(`Alice address: ${alice.address}`)
  log(`Ferdie address: ${ferdie.address}`)
  log(`Bridge wallet: ${bridgeAddress}`)

  // 1. Create twin for Alice (validator identity on TFChain)
  log('Creating twin for Alice...')
  try {
    await signAndWait(api, api.tx.tfgridModule.createTwin('::1'), alice)
    log('Twin created.')
  } catch (e) {
    if (e.message && e.message.includes('TwinExists')) {
      log('Twin already exists, continuing.')
    } else {
      throw e
    }
  }

  // Use sudo/council to set bridge config — all bridge pallet calls use EnsureRootOrCouncilApproval
  // On --dev chain, Alice is sudo
  const sudo = (call) => api.tx.sudo.sudo(call)

  // 2. Register Alice as bridge validator
  log('Registering Alice as bridge validator...')
  try {
    await signAndWait(api, sudo(api.tx.tftBridgeModule.addBridgeValidator(alice.address)), alice)
    log('Validator registered.')
  } catch (e) {
    if (e.message && (e.message.includes('ValidatorExists') || e.message.includes('AlreadyValidator'))) {
      log('Validator already registered, continuing.')
    } else {
      throw e
    }
  }

  // 3. Set bridge Stellar wallet address
  log(`Setting bridge wallet to ${bridgeAddress}...`)
  await signAndWait(api, sudo(api.tx.tftBridgeModule.setFeeAccount(ferdie.address)), alice)
  await signAndWait(api, sudo(api.tx.tftBridgeModule.setBridgeAddress(bridgeAddress)), alice)
  log('Bridge wallet set.')

  // 4. Set fees
  log(`Setting deposit fee: ${DEPOSIT_FEE}, withdraw fee: ${WITHDRAW_FEE}...`)
  await signAndWait(api, sudo(api.tx.tftBridgeModule.setDepositFee(DEPOSIT_FEE)), alice)
  await signAndWait(api, sudo(api.tx.tftBridgeModule.setWithdrawFee(WITHDRAW_FEE)), alice)
  log('Fees set.')

  // Verify configuration
  const validators = await api.query.tftBridgeModule.validators()
  const feeAccount = await api.query.tftBridgeModule.feeAccount()
  const depositFee = await api.query.tftBridgeModule.depositFee()
  const withdrawFee = await api.query.tftBridgeModule.withdrawFee()

  log('=== TFChain Bridge Configuration ===')
  log(`  Validators:   ${JSON.stringify(validators.toHuman())}`)
  log(`  Fee account:  ${feeAccount.toHuman()}`)
  log(`  Deposit fee:  ${depositFee.toHuman()} (${Number(depositFee.toString()) / 1e7} TFT)`)
  log(`  Withdraw fee: ${withdrawFee.toHuman()} (${Number(withdrawFee.toString()) / 1e7} TFT)`)
  log('Setup complete.')

  await api.disconnect()
}

main().catch(e => {
  console.error(`[setup] FATAL: ${e.message || e}`)
  process.exit(1)
})
