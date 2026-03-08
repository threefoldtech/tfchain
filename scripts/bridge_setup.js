#!/usr/bin/env node
/**
 * bridge_setup.js
 *
 * Verifies that the TFChain dev chain genesis has the bridge pallet configured
 * correctly for local development. The dev chain genesis pre-configures:
 *   - Bridge validators: Bob (//Bob) and Charlie (//Charlie)
 *   - Fee account: Ferdie (//Ferdie)
 *   - Deposit fee: 10,000,000 base units (1 TFT)
 *   - Withdraw fee: 10,000,000 base units (1 TFT)
 *
 * No pallet calls are made — there is no sudo pallet on TFChain and all bridge
 * admin calls require root or council approval. The genesis configuration is
 * sufficient for single-validator local development.
 *
 * Usage:
 *   node scripts/bridge_setup.js
 */

'use strict'

const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api')

const TFCHAIN_URL = process.env.TFCHAIN_URL || 'ws://localhost:9944'

function log (msg) { console.log(`[setup] ${msg}`) }
function warn (msg) { console.warn(`[setup] WARN: ${msg}`) }
function die (msg) { console.error(`[setup] ERROR: ${msg}`); process.exit(1) }

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

  log('=== TFChain Bridge Genesis Configuration ===')
  log(`  Validators:   ${JSON.stringify(valList)}`)
  log(`  Fee account:  ${feeAccount.toHuman()}`)
  log(`  Deposit fee:  ${Number(depositFee.toString()) / 1e7} TFT`)
  log(`  Withdraw fee: ${Number(withdrawFee.toString()) / 1e7} TFT`)

  // Verify expected validators are present
  if (!valList.includes(bob.address)) {
    warn(`Bob (${bob.address}) is not a genesis validator — bridge daemon using //Bob will be rejected`)
  } else {
    log(`  Bob (//Bob) ✓ is a registered validator`)
  }

  if (!valList.includes(charlie.address)) {
    warn(`Charlie (${charlie.address}) is not a genesis validator`)
  } else {
    log(`  Charlie (//Charlie) ✓ is a registered validator`)
  }

  if (feeAccount.toHuman() !== ferdie.address) {
    warn(`Fee account is ${feeAccount.toHuman()}, expected Ferdie (${ferdie.address})`)
  } else {
    log(`  Fee account ✓ is Ferdie`)
  }

  if (Number(depositFee.toString()) === 0) {
    warn('Deposit fee is 0 — bridge may not charge fees')
  }

  log('Setup verification complete.')
  await api.disconnect()
}

main().catch(e => {
  console.error(`[setup] FATAL: ${e.message || e}`)
  process.exit(1)
})
