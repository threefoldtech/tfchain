#!/usr/bin/env node
/**
 * bridge_tests.js
 *
 * E2E test suite for the TFChain bridge local dev environment.
 *
 * Tests (run sequentially):
 *   1. Normal withdraw   — swap 2 TFT on TFChain, receive 1 TFT on Stellar (1 TFT fee)
 *   2. Batch withdraws   — 5 simultaneous swaps in one block, all 5 delivered
 *   3. Bad deposit       — send TFT to bridge without memo, expect full refund
 *   4. Crash recovery    — SIGKILL bridge mid-withdraw, restart, verify delivery completes
 *
 * All tests assert exact TFT balances before and after. Non-zero exit on any failure.
 *
 * Usage:
 *   node scripts/bridge_tests.js
 *   TFCHAIN_URL=ws://localhost:9944 BRIDGE_PID_FILE=/tmp/bridge_local.pid node scripts/bridge_tests.js
 */

'use strict'

const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api')
const StellarSdk = require('@stellar/stellar-sdk')
const fs = require('fs')
const { execSync, spawn } = require('child_process')

const TFCHAIN_URL = process.env.TFCHAIN_URL || 'ws://localhost:9944'
const HORIZON_URL = process.env.STELLAR_HORIZON_URL || 'https://horizon-testnet.stellar.org'
const NETWORK_PASSPHRASE = StellarSdk.Networks.TESTNET
const ENV_FILE = process.env.BRIDGE_ENV_FILE || '/tmp/bridge_local_env.sh'
const BRIDGE_PID_FILE = process.env.BRIDGE_PID_FILE || '/tmp/bridge_local.pid'
const BRIDGE_LOG_FILE = process.env.BRIDGE_LOG_FILE || '/tmp/bridge_local.log'
const BRIDGE_BIN = process.env.BRIDGE_BIN || './bridge/tfchain_bridge/tfchain_bridge_local'
const BRIDGE_PERSISTENCY = process.env.BRIDGE_PERSISTENCY || './bridge/tfchain_bridge/signer_local.json'

// TFT has 7 decimal places: 1 TFT = 10_000_000 base units
const TFT = (amount) => amount * 10_000_000
const WITHDRAW_FEE_TFT = 1 // 1 TFT fee
const DEPOSIT_FEE_TFT = 1  // 1 TFT fee

let passed = 0
let failed = 0
let api, alice, horizon, userKp, bridgeAddress, issuerAddress

// ─── Helpers ─────────────────────────────────────────────────────────────────

function log (msg) { console.log(`  ${msg}`) }
function pass (name) { console.log(`✅ PASS: ${name}`); passed++ }
function fail (name, reason) { console.error(`❌ FAIL: ${name} — ${reason}`); failed++ }

function loadEnv () {
  if (!fs.existsSync(ENV_FILE)) {
    console.error(`[tests] Env file not found: ${ENV_FILE}. Run 'make accounts' first.`)
    process.exit(1)
  }
  const lines = fs.readFileSync(ENV_FILE, 'utf8').split('\n')
  for (const line of lines) {
    const m = line.match(/^export\s+(\w+)="([^"]*)"/)
    if (m) process.env[m[1]] = m[2]
  }
}

function getEnv (key) {
  const val = process.env[key]
  if (!val) { console.error(`Missing env var: ${key}`); process.exit(1) }
  return val
}

async function stellarTFTBalance (address) {
  const acc = await horizon.loadAccount(address)
  const tft = acc.balances.find(b => b.asset_code === 'TFT' && b.asset_issuer === issuerAddress)
  return tft ? parseFloat(tft.balance) : 0
}

async function tfchainTFTBalance (address) {
  const bal = await api.query.system.account(address)
  return bal.data.free.toNumber()
}

// Poll until condition() returns truthy or timeout
async function waitUntil (condition, { timeoutMs = 180_000, intervalMs = 3000, desc = '' } = {}) {
  const deadline = Date.now() + timeoutMs
  while (Date.now() < deadline) {
    const result = await condition()
    if (result) return result
    await new Promise(r => setTimeout(r, intervalMs))
  }
  throw new Error(`Timeout waiting for: ${desc}`)
}

async function swapToStellar (amount, nonce = -1) {
  const userAddress = getEnv('USER_ADDRESS')
  return new Promise((resolve, reject) => {
    api.tx.tftBridgeModule.swapToStellar(userAddress, TFT(amount))
      .signAndSend(alice, { nonce }, ({ status, dispatchError, events }) => {
        if (dispatchError?.isModule) {
          const d = api.registry.findMetaError(dispatchError.asModule)
          reject(new Error(`${d.section}.${d.name}`)); return
        }
        if (status.isInBlock) {
          let burnId = null
          events.forEach(({ event }) => {
            if (event.section === 'tftBridgeModule' && event.method === 'BurnTransactionCreated') {
              burnId = event.data[0].toNumber()
            }
          })
          resolve(burnId)
        }
      })
  })
}

async function bridgeIsRunning () {
  if (!fs.existsSync(BRIDGE_PID_FILE)) return false
  const pid = parseInt(fs.readFileSync(BRIDGE_PID_FILE, 'utf8').trim())
  try { process.kill(pid, 0); return true } catch { return false }
}

function getBridgePid () {
  if (!fs.existsSync(BRIDGE_PID_FILE)) return null
  return parseInt(fs.readFileSync(BRIDGE_PID_FILE, 'utf8').trim())
}

function killBridge (signal = 'SIGKILL') {
  const pid = getBridgePid()
  if (pid) {
    try { process.kill(pid, signal); log(`Bridge (PID ${pid}) killed with ${signal}`) } catch {}
  }
}

function startBridge () {
  const bridgeSecret = getEnv('BRIDGE_SECRET')
  // Genesis validator dev key 1 (from substrate-node/node/src/chain_spec.rs)
  const tfchainSeed = process.env.VAL1_TFCHAIN_SEED ||
    'quarter between satisfy three sphere six soda boss cute decade old trend'

  const child = spawn(BRIDGE_BIN, [
    '--secret', bridgeSecret,
    '--tfchainurl', TFCHAIN_URL,
    '--tfchainseed', tfchainSeed,
    '--bridgewallet', bridgeAddress,
    '--persistency', BRIDGE_PERSISTENCY,
    '--network', 'testnet'
  ], {
    detached: true,
    stdio: ['ignore', fs.openSync(BRIDGE_LOG_FILE, 'a'), fs.openSync(BRIDGE_LOG_FILE, 'a')]
  })
  child.unref()
  fs.writeFileSync(BRIDGE_PID_FILE, String(child.pid))
  log(`Bridge restarted (PID ${child.pid})`)
  return child.pid
}

async function waitForBridgeReady () {
  await waitUntil(async () => {
    if (!fs.existsSync(BRIDGE_LOG_FILE)) return false
    const tail = fs.readFileSync(BRIDGE_LOG_FILE, 'utf8').slice(-10000)
    return tail.includes('bridge_started')
  }, { timeoutMs: 30_000, desc: 'bridge_started log entry' })
}

// ─── Tests ────────────────────────────────────────────────────────────────────

async function test1_normalWithdraw () {
  console.log('\n── TEST 1: Normal withdraw (2 TFT swap → 1 TFT net on Stellar) ──')
  const name = 'test1_normalWithdraw'
  const userAddress = getEnv('USER_ADDRESS')

  try {
    const beforeStellar = await stellarTFTBalance(userAddress)
    log(`User Stellar TFT before: ${beforeStellar}`)

    const burnId = await swapToStellar(2)
    log(`Burn ID: ${burnId}`)

    const afterStellar = await waitUntil(async () => {
      const bal = await stellarTFTBalance(userAddress)
      if (bal > beforeStellar) return bal
    }, { timeoutMs: 180_000, desc: `Stellar balance to increase above ${beforeStellar}` })

    const delta = Math.round((afterStellar - beforeStellar) * 1e7) / 1e7
    const expected = 2 - WITHDRAW_FEE_TFT
    if (Math.abs(delta - expected) > 0.0000001) {
      fail(name, `Expected +${expected} TFT, got +${delta}`)
    } else {
      log(`User Stellar TFT after: ${afterStellar} (+${delta} TFT)`)
      pass(name)
    }
  } catch (e) {
    fail(name, e.message)
  }
}

async function test2_batchWithdraw () {
  console.log('\n── TEST 2: Batch withdraw (5 simultaneous swaps in one block) ──')
  const name = 'test2_batchWithdraw'
  const userAddress = getEnv('USER_ADDRESS')

  try {
    const beforeStellar = await stellarTFTBalance(userAddress)
    log(`User Stellar TFT before: ${beforeStellar}`)

    const nonce = await api.rpc.system.accountNextIndex(alice.address)
    const burnIds = await Promise.all(
      [0, 1, 2, 3, 4].map(i => swapToStellar(2, nonce.toNumber() + i))
    )
    log(`Burn IDs: ${burnIds.join(', ')}`)

    const expectedNet = 5 * (2 - WITHDRAW_FEE_TFT)
    const afterStellar = await waitUntil(async () => {
      const bal = await stellarTFTBalance(userAddress)
      if (bal >= beforeStellar + expectedNet - 0.0000001) return bal
    }, { timeoutMs: 300_000, desc: `Stellar balance ≥ ${beforeStellar + expectedNet}` })

    const delta = Math.round((afterStellar - beforeStellar) * 1e7) / 1e7
    log(`User Stellar TFT after: ${afterStellar} (+${delta} TFT, expected +${expectedNet})`)
    if (Math.abs(delta - expectedNet) < 0.0000001) {
      pass(name)
    } else {
      fail(name, `Expected +${expectedNet} TFT, got +${delta}`)
    }
  } catch (e) {
    fail(name, e.message)
  }
}

async function test3_badDeposit () {
  console.log('\n── TEST 3: Bad deposit (no memo → full refund) ──')
  const name = 'test3_badDeposit'
  const userAddress = getEnv('USER_ADDRESS')
  const userSecret = getEnv('USER_SECRET')
  const issuerSecret = getEnv('ISSUER_SECRET') // not needed for sending, just for asset

  try {
    const userKpStellar = StellarSdk.Keypair.fromSecret(userSecret)
    const TFTAsset = new StellarSdk.Asset('TFT', issuerAddress)
    const depositAmount = '3'

    const beforeStellar = await stellarTFTBalance(userAddress)
    log(`User Stellar TFT before: ${beforeStellar}`)

    // Send TFT to bridge without a memo
    const acc = await horizon.loadAccount(userAddress)
    const tx = new StellarSdk.TransactionBuilder(acc, {
      fee: '1000',
      networkPassphrase: NETWORK_PASSPHRASE
    })
      .addOperation(StellarSdk.Operation.payment({
        destination: bridgeAddress,
        asset: TFTAsset,
        amount: depositAmount
      }))
      .setTimeout(30)
      .build()
    tx.sign(userKpStellar)
    const result = await horizon.submitTransaction(tx)
    log(`Bad deposit sent: ${result.hash.slice(0, 16)}`)

    // Wait for refund — balance should return to (roughly) beforeStellar
    const afterStellar = await waitUntil(async () => {
      const bal = await stellarTFTBalance(userAddress)
      if (bal >= beforeStellar - 0.0000001) return bal
    }, { timeoutMs: 180_000, desc: 'refund to restore balance' })

    const delta = Math.round((afterStellar - beforeStellar) * 1e7) / 1e7
    log(`User Stellar TFT after: ${afterStellar} (delta: ${delta >= 0 ? '+' : ''}${delta})`)
    // Balance should be within 0 (full refund, no deposit fee on refunds)
    if (Math.abs(delta) < 0.0000001) {
      pass(name)
    } else {
      fail(name, `Expected net 0 change (full refund), got ${delta >= 0 ? '+' : ''}${delta}`)
    }
  } catch (e) {
    fail(name, e.message)
  }
}

async function test4_crashRecovery () {
  console.log('\n── TEST 4: Crash recovery (SIGKILL mid-withdraw, restart, verify delivery) ──')
  const name = 'test4_crashRecovery'
  const userAddress = getEnv('USER_ADDRESS')

  try {
    const beforeStellar = await stellarTFTBalance(userAddress)
    log(`User Stellar TFT before: ${beforeStellar}`)

    // Trigger a withdraw
    const burnId = await swapToStellar(2)
    log(`Burn ID: ${burnId}`)

    // Wait for BurnTransactionReady on TFChain (signatures collected), then kill bridge
    log('Waiting for BurnTransactionReady...')
    await waitUntil(async () => {
      const ready = await api.query.tftBridgeModule.burnTransactions(burnId)
      const json = ready.toJSON()
      return json && json.signatures && json.signatures.length >= 1
    }, { timeoutMs: 60_000, desc: 'BurnTransactionReady (≥1 sig)' })

    // Kill bridge mid-flight
    killBridge('SIGKILL')
    log('Bridge killed. Waiting 3s...')
    await new Promise(r => setTimeout(r, 3000))

    // Restart bridge
    startBridge()
    log('Bridge restarted. Waiting for it to come up...')
    await waitForBridgeReady()
    log('Bridge ready.')

    // Now wait for withdrawal to complete
    const afterStellar = await waitUntil(async () => {
      const bal = await stellarTFTBalance(userAddress)
      if (bal > beforeStellar) return bal
    }, { timeoutMs: 180_000, desc: 'Stellar balance to increase after crash recovery' })

    const delta = Math.round((afterStellar - beforeStellar) * 1e7) / 1e7
    const expected = 2 - WITHDRAW_FEE_TFT
    log(`User Stellar TFT after: ${afterStellar} (+${delta} TFT)`)
    if (Math.abs(delta - expected) < 0.0000001) {
      pass(name)
    } else {
      fail(name, `Expected +${expected} TFT after recovery, got +${delta}`)
    }
  } catch (e) {
    fail(name, e.message)
  }
}

// ─── Main ────────────────────────────────────────────────────────────────────

async function main () {
  loadEnv()

  bridgeAddress = getEnv('BRIDGE_ADDRESS')
  issuerAddress = getEnv('ISSUER_ADDRESS')

  console.log('[tests] Connecting to TFChain and Stellar...')
  api = await ApiPromise.create({ provider: new WsProvider(TFCHAIN_URL) })
  horizon = new StellarSdk.Horizon.Server(HORIZON_URL)

  const keyring = new Keyring({ type: 'sr25519' })
  alice = keyring.addFromUri('//Alice')

  console.log('[tests] Starting test suite...\n')

  await test1_normalWithdraw()
  await test2_batchWithdraw()
  await test3_badDeposit()
  await test4_crashRecovery()

  console.log(`\n${'─'.repeat(50)}`)
  console.log(`Results: ${passed} passed, ${failed} failed`)
  console.log('─'.repeat(50))

  await api.disconnect()
  process.exit(failed > 0 ? 1 : 0)
}

main().catch(e => {
  console.error(`[tests] FATAL: ${e.message || e}`)
  process.exit(1)
})
