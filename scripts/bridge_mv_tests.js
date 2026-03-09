#!/usr/bin/env node
/**
 * bridge_mv_tests.js
 *
 * Multi-validator E2E test suite for the TFChain bridge.
 * Assumes 3 bridge daemons running (Val1=genesis-key-1, Val2=genesis-key-2, Val3=genesis-key-3),
 * bridge Stellar account configured as 2-of-3 multi-sig (threshold=2).
 * Val2 and Val3 are added via council governance in bridge-mv-setup.
 *
 * Tests (run sequentially):
 *   MV1 — Normal withdraw: 3 validators, 2-of-3 signatures, 1 TFT delivered
 *   MV2 — Deposit/mint: send TFT with valid memo, all 3 propose mint, threshold met
 *   MV3 — Bad deposit: no memo, all 3 detect and propose refund, full refund delivered
 *   MV4 — Validator offline: kill Val3 before deposit, Val1+Val2 alone complete refund (2-of-3)
 *   MV5 — Batch withdraws: 3 simultaneous swaps, all 3 eventually delivered (may use expiry)
 *   MV6 — Crash recovery: kill Val2 mid-withdraw, restart, verify delivery completes
 *   MV7 — Clean state: verify no orphaned active transactions on-chain
 *
 * All tests assert exact TFT balances (Stellar + TFChain) and on-chain state.
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
const {
  log, pass, fail,
  loadEnv, getEnv,
  stellarTFTBalance,
  tfchainBalance,
  waitUntil,
  swapToStellar,
  TFT_DECIMALS
} = require('./bridge_helpers')

const TFCHAIN_URL = process.env.TFCHAIN_URL || 'ws://localhost:9944'
const HORIZON_URL = process.env.STELLAR_HORIZON_URL || 'https://horizon-testnet.stellar.org'
const NETWORK_PASSPHRASE = StellarSdk.Networks.TESTNET
const ENV_FILE = process.env.BRIDGE_MV_ENV_FILE || '/tmp/bridge_mv_env.sh'
const BRIDGE_BIN = process.env.BRIDGE_BIN || './bridge/tfchain_bridge/tfchain_bridge_local'
const BRIDGE_DIR = process.env.BRIDGE_DIR || './bridge/tfchain_bridge'

const VAL_PID_FILES = [1, 2, 3].map(i => `/tmp/bridge_mv_${i}.pid`)
const VAL_LOG_FILES = [1, 2, 3].map(i => `/tmp/bridge_mv_${i}.log`)

const WITHDRAW_FEE_TFT = 1

const counter = { passed: 0, failed: 0 }
let api, alice, horizon, issuerAddress, bridgeAddress

// ─── Validator lifecycle helpers ────────────────────────────────────────────

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

// Bridge validator dev seeds (from substrate-node/node/src/chain_spec.rs)
// Read from env vars (set by Makefile) with hardcoded defaults as fallback.
const VAL_TFCHAIN_SEEDS = [
  process.env.VAL1_TFCHAIN_SEED || 'quarter between satisfy three sphere six soda boss cute decade old trend',
  process.env.VAL2_TFCHAIN_SEED || 'employ split promote annual couple elder remain cricket company fitness senior fiscal',
  process.env.VAL3_TFCHAIN_SEED || 'remind bird banner word spread volume card keep want faith insect mind'
]

function startValidator (valIndex) {
  const secrets = ['VAL1_STELLAR_SECRET', 'VAL2_STELLAR_SECRET', 'VAL3_STELLAR_SECRET']
  const secret = getEnv(secrets[valIndex - 1])
  const seed = VAL_TFCHAIN_SEEDS[valIndex - 1]
  const persistency = `${BRIDGE_DIR}/signer_mv_${valIndex}.json`
  const logFile = VAL_LOG_FILES[valIndex - 1]

  // Use shell exec redirect — direct fd inheritance is unreliable on macOS after child.unref()
  const cmd = [
    BRIDGE_BIN,
    '--secret', secret,
    '--tfchainurl', TFCHAIN_URL,
    '--tfchainseed', `"${seed}"`,
    '--bridgewallet', bridgeAddress,
    '--persistency', persistency,
    '--network', 'local'
  ].join(' ')

  const child = spawn('/bin/sh', ['-c', `exec ${cmd} >> ${logFile} 2>&1`], {
    detached: true,
    stdio: 'ignore'
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

// ─── On-chain assertion helpers ─────────────────────────────────────────────

/**
 * Poll until a burn tx moves to ExecutedBurnTransactions (not stuck in active map).
 * Waits up to 30s for the on-chain state to settle — set_burn_transaction_executed
 * may finalize a few blocks after the Stellar payment is visible.
 */
async function assertBurnExecuted (name, burnId) {
  try {
    await waitUntil(async () => {
      const active = (await api.query.tftBridgeModule.burnTransactions(burnId)).toJSON()
      if (active && active.target) return false
      const executed = (await api.query.tftBridgeModule.executedBurnTransactions(burnId)).toJSON()
      return executed && executed.target
    }, { timeoutMs: 30_000, intervalMs: 3000, desc: `burn ${burnId} to reach ExecutedBurnTransactions` })
    return true
  } catch {
    const active = (await api.query.tftBridgeModule.burnTransactions(burnId)).toJSON()
    if (active && active.target) {
      fail(name, `burn ${burnId} still in active BurnTransactions after 30s`, counter)
    } else {
      fail(name, `burn ${burnId} not in ExecutedBurnTransactions after 30s`, counter)
    }
    return false
  }
}

/**
 * Poll until at least one new refund reaches ExecutedRefundTransactions since `countBefore`.
 * Waits up to 30s — set_refund_transaction_executed may finalize after the Stellar refund.
 */
async function assertRefundExecuted (name, countBefore) {
  try {
    await waitUntil(async () => {
      const after = await api.query.tftBridgeModule.executedRefundTransactions.entries()
      return after.length > countBefore
    }, { timeoutMs: 30_000, intervalMs: 3000, desc: 'new refund in ExecutedRefundTransactions' })
    return true
  } catch {
    const after = await api.query.tftBridgeModule.executedRefundTransactions.entries()
    fail(name, `no new refund in ExecutedRefundTransactions after 30s (before: ${countBefore}, after: ${after.length})`, counter)
    return false
  }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

async function testMV1_normalWithdraw () {
  console.log('\n── MV1: Normal withdraw (3 validators, threshold=2) ──')
  const name = 'MV1_normalWithdraw'
  const userAddress = getEnv('USER_ADDRESS')
  const swapAmount = 2

  try {
    const beforeStellar = await stellarTFTBalance(userAddress, horizon, issuerAddress)
    const beforeTFChain = await tfchainBalance(api, alice.address)
    log(`User Stellar TFT before: ${beforeStellar}`)
    log(`Alice TFChain TFT before: ${beforeTFChain}`)

    const burnId = await swapToStellar(api, alice, swapAmount, { userAddress })
    log(`Burn ID: ${burnId}`)

    const afterStellar = await waitUntil(async () => {
      const bal = await stellarTFTBalance(userAddress, horizon, issuerAddress)
      if (bal > beforeStellar) return bal
    }, { timeoutMs: 300_000, intervalMs: 4000, desc: 'Stellar balance to increase' })

    // Assert Stellar balance delta
    const delta = Math.round((afterStellar - beforeStellar) * TFT_DECIMALS) / TFT_DECIMALS
    const expected = swapAmount - WITHDRAW_FEE_TFT
    log(`User Stellar TFT after: ${afterStellar} (+${delta} TFT)`)
    if (Math.abs(delta - expected) > 1e-7) {
      fail(name, `Expected Stellar +${expected} TFT, got +${delta}`, counter); return
    }

    // Assert TFChain balance decreased by ~swapAmount (± 0.1 TFT for extrinsic fee).
    const afterTFChain = await tfchainBalance(api, alice.address)
    const tfDelta = Math.round((beforeTFChain - afterTFChain) * TFT_DECIMALS) / TFT_DECIMALS
    log(`Alice TFChain TFT after: ${afterTFChain} (-${tfDelta} TFT)`)
    if (Math.abs(tfDelta - swapAmount) > 0.1) {
      fail(name, `TFChain balance should decrease by ~${swapAmount} (±0.1), decreased by ${tfDelta}`, counter); return
    }

    // Assert on-chain: burn executed
    if (!(await assertBurnExecuted(name, burnId))) return

    pass(name, counter)
  } catch (e) { fail(name, e.message, counter) }
}

async function testMV2_deposit () {
  console.log('\n── MV2: Deposit/mint (3 validators all propose) ──')
  const name = 'MV2_deposit'
  const aliceAddress = alice.address

  try {
    // Get Alice's twin ID — twinIdByAccountID returns Option<u32>
    const twinOpt = await api.query.tfgridModule.twinIdByAccountID(aliceAddress)
    const twinId = twinOpt.isSome ? twinOpt.unwrap().toNumber() : twinOpt.toJSON()
    if (!twinId) throw new Error('Alice has no twin on TFChain — is bridge-setup complete?')
    log(`Alice twin ID: ${twinId}`)

    const depositAmount = '2'
    const depositFee = Number(await api.query.tftBridgeModule.depositFee()) / TFT_DECIMALS
    const expectedMint = parseFloat(depositAmount) - depositFee
    log(`Deposit fee: ${depositFee} TFT, expected mint: ${expectedMint} TFT`)

    const aliceBalBefore = await tfchainBalance(api, aliceAddress)
    const mintsBefore = (await api.query.tftBridgeModule.executedMintTransactions.entries()).length
    log(`Alice TFChain TFT before: ${aliceBalBefore}, executed mints: ${mintsBefore}`)

    // Memo format must be "twin_<id>" (bridge parses "object_objectID")
    const result = await sendStellarPayment(
      getEnv('USER_SECRET'),
      bridgeAddress,
      depositAmount,
      `twin_${twinId}`
    )
    log(`Deposit sent: ${result.hash.slice(0, 16)} (memo: twin_${twinId})`)

    // Wait for mint to be executed on TFChain
    const mintsAfter = await waitUntil(async () => {
      const mints = await api.query.tftBridgeModule.executedMintTransactions.entries()
      if (mints.length > mintsBefore) return mints
    }, { timeoutMs: 120_000, intervalMs: 4000, desc: 'executed mint count to increase' })

    log(`Executed mints after: ${mintsAfter.length}`)

    // Assert Alice's TFChain balance increased by ~expectedMint (± 0.1 TFT for block author rewards).
    const aliceBalAfter = await tfchainBalance(api, aliceAddress)
    const balDelta = Math.round((aliceBalAfter - aliceBalBefore) * TFT_DECIMALS) / TFT_DECIMALS
    log(`Alice TFChain TFT after: ${aliceBalAfter} (+${balDelta} TFT)`)
    if (Math.abs(balDelta - expectedMint) > 0.1) {
      fail(name, `Expected TFChain ~+${expectedMint} TFT (±0.1), got +${balDelta}`, counter); return
    }

    pass(name, counter)
  } catch (e) { fail(name, e.message, counter) }
}

async function testMV3_badDeposit () {
  console.log('\n── MV3: Bad deposit (no memo -> full refund, 3 validators) ──')
  const name = 'MV3_badDeposit'
  const userAddress = getEnv('USER_ADDRESS')

  try {
    const beforeStellar = await stellarTFTBalance(userAddress, horizon, issuerAddress)
    const refundsBefore = (await api.query.tftBridgeModule.executedRefundTransactions.entries()).length
    log(`User Stellar TFT before: ${beforeStellar}`)

    const result = await sendStellarPayment(getEnv('USER_SECRET'), bridgeAddress, '3')
    log(`Bad deposit sent: ${result.hash.slice(0, 16)} (no memo)`)

    const afterStellar = await waitUntil(async () => {
      const bal = await stellarTFTBalance(userAddress, horizon, issuerAddress)
      if (bal >= beforeStellar - 1e-7) return bal
    }, { timeoutMs: 180_000, intervalMs: 4000, desc: 'balance restored after refund' })

    const delta = Math.round((afterStellar - beforeStellar) * TFT_DECIMALS) / TFT_DECIMALS
    log(`User Stellar TFT after: ${afterStellar} (delta: ${delta >= 0 ? '+' : ''}${delta})`)

    if (Math.abs(delta) > 1e-7) {
      fail(name, `Expected net 0 (full refund), got ${delta >= 0 ? '+' : ''}${delta}`, counter); return
    }

    // Assert on-chain: refund executed
    if (!(await assertRefundExecuted(name, refundsBefore))) return

    pass(name, counter)
  } catch (e) { fail(name, e.message, counter) }
}

async function testMV4_validatorOffline () {
  console.log('\n── MV4: Val3 offline — Val1+Val2 complete refund with 2-of-3 threshold ──')
  const name = 'MV4_validatorOffline'
  const userAddress = getEnv('USER_ADDRESS')

  try {
    // Kill Val3 before the deposit — threshold=2, so Val1+Val2 alone can complete
    killValidator(3)
    await new Promise(r => setTimeout(r, 2000))

    const beforeStellar = await stellarTFTBalance(userAddress, horizon, issuerAddress)
    const refundsBefore = (await api.query.tftBridgeModule.executedRefundTransactions.entries()).length
    log(`Val3 killed. User Stellar TFT before: ${beforeStellar}`)

    // Send bad deposit (no memo) — Val1+Val2 detect it, propose refund, threshold=2 met
    const result = await sendStellarPayment(getEnv('USER_SECRET'), bridgeAddress, '4')
    log(`Bad deposit sent: ${result.hash.slice(0, 16)} (no memo, Val3 offline)`)

    // Wait for refund to complete — Val1+Val2 have enough signatures (2-of-3)
    const afterStellar = await waitUntil(async () => {
      const bal = await stellarTFTBalance(userAddress, horizon, issuerAddress)
      if (bal >= beforeStellar - 1e-7) return bal
    }, { timeoutMs: 180_000, intervalMs: 4000, desc: 'balance restored with Val3 offline' })

    const delta = Math.round((afterStellar - beforeStellar) * TFT_DECIMALS) / TFT_DECIMALS
    log(`User Stellar TFT after: ${afterStellar} (delta: ${delta >= 0 ? '+' : ''}${delta})`)

    // Evaluate result NOW — before restart attempt (restart is cleanup, not part of the test)
    const balancePassed = Math.abs(delta) < 1e-7
    const refundPassed = await (async () => {
      const afterRefunds = await api.query.tftBridgeModule.executedRefundTransactions.entries()
      return afterRefunds.length > refundsBefore
    })()

    // Restart Val3 for subsequent tests — best effort with fixed startup window.
    // Log-based readiness detection is unreliable on macOS for restarted processes.
    try {
      log('Restarting Val3 for subsequent tests...')
      startValidator(3)
      await new Promise(r => setTimeout(r, 8000)) // fixed startup window
      log('Val3 restarted.')
    } catch (restartErr) {
      log(`Warning: Val3 restart failed: ${restartErr.message}`)
    }

    if (!balancePassed) {
      fail(name, `Expected net 0 (full refund), got ${delta >= 0 ? '+' : ''}${delta}`, counter)
    } else if (!refundPassed) {
      fail(name, `Stellar balance correct but no new refund in ExecutedRefundTransactions`, counter)
    } else {
      pass(name, counter)
    }
  } catch (e) {
    fail(name, e.message, counter)
    // Best-effort Val3 restart so subsequent tests still run
    try { startValidator(3); await new Promise(r => setTimeout(r, 5000)) } catch {}
  }
}

async function testMV5_batchWithdraws () {
  console.log('\n── MV5: Batch withdraws (3 simultaneous, all validators) ──')
  const name = 'MV5_batchWithdraws'
  const userAddress = getEnv('USER_ADDRESS')

  try {
    const beforeStellar = await stellarTFTBalance(userAddress, horizon, issuerAddress)
    log(`User Stellar TFT before: ${beforeStellar}`)

    const nonce = await api.rpc.system.accountNextIndex(alice.address)
    const burnIds = await Promise.all(
      [0, 1, 2].map(i => swapToStellar(api, alice, 2, { userAddress, nonce: nonce.toNumber() + i }))
    )
    log(`Burn IDs: ${burnIds.join(', ')}`)

    const expectedNet = 3 * (2 - WITHDRAW_FEE_TFT)

    // Use longer timeout — sequence collisions may require expiry cycle (~2 min each)
    const afterStellar = await waitUntil(async () => {
      const bal = await stellarTFTBalance(userAddress, horizon, issuerAddress)
      if (bal >= beforeStellar + expectedNet - 1e-7) return bal
    }, { timeoutMs: 600_000, intervalMs: 4000, desc: `balance >= ${beforeStellar + expectedNet} (may need expiry cycles)` })

    const delta = Math.round((afterStellar - beforeStellar) * TFT_DECIMALS) / TFT_DECIMALS
    log(`User Stellar TFT after: ${afterStellar} (+${delta}, expected +${expectedNet})`)
    if (Math.abs(delta - expectedNet) > 1e-7) {
      fail(name, `Expected +${expectedNet}, got +${delta}`, counter); return
    }

    // Assert on-chain: all burns executed
    for (const burnId of burnIds) {
      if (!(await assertBurnExecuted(name, burnId))) return
    }

    pass(name, counter)
  } catch (e) { fail(name, e.message, counter) }
}

async function testMV6_crashRecovery () {
  console.log('\n── MV6: Crash recovery (kill Val2 mid-withdraw, restart, verify delivery) ──')
  const name = 'MV6_crashRecovery'
  const userAddress = getEnv('USER_ADDRESS')

  try {
    const beforeStellar = await stellarTFTBalance(userAddress, horizon, issuerAddress)
    log(`User Stellar TFT before: ${beforeStellar}`)

    const burnId = await swapToStellar(api, alice, 2, { userAddress })
    log(`Burn ID: ${burnId}`)

    // Wait for at least 1 signature (proposals submitted)
    await waitUntil(async () => {
      const burn = (await api.query.tftBridgeModule.burnTransactions(burnId)).toJSON()
      return burn && burn.signatures && burn.signatures.length >= 1
    }, { timeoutMs: 60_000, desc: 'BurnTransactionReady (>=1 sig)' })

    // Kill Val2 mid-flight
    killValidator(2, 'SIGKILL')
    log('Val2 killed. Waiting 3s...')
    await new Promise(r => setTimeout(r, 3000))

    // Restart Val2
    startValidator(2)
    log('Val2 restarted. Waiting 10s for startup...')
    await new Promise(r => setTimeout(r, 10_000))

    // Val1+Val3 should complete it (2-of-3), or Val2 reconciles after restart
    const afterStellar = await waitUntil(async () => {
      const bal = await stellarTFTBalance(userAddress, horizon, issuerAddress)
      if (bal > beforeStellar) return bal
    }, { timeoutMs: 300_000, intervalMs: 4000, desc: 'Stellar balance to increase after crash' })

    const delta = Math.round((afterStellar - beforeStellar) * TFT_DECIMALS) / TFT_DECIMALS
    const expected = 2 - WITHDRAW_FEE_TFT
    log(`User Stellar TFT after: ${afterStellar} (+${delta} TFT)`)

    if (Math.abs(delta - expected) > 1e-7) {
      fail(name, `Expected +${expected} TFT after recovery, got +${delta}`, counter); return
    }

    // Assert on-chain: burn executed
    if (!(await assertBurnExecuted(name, burnId))) return

    pass(name, counter)
  } catch (e) { fail(name, e.message, counter) }
}

async function testMV7_cleanState () {
  console.log('\n── MV7: Clean state (no orphaned active transactions) ──')
  const name = 'MV7_cleanState'

  try {
    // Wait for all active transaction maps to drain (tolerates in-flight processing)
    await waitUntil(async () => {
      const burns = await api.query.tftBridgeModule.burnTransactions.entries()
      const refunds = await api.query.tftBridgeModule.refundTransactions.entries()
      const mints = await api.query.tftBridgeModule.mintTransactions.entries()
      return burns.length === 0 && refunds.length === 0 && mints.length === 0
    }, { timeoutMs: 60_000, intervalMs: 5000, desc: 'all active tx maps to drain' })
    pass(name, counter)
  } catch (e) {
    // On timeout, report what's left
    const burns = await api.query.tftBridgeModule.burnTransactions.entries()
    const refunds = await api.query.tftBridgeModule.refundTransactions.entries()
    const mints = await api.query.tftBridgeModule.mintTransactions.entries()
    fail(name, `Orphaned: ${burns.length} burns, ${refunds.length} refunds, ${mints.length} mints`, counter)
  }
}

// ─── Main ────────────────────────────────────────────────────────────────────

async function main () {
  loadEnv(ENV_FILE, 'mv-tests')
  bridgeAddress = getEnv('BRIDGE_ADDRESS')
  issuerAddress = getEnv('ISSUER_ADDRESS')

  console.log('[mv-tests] Connecting to TFChain and Stellar...')
  api = await ApiPromise.create({ provider: new WsProvider(TFCHAIN_URL) })
  horizon = new StellarSdk.Horizon.Server(HORIZON_URL)

  try {
    const keyring = new Keyring({ type: 'sr25519' })
    alice = keyring.addFromUri('//Alice')

    console.log('[mv-tests] Starting multi-validator test suite...\n')

    await testMV1_normalWithdraw()
    await testMV2_deposit()
    await testMV3_badDeposit()
    await testMV4_validatorOffline()  // kills/restarts Val3
    await testMV5_batchWithdraws()
    await testMV6_crashRecovery()     // kills/restarts Val2
    await testMV7_cleanState()

    console.log(`\n${'─'.repeat(50)}`)
    console.log(`Results: ${counter.passed} passed, ${counter.failed} failed`)
    console.log('─'.repeat(50))
  } finally {
    await api.disconnect()
  }

  process.exit(counter.failed > 0 ? 1 : 0)
}

main().catch(e => {
  console.error(`[mv-tests] FATAL: ${e.message || e}`)
  process.exit(1)
})
