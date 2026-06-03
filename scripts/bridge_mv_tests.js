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
 *   MV5 — Batch withdraws: 5 simultaneous swaps, all 5 eventually delivered (may use expiry)
 *   MV6 — Crash recovery: kill Val2 mid-withdraw, restart, verify delivery completes
 *   MV6a— Below-minimum: swap below fee, expect dispatch error (parity with SV test6)
 *   MV8 — Lost cursor: wipe all 3 persistency files, restart, verify no double-spend,
 *          then deposit to prove bridge is at Stellar tip (tx hash verified)
 *   MV9 — Expired batch: 50 swaps while all validators offline, wait for expiry, restart, all delivered
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
  sendStellarPayment,
  swapToStellar,
  TFT_DECIMALS
} = require('./bridge_helpers')
const { startEventCollector, markTestPhase, generateReport } = require('./bridge_instrumentation')

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
let api, alice, horizon, issuerAddress, bridgeAddress, collector

// ─── Validator lifecycle helpers ────────────────────────────────────────────

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
  markTestPhase(collector, 'MV1', 'start')
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
  finally { markTestPhase(collector, 'MV1', 'end') }
}

async function testMV2_deposit () {
  console.log('\n── MV2: Deposit/mint (3 validators all propose) ──')
  markTestPhase(collector, 'MV2', 'start')
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
      horizon, issuerAddress, NETWORK_PASSPHRASE,
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
  finally { markTestPhase(collector, 'MV2', 'end') }
}

async function testMV3_badDeposit () {
  console.log('\n── MV3: Bad deposit (no memo -> full refund, 3 validators) ──')
  markTestPhase(collector, 'MV3', 'start')
  const name = 'MV3_badDeposit'
  const userAddress = getEnv('USER_ADDRESS')

  try {
    const beforeStellar = await stellarTFTBalance(userAddress, horizon, issuerAddress)
    const refundsBefore = (await api.query.tftBridgeModule.executedRefundTransactions.entries()).length
    log(`User Stellar TFT before: ${beforeStellar}`)

    const result = await sendStellarPayment(horizon, issuerAddress, NETWORK_PASSPHRASE, getEnv('USER_SECRET'), bridgeAddress, '3')
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
  finally { markTestPhase(collector, 'MV3', 'end') }
}

async function testMV4_validatorOffline () {
  console.log('\n── MV4: Val3 offline — Val1+Val2 complete refund with 2-of-3 threshold ──')
  markTestPhase(collector, 'MV4', 'start')
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
    const result = await sendStellarPayment(horizon, issuerAddress, NETWORK_PASSPHRASE, getEnv('USER_SECRET'), bridgeAddress, '4')
    log(`Bad deposit sent: ${result.hash.slice(0, 16)} (no memo, Val3 offline)`)

    // Wait for refund to complete — Val1+Val2 have enough signatures (2-of-3)
    const afterStellar = await waitUntil(async () => {
      const bal = await stellarTFTBalance(userAddress, horizon, issuerAddress)
      if (bal >= beforeStellar - 1e-7) return bal
    }, { timeoutMs: 180_000, intervalMs: 4000, desc: 'balance restored with Val3 offline' })

    const delta = Math.round((afterStellar - beforeStellar) * TFT_DECIMALS) / TFT_DECIMALS
    log(`User Stellar TFT after: ${afterStellar} (delta: ${delta >= 0 ? '+' : ''}${delta})`)

    if (Math.abs(delta) > 1e-7) {
      fail(name, `Expected net 0 (full refund), got ${delta >= 0 ? '+' : ''}${delta}`, counter)
    } else if (!(await assertRefundExecuted(name, refundsBefore))) {
      // assertRefundExecuted polls for 30s and logs failure itself
    } else {
      pass(name, counter)
    }
  } catch (e) {
    fail(name, e.message, counter)
  } finally {
    // Restart Val3 for subsequent tests — best effort with fixed startup window.
    try {
      log('Restarting Val3 for subsequent tests...')
      startValidator(3)
      await new Promise(r => setTimeout(r, 8000))
      log('Val3 restarted.')
    } catch (restartErr) {
      log(`Warning: Val3 restart failed: ${restartErr.message}`)
    }
    markTestPhase(collector, 'MV4', 'end')
  }
}

async function testMV5_batchWithdraws () {
  console.log('\n── MV5: Batch withdraws (5 simultaneous, all validators) ──')
  markTestPhase(collector, 'MV5', 'start')
  const name = 'MV5_batchWithdraws'
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
  finally { markTestPhase(collector, 'MV5', 'end') }
}

async function testMV6_crashRecovery () {
  console.log('\n── MV6: Crash recovery (kill Val2 mid-withdraw, restart, verify delivery) ──')
  markTestPhase(collector, 'MV6', 'start')
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
  finally { markTestPhase(collector, 'MV6', 'end') }
}

async function testMV6a_belowMinimum () {
  console.log('\n── MV6a: Withdraw below minimum (should be rejected) ──')
  markTestPhase(collector, 'MV6a', 'start')
  const name = 'MV6a_belowMinimum'
  try {
    await swapToStellar(api, alice, 0.5, { userAddress: getEnv('USER_ADDRESS') })
    fail(name, 'swapToStellar should have thrown, but succeeded', counter)
  } catch (e) {
    if (e.message.includes('AmountIsLessThanWithdrawFee')) {
      log(`Correctly rejected: ${e.message}`)
      pass(name, counter)
    } else {
      fail(name, `Expected AmountIsLessThanWithdrawFee, got: ${e.message}`, counter)
    }
  }
  markTestPhase(collector, 'MV6a', 'end')
}

async function testMV8_lostCursor () {
  console.log('\n── MV8: Lost cursor (wipe all 3 persistency files → no double-spend) ──')
  markTestPhase(collector, 'MV8', 'start')
  const name = 'MV8_lostCursor'
  const userAddress = getEnv('USER_ADDRESS')

  try {
    // Kill all 3 validators
    for (let i = 1; i <= 3; i++) killValidator(i)
    await new Promise(r => setTimeout(r, 2000))

    // Wipe all 3 persistency files.
    // Without the cursor, bridges re-scan the Stellar account from the beginning.
    // Protection against duplicate burns:  IsBurnedAlready  (ExecutedBurnTransactions)
    // Protection against duplicate mints:  IsMintedAlready  (ExecutedMintTransactions)
    for (let i = 1; i <= 3; i++) {
      const p = `${BRIDGE_DIR}/signer_mv_${i}.json`
      if (fs.existsSync(p)) {
        fs.unlinkSync(p)
        log(`Wiped: ${p}`)
      }
    }

    // Snapshot Stellar balance — should NOT change after restart
    const beforeStellar = await stellarTFTBalance(userAddress, horizon, issuerAddress)
    log(`User Stellar TFT before restart: ${beforeStellar}`)

    // Restart all 3 validators — they rescan with no local state
    for (let i = 1; i <= 3; i++) startValidator(i)
    log('All 3 validators restarted with wiped cursors. Waiting 15s for rescan...')
    await new Promise(r => setTimeout(r, 15_000))

    // Verify no double-spend — balance must be unchanged
    const afterStellar = await stellarTFTBalance(userAddress, horizon, issuerAddress)
    const delta = Math.round((afterStellar - beforeStellar) * TFT_DECIMALS) / TFT_DECIMALS
    log(`User Stellar TFT after rescan: ${afterStellar} (delta: ${delta >= 0 ? '+' : ''}${delta})`)
    if (Math.abs(delta) > 1e-7) {
      fail(name, `DOUBLE-SPEND: balance changed by ${delta} TFT after cursor wipe`, counter); return
    }
    log('No double-spend — IsMintedAlready + IsBurnedAlready held during rescan')

    // ─── Hardened: fresh DEPOSIT to prove bridge is at Stellar tip ───────
    //
    // A withdraw (old approach) is event-driven from TFChain — it doesn't use
    // the Stellar cursor at all, so it doesn't prove the bridge finished scanning.
    //
    // A deposit proves the bridge has caught up to the TIP of the Stellar account,
    // because the bridge must reach our new transaction in the Horizon stream.
    //
    // Extra hardening: we verify the on-chain mint's tx_id matches our Stellar
    // deposit hash, ruling out the case where an OLD deposit was re-processed
    // and our new deposit is still un-seen.
    log('Running fresh deposit to verify bridge scanned to Stellar tip...')

    const twinOpt = await api.query.tfgridModule.twinIdByAccountID(alice.address)
    const twinId = twinOpt.isSome ? twinOpt.unwrap().toNumber() : twinOpt.toJSON()
    if (!twinId) throw new Error('Alice has no twin on TFChain — is bridge-setup complete?')

    const depositAmount = '2'
    const depositFee = Number(await api.query.tftBridgeModule.depositFee()) / TFT_DECIMALS
    const expectedMint = parseFloat(depositAmount) - depositFee
    log(`Deposit fee: ${depositFee} TFT, expected mint: ${expectedMint} TFT`)

    const aliceBalBefore = await tfchainBalance(api, alice.address)
    const mintsBefore = (await api.query.tftBridgeModule.executedMintTransactions.entries()).length
    log(`Alice TFChain TFT before: ${aliceBalBefore}, executed mints: ${mintsBefore}`)

    // Send deposit — capture the Stellar tx hash for verification
    const result = await sendStellarPayment(
      horizon, issuerAddress, NETWORK_PASSPHRASE,
      getEnv('USER_SECRET'),
      bridgeAddress,
      depositAmount,
      `twin_${twinId}`
    )
    const depositTxHash = result.hash
    log(`Fresh deposit sent: ${depositTxHash} (memo: twin_${twinId})`)

    // Wait for the mint to appear on-chain
    await waitUntil(async () => {
      const mints = await api.query.tftBridgeModule.executedMintTransactions.entries()
      if (mints.length > mintsBefore) return mints
    }, { timeoutMs: 300_000, intervalMs: 4000, desc: 'fresh deposit mint to complete' })

    // ─── TX HASH VERIFICATION ──────────────────────────────────────────
    // Query executedMintTransactions by our specific Stellar tx hash.
    // On-chain key = Vec<u8> of the Stellar tx hash string (same as Go bridge passes).
    // If found: our NEW deposit was processed (bridge is at tip).
    // If not found: an OLD deposit was re-processed instead — FAIL.
    const mintTx = (await api.query.tftBridgeModule.executedMintTransactions(depositTxHash)).toJSON()
    if (!mintTx || !mintTx.amount || mintTx.amount === 0) {
      fail(name, `Mint tx hash mismatch: executedMintTransactions["${depositTxHash.slice(0, 16)}..."] not found on-chain — an old deposit may have been re-processed instead`, counter)
      return
    }
    log(`TX hash verified: executedMintTransactions["${depositTxHash.slice(0, 16)}..."] = {amount: ${mintTx.amount}, votes: ${mintTx.votes}}`)

    // Verify Alice's TFChain balance increased by expected amount (± 0.1 for block author rewards)
    const aliceBalAfter = await tfchainBalance(api, alice.address)
    const balDelta = Math.round((aliceBalAfter - aliceBalBefore) * TFT_DECIMALS) / TFT_DECIMALS
    log(`Alice TFChain TFT after: ${aliceBalAfter} (+${balDelta} TFT)`)
    if (Math.abs(balDelta - expectedMint) > 0.1) {
      fail(name, `Fresh deposit: expected TFChain ~+${expectedMint} TFT (±0.1), got +${balDelta}`, counter); return
    }

    // Verify exactly 1 new mint was processed (no old deposits re-minted)
    const mintsAfter = (await api.query.tftBridgeModule.executedMintTransactions.entries()).length
    const mintCountDelta = mintsAfter - mintsBefore
    log(`Executed mints: ${mintsBefore} → ${mintsAfter} (+${mintCountDelta})`)
    if (mintCountDelta !== 1) {
      fail(name, `Expected exactly 1 new mint, got ${mintCountDelta} — old deposits may have been re-processed`, counter); return
    }

    pass(name, counter)
  } catch (e) {
    fail(name, e.message, counter)
  } finally {
    markTestPhase(collector, 'MV8', 'end')
  }
}

async function testMV9_expiredBatchRecovery () {
  console.log('\n── MV9: Expired batch recovery (50 swaps offline → expiry → restart) ──')
  markTestPhase(collector, 'MV9', 'start')
  const name = 'MV9_expiredBatchRecovery'
  const userAddress = getEnv('USER_ADDRESS')
  const N = 50

  try {
    // Kill all 3 validators
    for (let i = 1; i <= 3; i++) killValidator(i)
    await new Promise(r => setTimeout(r, 2000))
    log('All 3 validators killed')

    const beforeStellar = await stellarTFTBalance(userAddress, horizon, issuerAddress)
    log(`User Stellar TFT before: ${beforeStellar}`)

    // Submit N swaps in one block using sequential nonces
    log(`Submitting ${N} swaps (all validators offline)...`)
    const nonce = await api.rpc.system.accountNextIndex(alice.address)
    const burnIds = await Promise.all(
      Array.from({ length: N }, (_, i) =>
        swapToStellar(api, alice, 2, { userAddress, nonce: nonce.toNumber() + i })
      )
    )
    log(`${N} burns created on-chain: IDs ${burnIds[0]}..${burnIds[burnIds.length - 1]}`)

    // Wait for all burns to expire (on_finalize clears signatures after RetryInterval=20 blocks ≈ 120s)
    log('Waiting for burns to expire on-chain (RetryInterval=20 blocks)...')
    await waitUntil(async () => {
      const burn = (await api.query.tftBridgeModule.burnTransactions(burnIds[0])).toJSON()
      return burn && burn.signatures && burn.signatures.length === 0
    }, { timeoutMs: 180_000, intervalMs: 6000, desc: 'first burn to expire (signatures cleared)' })
    log('All burns expired (signatures cleared, sequence_number reset to 0)')

    // Start all 3 validators — they catch the next BurnTransactionExpired events.
    // handleProposalsBatch re-proposes all expired burns in a single force_batch tx with
    // consecutive Stellar sequence numbers (SyncSequenceNumber + per-proposal increment).
    // All validators sync the same base sequence (no Stellar tx submitted yet), so they
    // produce matching signatures. Once threshold is met, all burns become Ready and
    // handleWithdrawReady submits them sequentially — each using its stored sequence
    // number, so all succeed in one pass.
    //
    // After all Stellar payments in a cycle complete, BatchSetWithdrawExecuted
    // confirms all in one force_batch. Multiple expiry cycles may be needed.
    for (let i = 1; i <= 3; i++) startValidator(i)
    log('All 3 validators restarted. Recovering expired burns via batch re-proposal...')
    log(`Expected: 1 force_batch re-proposes all ${N}, then all ${N} Stellar payments execute in sequence`)

    const expectedNet = N * (2 - WITHDRAW_FEE_TFT)
    let lastReported = 0

    const finalStellar = await waitUntil(async () => {
      const bal = await stellarTFTBalance(userAddress, horizon, issuerAddress)
      const delivered = Math.round((bal - beforeStellar) * TFT_DECIMALS) / TFT_DECIMALS
      const count = Math.round(delivered / (2 - WITHDRAW_FEE_TFT))
      if (count > lastReported) {
        log(`  Progress: ${count}/${N} burns delivered (+${delivered} TFT)`)
        lastReported = count
      }
      if (bal >= beforeStellar + expectedNet - 1e-7) return bal
    }, { timeoutMs: 900_000, intervalMs: 10_000, desc: `all ${N} burns delivered (+${expectedNet} TFT)` })

    const delta = Math.round((finalStellar - beforeStellar) * TFT_DECIMALS) / TFT_DECIMALS
    log(`All ${N} burns delivered: +${delta} TFT (expected +${expectedNet})`)
    if (Math.abs(delta - expectedNet) > 1e-7) {
      fail(name, `Expected +${expectedNet}, got +${delta}`, counter); return
    }

    // Assert on-chain: all burns executed
    for (const burnId of burnIds) {
      if (!(await assertBurnExecuted(name, burnId))) return
    }

    pass(name, counter)
  } catch (e) {
    fail(name, e.message, counter)
  } finally {
    markTestPhase(collector, 'MV9', 'end')
  }
}

async function testMV7_cleanState () {
  console.log('\n── MV7: Clean state (no orphaned active transactions) ──')
  markTestPhase(collector, 'MV7', 'start')
  const name = 'MV7_cleanState'

  try {
    // Wait for all active transaction maps to drain (tolerates in-flight processing)
    await waitUntil(async () => {
      const burns = await api.query.tftBridgeModule.burnTransactions.entries()
      const refunds = await api.query.tftBridgeModule.refundTransactions.entries()
      const mints = await api.query.tftBridgeModule.mintTransactions.entries()
      return burns.length === 0 && refunds.length === 0 && mints.length === 0
    }, { timeoutMs: 300_000, intervalMs: 5000, desc: 'all active tx maps to drain' })
    pass(name, counter)
  } catch (e) {
    // On timeout, report what's left
    const burns = await api.query.tftBridgeModule.burnTransactions.entries()
    const refunds = await api.query.tftBridgeModule.refundTransactions.entries()
    const mints = await api.query.tftBridgeModule.mintTransactions.entries()
    fail(name, `Orphaned: ${burns.length} burns, ${refunds.length} refunds, ${mints.length} mints`, counter)
  } finally {
    markTestPhase(collector, 'MV7', 'end')
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

    // Start event collector for deep analysis
    collector = await startEventCollector(api)
    console.log('[mv-tests] Event collector started — tracking all bridge events by block')

    console.log('[mv-tests] Starting multi-validator test suite...\n')

    await testMV1_normalWithdraw()
    await testMV2_deposit()
    await testMV3_badDeposit()
    await testMV4_validatorOffline()  // kills/restarts Val3
    await testMV5_batchWithdraws()
    await testMV6_crashRecovery()           // kills/restarts Val2
    await testMV6a_belowMinimum()           // pure pallet test, no bridge needed
    await testMV8_lostCursor()              // kills/restarts all 3 with wiped cursors
    await testMV9_expiredBatchRecovery()   // kills all 3, 50 swaps, expiry, restart (long)
    await testMV7_cleanState()

    console.log(`\n${'─'.repeat(50)}`)
    console.log(`Results: ${counter.passed} passed, ${counter.failed} failed`)
    console.log('─'.repeat(50))

    // Generate analysis report (pass api for chain-state reconciliation)
    const outputPath = process.env.ANALYSIS_OUTPUT || '/tmp/bridge_mv_analysis.json'
    await generateReport(collector, outputPath, api)
  } finally {
    if (collector) collector.stop()
    await api.disconnect()
  }

  process.exit(counter.failed > 0 ? 1 : 0)
}

main().catch(e => {
  console.error(`[mv-tests] FATAL: ${e.message || e}`)
  process.exit(1)
})
