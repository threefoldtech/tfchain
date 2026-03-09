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
const { spawn } = require('child_process')
const {
  log, pass, fail,
  loadEnv, getEnv,
  stellarTFTBalance,
  waitUntil,
  swapToStellar,
  TFT_DECIMALS
} = require('./bridge_helpers')

const TFCHAIN_URL = process.env.TFCHAIN_URL || 'ws://localhost:9944'
const HORIZON_URL = process.env.STELLAR_HORIZON_URL || 'https://horizon-testnet.stellar.org'
const NETWORK_PASSPHRASE = StellarSdk.Networks.TESTNET
const ENV_FILE = process.env.BRIDGE_ENV_FILE || '/tmp/bridge_local_env.sh'
const BRIDGE_PID_FILE = process.env.BRIDGE_PID_FILE || '/tmp/bridge_local.pid'
const BRIDGE_LOG_FILE = process.env.BRIDGE_LOG_FILE || '/tmp/bridge_local.log'
const BRIDGE_BIN = process.env.BRIDGE_BIN || './bridge/tfchain_bridge/tfchain_bridge_local'
const BRIDGE_PERSISTENCY = process.env.BRIDGE_PERSISTENCY || './bridge/tfchain_bridge/signer_local.json'

const WITHDRAW_FEE_TFT = 1 // 1 TFT fee

const counter = { passed: 0, failed: 0 }
let api, alice, horizon, bridgeAddress, issuerAddress

// ─── Bridge lifecycle helpers ───────────────────────────────────────────────

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
  const tfchainSeed = process.env.VAL1_TFCHAIN_SEED ||
    'quarter between satisfy three sphere six soda boss cute decade old trend'

  // Use shell exec + append redirect instead of fd inheritance.
  // On macOS, passing a numeric fd to a detached child's stdio is unreliable:
  // the fd silently becomes invalid after child.unref(), so the bridge writes nothing
  // to the log. Shell exec replaces sh with the bridge binary (same PID), and
  // >> redirect is handled by the shell before exec, so it works cross-platform.
  const shellCmd = [
    'exec',
    `"${BRIDGE_BIN}"`,
    '--secret', `"${bridgeSecret}"`,
    '--tfchainurl', TFCHAIN_URL,
    '--tfchainseed', `"${tfchainSeed}"`,
    '--bridgewallet', bridgeAddress,
    '--persistency', BRIDGE_PERSISTENCY,
    '--network', 'local',
    `>>"${BRIDGE_LOG_FILE}"`, '2>&1'
  ].join(' ')

  const child = spawn('/bin/sh', ['-c', shellCmd], {
    detached: true,
    stdio: 'ignore'
  })
  child.unref()
  fs.writeFileSync(BRIDGE_PID_FILE, String(child.pid))
  log(`Bridge restarted (PID ${child.pid})`)
  return child.pid
}

// ─── Tests ────────────────────────────────────────────────────────────────────

async function test1_normalWithdraw () {
  console.log('\n── TEST 1: Normal withdraw (2 TFT swap → 1 TFT net on Stellar) ──')
  const name = 'test1_normalWithdraw'
  const userAddress = getEnv('USER_ADDRESS')

  try {
    const beforeStellar = await stellarTFTBalance(userAddress, horizon, issuerAddress)
    log(`User Stellar TFT before: ${beforeStellar}`)

    const burnId = await swapToStellar(api, alice, 2, { userAddress })
    log(`Burn ID: ${burnId}`)

    const afterStellar = await waitUntil(async () => {
      const bal = await stellarTFTBalance(userAddress, horizon, issuerAddress)
      if (bal > beforeStellar) return bal
    }, { timeoutMs: 180_000, desc: `Stellar balance to increase above ${beforeStellar}` })

    const delta = Math.round((afterStellar - beforeStellar) * TFT_DECIMALS) / TFT_DECIMALS
    const expected = 2 - WITHDRAW_FEE_TFT
    if (Math.abs(delta - expected) > 0.0000001) {
      fail(name, `Expected +${expected} TFT, got +${delta}`, counter)
    } else {
      log(`User Stellar TFT after: ${afterStellar} (+${delta} TFT)`)
      pass(name, counter)
    }
  } catch (e) {
    fail(name, e.message, counter)
  }
}

async function test2_batchWithdraw () {
  console.log('\n── TEST 2: Batch withdraw (5 simultaneous swaps in one block) ──')
  const name = 'test2_batchWithdraw'
  const userAddress = getEnv('USER_ADDRESS')

  try {
    const beforeStellar = await stellarTFTBalance(userAddress, horizon, issuerAddress)
    log(`User Stellar TFT before: ${beforeStellar}`)

    const nonce = await api.rpc.system.accountNextIndex(alice.address)
    const burnIds = await Promise.all(
      [0, 1, 2, 3, 4].map(i => swapToStellar(api, alice, 2, { userAddress, nonce: nonce.toNumber() + i }))
    )
    log(`Burn IDs: ${burnIds.join(', ')}`)

    const expectedNet = 5 * (2 - WITHDRAW_FEE_TFT)
    const afterStellar = await waitUntil(async () => {
      const bal = await stellarTFTBalance(userAddress, horizon, issuerAddress)
      if (bal >= beforeStellar + expectedNet - 0.0000001) return bal
    }, { timeoutMs: 300_000, desc: `Stellar balance ≥ ${beforeStellar + expectedNet}` })

    const delta = Math.round((afterStellar - beforeStellar) * TFT_DECIMALS) / TFT_DECIMALS
    log(`User Stellar TFT after: ${afterStellar} (+${delta} TFT, expected +${expectedNet})`)
    if (Math.abs(delta - expectedNet) < 0.0000001) {
      pass(name, counter)
    } else {
      fail(name, `Expected +${expectedNet} TFT, got +${delta}`, counter)
    }
  } catch (e) {
    fail(name, e.message, counter)
  }
}

async function test3_badDeposit () {
  console.log('\n── TEST 3: Bad deposit (no memo → full refund) ──')
  const name = 'test3_badDeposit'
  const userAddress = getEnv('USER_ADDRESS')
  const userSecret = getEnv('USER_SECRET')

  try {
    const userKpStellar = StellarSdk.Keypair.fromSecret(userSecret)
    const TFTAsset = new StellarSdk.Asset('TFT', issuerAddress)
    const depositAmount = '3'

    const beforeStellar = await stellarTFTBalance(userAddress, horizon, issuerAddress)
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
      const bal = await stellarTFTBalance(userAddress, horizon, issuerAddress)
      if (bal >= beforeStellar - 0.0000001) return bal
    }, { timeoutMs: 180_000, desc: 'refund to restore balance' })

    const delta = Math.round((afterStellar - beforeStellar) * TFT_DECIMALS) / TFT_DECIMALS
    log(`User Stellar TFT after: ${afterStellar} (delta: ${delta >= 0 ? '+' : ''}${delta})`)
    // Balance should be within 0 (full refund, no deposit fee on refunds)
    if (Math.abs(delta) < 0.0000001) {
      pass(name, counter)
    } else {
      fail(name, `Expected net 0 change (full refund), got ${delta >= 0 ? '+' : ''}${delta}`, counter)
    }
  } catch (e) {
    fail(name, e.message, counter)
  }
}

async function test4_crashRecovery () {
  console.log('\n── TEST 4: Crash recovery (SIGKILL mid-withdraw, restart, verify delivery) ──')
  const name = 'test4_crashRecovery'
  const userAddress = getEnv('USER_ADDRESS')

  try {
    const beforeStellar = await stellarTFTBalance(userAddress, horizon, issuerAddress)
    log(`User Stellar TFT before: ${beforeStellar}`)

    // Trigger a withdraw
    const burnId = await swapToStellar(api, alice, 2, { userAddress })
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

    // Restart bridge.
    // Note: detecting bridge readiness via log file is unreliable on macOS because
    // detached process stdout fd inheritance breaks after child.unref(). Instead, we
    // give the bridge a fixed startup window and then verify the actual outcome.
    startBridge()
    log('Bridge restarted. Waiting 10s for startup...')
    await new Promise(r => setTimeout(r, 10_000))

    // Verify the withdrawal completed — either:
    //   (a) bridge completed before kill and balance is already updated, or
    //   (b) bridge restarted and completed via reconciliation / expiry recovery
    const afterStellar = await waitUntil(async () => {
      const bal = await stellarTFTBalance(userAddress, horizon, issuerAddress)
      if (bal > beforeStellar) return bal
    }, { timeoutMs: 300_000, desc: 'Stellar balance to increase after crash recovery' })

    const delta = Math.round((afterStellar - beforeStellar) * TFT_DECIMALS) / TFT_DECIMALS
    const expected = 2 - WITHDRAW_FEE_TFT
    log(`User Stellar TFT after: ${afterStellar} (+${delta} TFT)`)
    if (Math.abs(delta - expected) < 0.0000001) {
      pass(name, counter)
    } else {
      fail(name, `Expected +${expected} TFT after recovery, got +${delta}`, counter)
    }
  } catch (e) {
    fail(name, e.message, counter)
  }
}

// ─── Main ────────────────────────────────────────────────────────────────────

async function main () {
  loadEnv(ENV_FILE, 'tests')

  bridgeAddress = getEnv('BRIDGE_ADDRESS')
  issuerAddress = getEnv('ISSUER_ADDRESS')

  console.log('[tests] Connecting to TFChain and Stellar...')
  api = await ApiPromise.create({ provider: new WsProvider(TFCHAIN_URL) })
  horizon = new StellarSdk.Horizon.Server(HORIZON_URL)

  try {
    const keyring = new Keyring({ type: 'sr25519' })
    alice = keyring.addFromUri('//Alice')

    console.log('[tests] Starting test suite...\n')

    await test1_normalWithdraw()
    await test2_batchWithdraw()
    await test3_badDeposit()
    await test4_crashRecovery()

    console.log(`\n${'─'.repeat(50)}`)
    console.log(`Results: ${counter.passed} passed, ${counter.failed} failed`)
    console.log('─'.repeat(50))
  } finally {
    await api.disconnect()
  }

  process.exit(counter.failed > 0 ? 1 : 0)
}

main().catch(e => {
  console.error(`[tests] FATAL: ${e.message || e}`)
  process.exit(1)
})
