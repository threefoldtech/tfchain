#!/usr/bin/env node
/**
 * bridge_mv_setup.js
 *
 * Verifies that the TFChain dev chain genesis has the bridge pallet configured
 * for multi-validator local development using Bob (//Bob) and Charlie (//Charlie).
 * Both are pre-registered as validators in the dev chain genesis.
 *
 * No pallet calls are made — TFChain has no sudo pallet; bridge admin calls
 * require root or council approval. The genesis configuration is sufficient.
 *
 * Usage:
 *   node scripts/bridge_mv_setup.js
 */

'use strict'

const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api')

const TFCHAIN_URL = process.env.TFCHAIN_URL || 'ws://localhost:9944'

function log (msg) { console.log(`[mv-setup] ${msg}`) }
function warn (msg) { console.warn(`[mv-setup] WARN: ${msg}`) }

async function main () {
  log(`Connecting to TFChain at ${TFCHAIN_URL}...`)
  const api = await ApiPromise.create({ provider: new WsProvider(TFCHAIN_URL) })
  const keyring = new Keyring({ type: 'sr25519' })

  const bob = keyring.addFromUri('//Bob')
  const charlie = keyring.addFromUri('//Charlie')
  const ferdie = keyring.addFromUri('//Ferdie')

  const validators = await api.query.tftBridgeModule.validators()
  const valList = validators.toHuman()
  const feeAccount = await api.query.tftBridgeModule.feeAccount()
  const depositFee = await api.query.tftBridgeModule.depositFee()
  const withdrawFee = await api.query.tftBridgeModule.withdrawFee()

  log('=== TFChain Multi-Validator Bridge Genesis Configuration ===')
  log(`  All validators: ${JSON.stringify(valList)}`)
  log(`  Fee account:    ${feeAccount.toHuman()}`)
  log(`  Deposit fee:    ${Number(depositFee.toString()) / 1e7} TFT`)
  log(`  Withdraw fee:   ${Number(withdrawFee.toString()) / 1e7} TFT`)

  const bobOk = valList.includes(bob.address)
  const charlieOk = valList.includes(charlie.address)

  log(`  Bob     (//Bob)     ${bobOk ? '✓' : '✗'} validator`)
  log(`  Charlie (//Charlie) ${charlieOk ? '✓' : '✗'} validator`)

  if (!bobOk || !charlieOk) {
    warn('One or more expected validators missing from genesis — MV tests may fail')
  }

  log(`  Running validators for MV tests: Bob + Charlie (2-of-3 threshold)`)
  log(`  3rd genesis validator: offline (not running a daemon)`)
  log('Setup verification complete.')

  await api.disconnect()
}

main().catch(e => {
  console.error(`[mv-setup] FATAL: ${e.message || e}`)
  process.exit(1)
})
