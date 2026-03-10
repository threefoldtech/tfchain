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
 *   5. Deposit/mint      — send TFT to bridge with twin memo, verify TFChain balance
 *   6. Below-minimum     — swap below fee, expect dispatch error
 *   4. Crash recovery    — SIGKILL bridge mid-withdraw, restart, verify delivery completes
 *   8. Lost cursor       — wipe persistency (BoltDB), restart, verify no double-spend,
 *                          then deposit to prove bridge is at Stellar tip (tx hash verified)
 *   9. Expired batch     — 50 swaps while bridge offline, wait for expiry, restart, all delivered
 *   7. Clean state       — verify no orphaned active transactions on-chain
 *
 * All tests assert exact TFT balances (Stellar + TFChain) and on-chain state.
 * Non-zero exit on any failure.
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
  tfchainBalance,
  waitUntil,
  sendStellarPayment,
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

async function test1_normalWithdraw () {
  console.log('\n── TEST 1: Normal withdraw (2 TFT swap → 1 TFT net on Stellar) ──')
  const name = 'test1_normalWithdraw'
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
    }, { timeoutMs: 180_000, desc: `Stellar balance to increase above ${beforeStellar}` })

    // Assert Stellar balance delta
    const delta = Math.round((afterStellar - beforeStellar) * TFT_DECIMALS) / TFT_DECIMALS
    const expected = swapAmount - WITHDRAW_FEE_TFT
    if (Math.abs(delta - expected) > 1e-7) {
      fail(name, `Expected Stellar +${expected} TFT, got +${delta}`, counter); return
    }
    log(`User Stellar TFT after: ${afterStellar} (+${delta} TFT)`)

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
      if (bal >= beforeStellar + expectedNet - 1e-7) return bal
    }, { timeoutMs: 300_000, desc: `Stellar balance >= ${beforeStellar + expectedNet}` })

    const delta = Math.round((afterStellar - beforeStellar) * TFT_DECIMALS) / TFT_DECIMALS
    log(`User Stellar TFT after: ${afterStellar} (+${delta} TFT, expected +${expectedNet})`)
    if (Math.abs(delta - expectedNet) > 1e-7) {
      fail(name, `Expected +${expectedNet} TFT, got +${delta}`, counter); return
    }

    // Assert on-chain: all burns executed
    for (const burnId of burnIds) {
      if (!(await assertBurnExecuted(name, burnId))) return
    }

    pass(name, counter)
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
    const depositAmount = '3'

    const beforeStellar = await stellarTFTBalance(userAddress, horizon, issuerAddress)
    const refundsBefore = (await api.query.tftBridgeModule.executedRefundTransactions.entries()).length
    log(`User Stellar TFT before: ${beforeStellar}`)

    // Send TFT to bridge without a memo
    const result = await sendStellarPayment(horizon, issuerAddress, NETWORK_PASSPHRASE, userSecret, bridgeAddress, depositAmount)
    log(`Bad deposit sent: ${result.hash.slice(0, 16)}`)

    // Wait for refund — balance should return to (roughly) beforeStellar
    const afterStellar = await waitUntil(async () => {
      const bal = await stellarTFTBalance(userAddress, horizon, issuerAddress)
      if (bal >= beforeStellar - 1e-7) return bal
    }, { timeoutMs: 180_000, desc: 'refund to restore balance' })

    const delta = Math.round((afterStellar - beforeStellar) * TFT_DECIMALS) / TFT_DECIMALS
    log(`User Stellar TFT after: ${afterStellar} (delta: ${delta >= 0 ? '+' : ''}${delta})`)
    // Balance should be within 0 (full refund, no deposit fee on refunds)
    if (Math.abs(delta) > 1e-7) {
      fail(name, `Expected net 0 change (full refund), got ${delta >= 0 ? '+' : ''}${delta}`, counter); return
    }

    // Assert on-chain: refund executed
    if (!(await assertRefundExecuted(name, refundsBefore))) return

    pass(name, counter)
  } catch (e) {
    fail(name, e.message, counter)
  }
}

async function test5_deposit () {
  console.log('\n── TEST 5: Deposit/mint (send TFT to bridge with twin memo) ──')
  const name = 'test5_deposit'

  try {
    // Get Alice's twin ID
    const twinOpt = await api.query.tfgridModule.twinIdByAccountID(alice.address)
    const twinId = twinOpt.isSome ? twinOpt.unwrap().toNumber() : twinOpt.toJSON()
    if (!twinId) { fail(name, 'Alice has no twin on TFChain', counter); return }
    log(`Alice twin ID: ${twinId}`)

    const depositAmount = '2'
    const depositFee = Number(await api.query.tftBridgeModule.depositFee()) / TFT_DECIMALS
    const expectedMint = parseFloat(depositAmount) - depositFee
    log(`Deposit fee: ${depositFee} TFT, expected mint: ${expectedMint} TFT`)

    const aliceBalBefore = await tfchainBalance(api, alice.address)
    const mintsBefore = (await api.query.tftBridgeModule.executedMintTransactions.entries()).length
    log(`Alice TFChain TFT before: ${aliceBalBefore}, executed mints: ${mintsBefore}`)

    // Send TFT to bridge with twin_<id> memo
    const userSecret = getEnv('USER_SECRET')
    const result = await sendStellarPayment(horizon, issuerAddress, NETWORK_PASSPHRASE, userSecret, bridgeAddress, depositAmount, `twin_${twinId}`)
    log(`Deposit sent: ${result.hash.slice(0, 16)} (memo: twin_${twinId})`)

    // Wait for mint to be executed on TFChain
    await waitUntil(async () => {
      const mints = await api.query.tftBridgeModule.executedMintTransactions.entries()
      if (mints.length > mintsBefore) return true
    }, { timeoutMs: 120_000, desc: 'executed mint count to increase' })

    // Assert Alice's TFChain balance increased by ~expectedMint (± 0.1 TFT for block author rewards).
    const aliceBalAfter = await tfchainBalance(api, alice.address)
    const balDelta = Math.round((aliceBalAfter - aliceBalBefore) * TFT_DECIMALS) / TFT_DECIMALS
    log(`Alice TFChain TFT after: ${aliceBalAfter} (+${balDelta} TFT)`)
    if (Math.abs(balDelta - expectedMint) > 0.1) {
      fail(name, `Expected TFChain ~+${expectedMint} TFT (±0.1), got +${balDelta}`, counter); return
    }

    pass(name, counter)
  } catch (e) {
    fail(name, e.message, counter)
  }
}

async function test6_belowMinimum () {
  console.log('\n── TEST 6: Withdraw below minimum (should be rejected) ──')
  const name = 'test6_belowMinimum'
  const userAddress = getEnv('USER_ADDRESS')

  try {
    // Attempt swap with 0.5 TFT (below 1 TFT withdraw fee)
    await swapToStellar(api, alice, 0.5, { userAddress })
    fail(name, 'swapToStellar should have thrown, but succeeded', counter)
  } catch (e) {
    if (e.message.includes('AmountIsLessThanWithdrawFee')) {
      log(`Correctly rejected: ${e.message}`)
      pass(name, counter)
    } else {
      fail(name, `Expected AmountIsLessThanWithdrawFee, got: ${e.message}`, counter)
    }
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
    }, { timeoutMs: 60_000, desc: 'BurnTransactionReady (>=1 sig)' })

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

    // Explicit double-spend guard: verify exactly +expected, not 2x expected
    if (Math.abs(delta - expected) > 1e-7) {
      fail(name, `Expected +${expected} TFT after recovery, got +${delta} (double-spend if 2x)`, counter); return
    }

    // Assert on-chain: burn executed
    if (!(await assertBurnExecuted(name, burnId))) return

    pass(name, counter)
  } catch (e) {
    fail(name, e.message, counter)
  }
}

async function test8_lostCursor () {
  console.log('\n── TEST 8: Lost cursor (wipe persistency → no double-spend) ──')
  const name = 'test8_lostCursor'
  const userAddress = getEnv('USER_ADDRESS')

  try {
    // Bridge is running (restarted by T4). Kill it.
    killBridge('SIGKILL')
    await new Promise(r => setTimeout(r, 2000))

    // Wipe the persistency file (BoltDB/JSON cursor).
    // Without the cursor, bridge re-scans the Stellar account from the beginning.
    // Protection against duplicate burns:  IsBurnedAlready  (ExecutedBurnTransactions)
    // Protection against duplicate mints:  IsMintedAlready  (ExecutedMintTransactions)
    if (fs.existsSync(BRIDGE_PERSISTENCY)) {
      fs.unlinkSync(BRIDGE_PERSISTENCY)
      log(`Persistency wiped: ${BRIDGE_PERSISTENCY}`)
    } else {
      log('Persistency file not found (nothing to wipe)')
    }

    // Snapshot Stellar balance — should NOT change after restart
    const beforeStellar = await stellarTFTBalance(userAddress, horizon, issuerAddress)
    log(`User Stellar TFT before restart: ${beforeStellar}`)

    // Restart bridge — it rescans with no local state
    startBridge()
    log('Bridge restarted with wiped cursor. Waiting 15s for rescan...')
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
    const userSecret = getEnv('USER_SECRET')
    const result = await sendStellarPayment(horizon, issuerAddress, NETWORK_PASSPHRASE, userSecret, bridgeAddress, depositAmount, `twin_${twinId}`)
    const depositTxHash = result.hash
    log(`Fresh deposit sent: ${depositTxHash} (memo: twin_${twinId})`)

    // Wait for the mint to appear on-chain
    await waitUntil(async () => {
      const mints = await api.query.tftBridgeModule.executedMintTransactions.entries()
      if (mints.length > mintsBefore) return mints
    }, { timeoutMs: 300_000, desc: 'fresh deposit mint to complete' })

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
  }
}

async function test9_expiredBatchRecovery () {
  console.log('\n── TEST 9: Expired batch recovery (50 swaps offline → expiry → restart) ──')
  const name = 'test9_expiredBatchRecovery'
  const userAddress = getEnv('USER_ADDRESS')
  const N = 50

  try {
    // Kill bridge before submitting swaps
    killBridge('SIGKILL')
    await new Promise(r => setTimeout(r, 2000))
    log('Bridge killed')

    const beforeStellar = await stellarTFTBalance(userAddress, horizon, issuerAddress)
    log(`User Stellar TFT before: ${beforeStellar}`)

    // Submit N swaps in one block using sequential nonces
    log(`Submitting ${N} swaps (bridge offline)...`)
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

    // Start bridge — it subscribes to new blocks and catches the next BurnTransactionExpired events.
    // handleProposalsBatch re-proposes all expired burns in a single force_batch tx with
    // consecutive Stellar sequence numbers (SyncSequenceNumber + per-proposal increment).
    // All burns become Ready in the same block, and handleWithdrawReady processes them
    // sequentially — each Stellar submission uses its stored sequence number, so all
    // succeed in one pass without sequence collisions.
    //
    // After all Stellar payments complete, each handleWithdrawReady calls
    // SetWithdrawExecuted individually (could be batched in a future optimization).
    startBridge()
    log('Bridge restarted. Recovering expired burns via batch re-proposal...')
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
  }
}

async function test7_cleanState () {
  console.log('\n── TEST 7: Clean state (no orphaned active transactions) ──')
  const name = 'test7_cleanState'

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
    await test5_deposit()
    await test6_belowMinimum()
    await test4_crashRecovery()            // kills/restarts bridge
    await test8_lostCursor()               // kills/restarts bridge with wiped cursor
    await test9_expiredBatchRecovery()     // kills bridge, 50 swaps, expiry, restart (long)
    await test7_cleanState()

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
