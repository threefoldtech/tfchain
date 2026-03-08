#!/usr/bin/env node
/**
 * bridge_mv_tests.js
 *
 * Multi-validator E2E test suite for the TFChain bridge.
 * Assumes 3 bridge daemons running (Val1=Alice, Val2=Bob, Val3=Charlie),
 * bridge Stellar account configured as 2-of-3 multi-sig (threshold=2).
 *
 * Tests (run sequentially):
 *   MV1 — Normal withdraw: 3 validators, 2-of-3 signatures, 1 TFT delivered
 *   MV2 — Deposit/mint: send TFT with valid memo, all 3 propose mint, threshold met
 *   MV3 — Bad deposit: no memo, all 3 detect and propose refund, full refund delivered
 *   MV4 — Validator offline: kill Val3, bad deposit, Val1+Val2 meet threshold=2, refund works
 *   MV5 — Batch withdraws: 3 simultaneous swaps, all 3 eventually delivered (may use expiry)
 *
 * Non-zero exit on any failure.
 *
 * Usage:
 *   node scripts/bridge_mv_tests.js
 */

'use strict'

const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api')
const StellarSdk = require('@stellar/stellar-sdk')
const fs = require('fs')
const { spawn } = require('child_process')

const TFCHAIN_URL = process.env.TFCHAIN_URL || 'ws://localhost:9944'
const HORIZON_URL = process.env.STELLAR_HORIZON_URL || 'https://horizon-testnet.stellar.org'
const NETWORK_PASSPHRASE = StellarSdk.Networks.TESTNET
const ENV_FILE = process.env.BRIDGE_MV_ENV_FILE || '/tmp/bridge_mv_env.sh'
const BRIDGE_BIN = process.env.BRIDGE_BIN || './bridge/tfchain_bridge/tfchain_bridge_local'
const BRIDGE_DIR = process.env.BRIDGE_DIR || './bridge/tfchain_bridge'

const VAL_PID_FILES = [1, 2, 3].map(i => `/tmp/bridge_mv_${i}.pid`)
const VAL_LOG_FILES = [1, 2, 3].map(i => `/tmp/bridge_mv_${i}.log`)

const WITHDRAW_FEE_TFT = 1
const TFT_DECIMALS = 1e7

let passed = 0
let failed = 0
let api, alice, horizon, issuerAddress, bridgeAddress

// ─── Helpers ─────────────────────────────────────────────────────────────────

function log (msg) { console.log(`  ${msg}`) }
function pass (name) { console.log(`✅ PASS: ${name}`); passed++ }
function fail (name, reason) { console.error(`❌ FAIL: ${name} — ${reason}`); failed++ }

function loadEnv () {
  if (!fs.existsSync(ENV_FILE)) {
    console.error(`[mv-tests] Env file not found: ${ENV_FILE}. Run 'make bridge-mv-accounts' first.`)
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

async function waitUntil (condition, { timeoutMs = 300_000, intervalMs = 4000, desc = '' } = {}) {
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
    api.tx.tftBridgeModule.swapToStellar(userAddress, Math.round(amount * TFT_DECIMALS))
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

async function sendStellarPayment (fromSecret, toAddress, amount, memo = null) {
  const kp = StellarSdk.Keypair.fromSecret(fromSecret)
  const TFTAsset = new StellarSdk.Asset('TFT', issuerAddress)
  const acc = await horizon.loadAccount(kp.publicKey())

  const builder = new StellarSdk.TransactionBuilder(acc, {
    fee: '1000',
    networkPassphrase: NETWORK_PASSPHRASE
  }).addOperation(StellarSdk.Operation.payment({
    destination: toAddress,
    asset: TFTAsset,
    amount: String(amount)
  })).setTimeout(30)

  if (memo) builder.addMemo(StellarSdk.Memo.text(String(memo)))

  const tx = builder.build()
  tx.sign(kp)
  return horizon.submitTransaction(tx)
}

function getValPid (valIndex) {
  const pidFile = VAL_PID_FILES[valIndex - 1]
  if (!fs.existsSync(pidFile)) return null
  return parseInt(fs.readFileSync(pidFile, 'utf8').trim())
}

function killValidator (valIndex, signal = 'SIGKILL') {
  const pid = getValPid(valIndex)
  if (pid) {
    try { process.kill(pid, signal); log(`Val${valIndex} (PID ${pid}) killed`) } catch {}
  }
}

function startValidator (valIndex) {
  const secrets = ['VAL1_STELLAR_SECRET', 'VAL2_STELLAR_SECRET', 'VAL3_STELLAR_SECRET']
  const seeds = ['//Alice', '//Bob', '//Charlie']
  const secret = getEnv(secrets[valIndex - 1])
  const seed = seeds[valIndex - 1]
  const persistency = `${BRIDGE_DIR}/signer_mv_${valIndex}.json`
  const logFile = VAL_LOG_FILES[valIndex - 1]

  const child = spawn(BRIDGE_BIN, [
    '--secret', secret,
    '--tfchainurl', TFCHAIN_URL,
    '--tfchainseed', seed,
    '--bridgewallet', bridgeAddress,
    '--persistency', persistency,
    '--network', 'testnet'
  ], {
    detached: true,
    stdio: ['ignore', fs.openSync(logFile, 'a'), fs.openSync(logFile, 'a')]
  })
  child.unref()
  fs.writeFileSync(VAL_PID_FILES[valIndex - 1], String(child.pid))
  log(`Val${valIndex} restarted (PID ${child.pid})`)
}

async function waitForValReady (valIndex) {
  const logFile = VAL_LOG_FILES[valIndex - 1]
  await waitUntil(async () => {
    if (!fs.existsSync(logFile)) return false
    const tail = fs.readFileSync(logFile, 'utf8').slice(-20000)
    return tail.includes('bridge_started')
  }, { timeoutMs: 30_000, desc: `Val${valIndex} bridge_started` })
}

// ─── Tests ────────────────────────────────────────────────────────────────────

async function testMV1_normalWithdraw () {
  console.log('\n── MV1: Normal withdraw (3 validators, threshold=2) ──')
  const name = 'MV1_normalWithdraw'
  const userAddress = getEnv('USER_ADDRESS')

  try {
    const before = await stellarTFTBalance(userAddress)
    log(`User Stellar TFT before: ${before}`)

    const burnId = await swapToStellar(2)
    log(`Burn ID: ${burnId}`)

    const after = await waitUntil(async () => {
      const bal = await stellarTFTBalance(userAddress)
      if (bal > before) return bal
    }, { timeoutMs: 300_000, desc: 'Stellar balance to increase' })

    const delta = Math.round((after - before) * TFT_DECIMALS) / TFT_DECIMALS
    const expected = 2 - WITHDRAW_FEE_TFT
    log(`User Stellar TFT after: ${after} (+${delta})`)

    if (Math.abs(delta - expected) < 1e-7) {
      pass(name)
    } else {
      fail(name, `Expected +${expected}, got +${delta}`)
    }
  } catch (e) { fail(name, e.message) }
}

async function testMV2_deposit () {
  console.log('\n── MV2: Deposit/mint (3 validators all propose) ──')
  const name = 'MV2_deposit'
  const userAddress = getEnv('USER_ADDRESS')
  const aliceAddress = alice.address

  try {
    // Get Alice's TFChain TFT balance (minted TFT, not native)
    // We check executed mints on TFChain instead of TFT balance
    const mintsBefore = await api.query.tftBridgeModule.executedMintTransactions.entries()
    log(`Executed mints before: ${mintsBefore.length}`)

    // Send 2 TFT from user to bridge with Alice's TFChain address as memo (twin ID)
    // First, get Alice's twin ID
    const twin = await api.query.tfgridModule.twinIdByAccountID(aliceAddress)
    const twinId = twin.toNumber()
    log(`Alice twin ID: ${twinId}`)

    const result = await sendStellarPayment(
      getEnv('USER_SECRET'),
      bridgeAddress,
      '2',
      String(twinId)
    )
    log(`Deposit sent: ${result.hash.slice(0, 16)} (memo: twin ${twinId})`)

    // Wait for mint to be executed on TFChain
    const mintsAfter = await waitUntil(async () => {
      const mints = await api.query.tftBridgeModule.executedMintTransactions.entries()
      if (mints.length > mintsBefore.length) return mints
    }, { timeoutMs: 120_000, desc: 'executed mint count to increase' })

    log(`Executed mints after: ${mintsAfter.length}`)
    pass(name)
  } catch (e) { fail(name, e.message) }
}

async function testMV3_badDeposit () {
  console.log('\n── MV3: Bad deposit (no memo → full refund, 3 validators) ──')
  const name = 'MV3_badDeposit'
  const userAddress = getEnv('USER_ADDRESS')

  try {
    const before = await stellarTFTBalance(userAddress)
    log(`User Stellar TFT before: ${before}`)

    const result = await sendStellarPayment(getEnv('USER_SECRET'), bridgeAddress, '3')
    log(`Bad deposit sent: ${result.hash.slice(0, 16)} (no memo)`)

    const after = await waitUntil(async () => {
      const bal = await stellarTFTBalance(userAddress)
      if (bal >= before - 1e-7) return bal
    }, { timeoutMs: 180_000, desc: 'balance restored after refund' })

    const delta = Math.round((after - before) * TFT_DECIMALS) / TFT_DECIMALS
    log(`User Stellar TFT after: ${after} (delta: ${delta >= 0 ? '+' : ''}${delta})`)

    if (Math.abs(delta) < 1e-7) {
      pass(name)
    } else {
      fail(name, `Expected net 0 (full refund), got ${delta >= 0 ? '+' : ''}${delta}`)
    }
  } catch (e) { fail(name, e.message) }
}

async function testMV4_validatorOffline () {
  console.log('\n── MV4: Val3 offline — bad deposit, Val1+Val2 meet threshold=2 ──')
  const name = 'MV4_validatorOffline'
  const userAddress = getEnv('USER_ADDRESS')

  try {
    // Kill Val3
    killValidator(3)
    await new Promise(r => setTimeout(r, 2000))

    const before = await stellarTFTBalance(userAddress)
    log(`Val3 killed. User Stellar TFT before: ${before}`)

    const result = await sendStellarPayment(getEnv('USER_SECRET'), bridgeAddress, '4')
    log(`Bad deposit sent: ${result.hash.slice(0, 16)} (no memo, Val3 offline)`)

    const after = await waitUntil(async () => {
      const bal = await stellarTFTBalance(userAddress)
      if (bal >= before - 1e-7) return bal
    }, { timeoutMs: 180_000, desc: 'balance restored with only 2 validators' })

    const delta = Math.round((after - before) * TFT_DECIMALS) / TFT_DECIMALS
    log(`User Stellar TFT after: ${after} (delta: ${delta >= 0 ? '+' : ''}${delta})`)

    if (Math.abs(delta) < 1e-7) {
      pass(name)
    } else {
      fail(name, `Expected net 0 (full refund), got ${delta >= 0 ? '+' : ''}${delta}`)
    }

    // Restart Val3 for subsequent tests
    log('Restarting Val3...')
    startValidator(3)
    await waitForValReady(3)
    log('Val3 back online.')
  } catch (e) {
    // Ensure Val3 is restarted even on failure
    try { startValidator(3); await waitForValReady(3) } catch {}
    fail(name, e.message)
  }
}

async function testMV5_batchWithdraws () {
  console.log('\n── MV5: Batch withdraws (3 simultaneous, all 3 validators) ──')
  const name = 'MV5_batchWithdraws'
  const userAddress = getEnv('USER_ADDRESS')

  try {
    const before = await stellarTFTBalance(userAddress)
    log(`User Stellar TFT before: ${before}`)

    const nonce = await api.rpc.system.accountNextIndex(alice.address)
    const burnIds = await Promise.all(
      [0, 1, 2].map(i => swapToStellar(2, nonce.toNumber() + i))
    )
    log(`Burn IDs: ${burnIds.join(', ')}`)

    const expectedNet = 3 * (2 - WITHDRAW_FEE_TFT)

    // Use longer timeout — sequence collisions may require expiry cycle (~2 min each)
    const after = await waitUntil(async () => {
      const bal = await stellarTFTBalance(userAddress)
      if (bal >= before + expectedNet - 1e-7) return bal
    }, { timeoutMs: 600_000, desc: `balance ≥ ${before + expectedNet} (may need expiry cycles)` })

    const delta = Math.round((after - before) * TFT_DECIMALS) / TFT_DECIMALS
    log(`User Stellar TFT after: ${after} (+${delta}, expected +${expectedNet})`)

    if (Math.abs(delta - expectedNet) < 1e-7) {
      pass(name)
    } else {
      fail(name, `Expected +${expectedNet}, got +${delta}`)
    }
  } catch (e) { fail(name, e.message) }
}

// ─── Main ────────────────────────────────────────────────────────────────────

async function main () {
  loadEnv()
  bridgeAddress = getEnv('BRIDGE_ADDRESS')
  issuerAddress = getEnv('ISSUER_ADDRESS')

  console.log('[mv-tests] Connecting to TFChain and Stellar...')
  api = await ApiPromise.create({ provider: new WsProvider(TFCHAIN_URL) })
  horizon = new StellarSdk.Horizon.Server(HORIZON_URL)

  const keyring = new Keyring({ type: 'sr25519' })
  alice = keyring.addFromUri('//Alice')

  console.log('[mv-tests] Starting multi-validator test suite...\n')

  await testMV1_normalWithdraw()
  await testMV2_deposit()
  await testMV3_badDeposit()
  await testMV4_validatorOffline()
  await testMV5_batchWithdraws()

  console.log(`\n${'─'.repeat(50)}`)
  console.log(`Results: ${passed} passed, ${failed} failed`)
  console.log('─'.repeat(50))

  await api.disconnect()
  process.exit(failed > 0 ? 1 : 0)
}

main().catch(e => {
  console.error(`[mv-tests] FATAL: ${e.message || e}`)
  process.exit(1)
})
