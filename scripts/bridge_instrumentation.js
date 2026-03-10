#!/usr/bin/env node
/**
 * bridge_instrumentation.js
 *
 * Event collector and analysis engine for bridge E2E tests.
 * Subscribes to every finalized block, captures all tftBridgeModule and utility
 * events with block numbers and wall-clock timestamps, then generates a
 * structured report mapping each transaction's full lifecycle.
 *
 * Usage (from bridge_mv_tests.js):
 *   const { startEventCollector, markTestPhase, generateReport } = require('./bridge_instrumentation')
 *   const collector = await startEventCollector(api)
 *   markTestPhase(collector, 'MV1', 'start')
 *   // ... test body ...
 *   markTestPhase(collector, 'MV1', 'end')
 *   await generateReport(collector, '/tmp/bridge_mv_analysis.json', api)
 *   collector.stop()
 */

'use strict'

const fs = require('fs')

// ─── Event Collector ────────────────────────────────────────────────────────

/**
 * Start a background event collector that captures all bridge-related events
 * from every finalized block.
 *
 * @param {object} api - Connected ApiPromise instance
 * @returns {object} collector with events, indexes, and stop()
 */
async function startEventCollector (api) {
  const collector = {
    events: [],
    blockTimestamps: {},
    burnEvents: {},
    refundEvents: {},
    mintEvents: {},
    utilityEvents: [],
    blockEventLog: {},
    testPhases: [],
    _unsub: null,
    _startTime: Date.now(),
    _firstBlock: null,
    _lastBlock: null,

    stop () {
      if (this._unsub) {
        this._unsub()
        this._unsub = null
      }
    }
  }

  collector._unsub = await api.rpc.chain.subscribeFinalizedHeads(async (header) => {
    const blockNum = header.number.toNumber()
    const blockHash = header.hash
    collector.blockTimestamps[blockNum] = Date.now()
    if (collector._firstBlock === null) collector._firstBlock = blockNum
    collector._lastBlock = blockNum

    try {
      const apiAt = await api.at(blockHash)
      const records = await apiAt.query.system.events()

      records.forEach((record, idx) => {
        const { event, phase } = record
        const section = event.section
        const method = event.method

        // Only capture bridge and utility events
        if (section !== 'tftBridgeModule' && section !== 'utility') return

        const entry = {
          block: blockNum,
          timestamp: collector.blockTimestamps[blockNum],
          index: idx,
          section,
          method,
          data: event.data.map(d => d.toString()),
          phase: phase.toString()
        }

        collector.events.push(entry)

        // Index by block
        if (!collector.blockEventLog[blockNum]) collector.blockEventLog[blockNum] = []
        collector.blockEventLog[blockNum].push(entry)

        // Index bridge events by transaction ID
        if (section === 'tftBridgeModule') {
          indexBridgeEvent(collector, entry)
        }
        if (section === 'utility') {
          collector.utilityEvents.push(entry)
        }
      })
    } catch (err) {
      // Silently ignore block fetch errors — don't disrupt the tests
    }
  })

  return collector
}

/**
 * Index a bridge event into the collector's transaction maps.
 */
function indexBridgeEvent (collector, entry) {
  const { method, data } = entry

  switch (method) {
    // ── Burn lifecycle ──
    case 'BurnTransactionCreated': {
      const burnId = data[0]
      if (!collector.burnEvents[burnId]) collector.burnEvents[burnId] = []
      collector.burnEvents[burnId].push({ ...entry, state: 'created' })
      break
    }
    case 'BurnTransactionProposed': {
      const burnId = data[0]
      if (!collector.burnEvents[burnId]) collector.burnEvents[burnId] = []
      collector.burnEvents[burnId].push({ ...entry, state: 'proposed' })
      break
    }
    case 'BurnTransactionSignatureAdded': {
      const burnId = data[0]
      if (!collector.burnEvents[burnId]) collector.burnEvents[burnId] = []
      collector.burnEvents[burnId].push({ ...entry, state: 'sig_added' })
      break
    }
    case 'BurnTransactionReady': {
      const burnId = data[0]
      if (!collector.burnEvents[burnId]) collector.burnEvents[burnId] = []
      collector.burnEvents[burnId].push({ ...entry, state: 'ready' })
      break
    }
    case 'BurnTransactionProcessed': {
      // BurnTransactionProcessed carries the full BurnTransaction struct, not the burn_id.
      // Store globally for post-collection reconciliation.
      if (!collector.burnEvents._processed) collector.burnEvents._processed = []
      collector.burnEvents._processed.push({ ...entry, state: 'processed' })
      break
    }
    case 'BurnTransactionExpired': {
      const burnId = data[0]
      if (!collector.burnEvents[burnId]) collector.burnEvents[burnId] = []
      collector.burnEvents[burnId].push({ ...entry, state: 'expired' })
      break
    }

    // ── Refund lifecycle ──
    case 'RefundTransactionCreated': {
      const txHash = data[0]
      if (!collector.refundEvents[txHash]) collector.refundEvents[txHash] = []
      collector.refundEvents[txHash].push({ ...entry, state: 'created' })
      break
    }
    case 'RefundTransactionsignatureAdded': {
      // Note: lowercase 's' — that's a typo in the pallet, not here.
      const txHash = data[0]
      if (!collector.refundEvents[txHash]) collector.refundEvents[txHash] = []
      collector.refundEvents[txHash].push({ ...entry, state: 'sig_added' })
      break
    }
    case 'RefundTransactionReady': {
      const txHash = data[0]
      if (!collector.refundEvents[txHash]) collector.refundEvents[txHash] = []
      collector.refundEvents[txHash].push({ ...entry, state: 'ready' })
      break
    }
    case 'RefundTransactionProcessed': {
      if (!collector.refundEvents._processed) collector.refundEvents._processed = []
      collector.refundEvents._processed.push({ ...entry, state: 'processed' })
      break
    }
    case 'RefundTransactionExpired': {
      const txHash = data[0]
      if (!collector.refundEvents[txHash]) collector.refundEvents[txHash] = []
      collector.refundEvents[txHash].push({ ...entry, state: 'expired' })
      break
    }

    // ── Mint lifecycle ──
    case 'MintTransactionProposed': {
      const txId = data[0]
      if (!collector.mintEvents[txId]) collector.mintEvents[txId] = []
      collector.mintEvents[txId].push({ ...entry, state: 'proposed' })
      break
    }
    case 'MintTransactionVoted': {
      const txId = data[0]
      if (!collector.mintEvents[txId]) collector.mintEvents[txId] = []
      collector.mintEvents[txId].push({ ...entry, state: 'voted' })
      break
    }
    case 'MintCompleted': {
      // MintCompleted data: (MintTransaction, tx_id)
      const txId = data[1]
      if (!collector.mintEvents[txId]) collector.mintEvents[txId] = []
      collector.mintEvents[txId].push({ ...entry, state: 'completed' })
      break
    }
    case 'MintTransactionExpired': {
      const txId = data[0]
      if (!collector.mintEvents[txId]) collector.mintEvents[txId] = []
      collector.mintEvents[txId].push({ ...entry, state: 'expired' })
      break
    }
  }
}

// ─── Test Phase Markers ─────────────────────────────────────────────────────

/**
 * Mark the start or end of a test phase for event correlation.
 * Uses block numbers (not timestamps) to avoid async lag between
 * finalized head subscription and test code execution.
 */
function markTestPhase (collector, testName, phase) {
  collector.testPhases.push({
    test: testName,
    phase,
    timestamp: Date.now(),
    block: collector._lastBlock || 0
  })
}

// ─── Transaction Lifecycle Analysis ─────────────────────────────────────────

function analyzeBurn (collector, burnId) {
  const events = collector.burnEvents[burnId] || []
  if (events.length === 0) return null

  const record = {
    burnId: Number(burnId),
    type: 'burn',
    createdBlock: null,
    firstSigBlock: null,
    readyBlock: null,
    processedBlock: null,
    expiryCount: 0,
    expiryBlocks: [],
    sigCount: 0,
    blocksCreatedToReady: null,
    blocksReadyToProcessed: null,
    blocksCreatedToProcessed: null,
    wallClockMs: null,
    transitions: []
  }

  for (const evt of events) {
    record.transitions.push({
      state: evt.state,
      block: evt.block,
      timestamp: evt.timestamp
    })

    switch (evt.state) {
      case 'created':
        record.createdBlock = evt.block
        break
      case 'sig_added':
        record.sigCount++
        if (!record.firstSigBlock) record.firstSigBlock = evt.block
        break
      case 'ready':
        record.readyBlock = evt.block
        break
      case 'processed':
        record.processedBlock = evt.block
        break
      case 'expired':
        record.expiryCount++
        record.expiryBlocks.push(evt.block)
        break
    }
  }

  // Compute deltas
  if (record.createdBlock != null && record.readyBlock != null) {
    record.blocksCreatedToReady = record.readyBlock - record.createdBlock
  }
  if (record.readyBlock != null && record.processedBlock != null) {
    record.blocksReadyToProcessed = record.processedBlock - record.readyBlock
  }
  if (record.createdBlock != null && record.processedBlock != null) {
    record.blocksCreatedToProcessed = record.processedBlock - record.createdBlock
  }

  // Wall-clock time
  if (events.length >= 2) {
    record.wallClockMs = events[events.length - 1].timestamp - events[0].timestamp
  }

  return record
}

function analyzeRefund (collector, txHash) {
  const events = collector.refundEvents[txHash] || []
  if (events.length === 0) return null

  const record = {
    txHash,
    type: 'refund',
    createdBlock: null,
    readyBlock: null,
    processedBlock: null,
    expiryCount: 0,
    sigCount: 0,
    blocksCreatedToReady: null,
    blocksCreatedToProcessed: null,
    wallClockMs: null,
    transitions: []
  }

  for (const evt of events) {
    record.transitions.push({ state: evt.state, block: evt.block, timestamp: evt.timestamp })
    switch (evt.state) {
      case 'created': record.createdBlock = evt.block; break
      case 'sig_added': record.sigCount++; break
      case 'ready': record.readyBlock = evt.block; break
      case 'processed': record.processedBlock = evt.block; break
      case 'expired': record.expiryCount++; break
    }
  }

  if (!record.processedBlock && collector.refundEvents._processed) {
    for (const pe of collector.refundEvents._processed) {
      if (record.readyBlock && pe.block >= record.readyBlock && !record.processedBlock) {
        record.processedBlock = pe.block
      }
    }
  }

  if (record.createdBlock != null && record.readyBlock != null) {
    record.blocksCreatedToReady = record.readyBlock - record.createdBlock
  }
  if (record.createdBlock != null && record.processedBlock != null) {
    record.blocksCreatedToProcessed = record.processedBlock - record.createdBlock
  }
  if (events.length >= 2) {
    record.wallClockMs = events[events.length - 1].timestamp - events[0].timestamp
  }

  return record
}

function analyzeMint (collector, txId) {
  const events = collector.mintEvents[txId] || []
  if (events.length === 0) return null

  const record = {
    txId,
    type: 'mint',
    proposedBlock: null,
    completedBlock: null,
    voteCount: 0,
    expiryCount: 0,
    blocksProposedToCompleted: null,
    wallClockMs: null,
    transitions: []
  }

  for (const evt of events) {
    record.transitions.push({ state: evt.state, block: evt.block, timestamp: evt.timestamp })
    switch (evt.state) {
      case 'proposed': record.proposedBlock = evt.block; break
      case 'voted': record.voteCount++; break
      case 'completed': record.completedBlock = evt.block; break
      case 'expired': record.expiryCount++; break
    }
  }

  if (record.proposedBlock != null && record.completedBlock != null) {
    record.blocksProposedToCompleted = record.completedBlock - record.proposedBlock
  }
  if (events.length >= 2) {
    record.wallClockMs = events[events.length - 1].timestamp - events[0].timestamp
  }

  return record
}

// ─── Test Phase Correlation ─────────────────────────────────────────────────

/**
 * Determine which test a block belongs to, using block-number ranges
 * from the test phase markers. This avoids the async lag issue where
 * finalized-head events arrive after the test code has already moved on.
 *
 * Phase markers record collector._lastBlock at the time markTestPhase is
 * called. A block B belongs to test T if:
 *   T.start.block <= B <= T.end.block  (or T has no end yet and B >= T.start.block)
 */
function getTestForBlock (collector, block) {
  // Build sorted phase marker pairs
  const tests = {}
  for (const marker of collector.testPhases) {
    if (!tests[marker.test]) tests[marker.test] = {}
    tests[marker.test][marker.phase] = marker.block
  }

  // Walk test order (use testPhases order of first appearance)
  const testOrder = []
  const seen = new Set()
  for (const marker of collector.testPhases) {
    if (marker.phase === 'start' && !seen.has(marker.test)) {
      seen.add(marker.test)
      testOrder.push(marker.test)
    }
  }

  // Find the test whose block range contains this block
  for (let i = testOrder.length - 1; i >= 0; i--) {
    const test = testOrder[i]
    const startBlock = tests[test].start
    const endBlock = tests[test].end

    if (startBlock == null) continue

    if (endBlock != null) {
      // Completed test: block must be in range [startBlock, endBlock]
      if (block >= startBlock && block <= endBlock) return test
    } else {
      // Still running: block must be >= startBlock
      if (block >= startBlock) return test
    }
  }

  // Blocks before any test started — assign to first test if close
  if (testOrder.length > 0) {
    const firstStart = tests[testOrder[0]].start
    if (block < firstStart) return 'pre-test'
  }

  return 'unknown'
}

function groupEventsByTest (collector) {
  const testGroups = {}

  // Group burn events by test phase (using block of first event)
  for (const [burnId, events] of Object.entries(collector.burnEvents)) {
    if (burnId === '_processed') continue
    if (events.length === 0) continue
    const test = getTestForBlock(collector, events[0].block)
    if (!testGroups[test]) testGroups[test] = { burns: [], refunds: [], mints: [] }
    testGroups[test].burns.push(burnId)
  }

  // Group refund events by test phase
  for (const [txHash, events] of Object.entries(collector.refundEvents)) {
    if (txHash === '_processed') continue
    if (events.length === 0) continue
    const test = getTestForBlock(collector, events[0].block)
    if (!testGroups[test]) testGroups[test] = { burns: [], refunds: [], mints: [] }
    testGroups[test].refunds.push(txHash)
  }

  // Group mint events by test phase
  for (const [txId, events] of Object.entries(collector.mintEvents)) {
    if (events.length === 0) continue
    const test = getTestForBlock(collector, events[0].block)
    if (!testGroups[test]) testGroups[test] = { burns: [], refunds: [], mints: [] }
    testGroups[test].mints.push(txId)
  }

  return testGroups
}

// ─── Batch Analysis ─────────────────────────────────────────────────────────

/**
 * Analyze how set_burn_transaction_executed / set_refund_transaction_executed
 * calls are batched. For each block, count how many Processed events arrived
 * in a single batch (paired with ItemCompleted inside a BatchCompleted or
 * BatchCompletedWithErrors).
 *
 * Returns an array of batch records:
 *   { block, type, processedCount, failedCount, batchType }
 */
function analyzeBatches (collector) {
  const batches = []
  const blockNums = Object.keys(collector.blockEventLog).map(Number).sort((a, b) => a - b)

  for (const blockNum of blockNums) {
    const events = collector.blockEventLog[blockNum]

    // Walk through events and group by batch boundaries
    let currentBatch = null

    for (const evt of events) {
      if (evt.section === 'tftBridgeModule' &&
          (evt.method === 'BurnTransactionProcessed' || evt.method === 'RefundTransactionProcessed')) {
        if (!currentBatch) {
          currentBatch = {
            block: blockNum,
            type: evt.method === 'BurnTransactionProcessed' ? 'burn_executed' : 'refund_executed',
            processedCount: 0,
            failedCount: 0,
            batchType: 'standalone'
          }
        }
        currentBatch.processedCount++
      }
      if (evt.section === 'utility' && evt.method === 'ItemFailed' && currentBatch) {
        currentBatch.failedCount++
      }
      if (evt.section === 'utility' && evt.method === 'BatchCompleted' && currentBatch) {
        currentBatch.batchType = 'BatchCompleted'
        batches.push(currentBatch)
        currentBatch = null
      }
      if (evt.section === 'utility' && evt.method === 'BatchCompletedWithErrors' && currentBatch) {
        currentBatch.batchType = 'BatchCompletedWithErrors'
        batches.push(currentBatch)
        currentBatch = null
      }
    }

    // If we have a dangling batch (Processed events without a Batch* wrapper)
    if (currentBatch && currentBatch.processedCount > 0) {
      batches.push(currentBatch)
    }
  }

  return batches
}

/**
 * Analyze signature proposal batches (propose_stellar_burn_transaction_or_add_sig).
 * For each block, count how many BurnTransactionSignatureAdded events arrived
 * inside a single force_batch.
 */
function analyzeSignatureBatches (collector) {
  const batches = []
  const blockNums = Object.keys(collector.blockEventLog).map(Number).sort((a, b) => a - b)

  for (const blockNum of blockNums) {
    const events = collector.blockEventLog[blockNum]
    let currentBatch = null

    for (const evt of events) {
      if (evt.section === 'tftBridgeModule' && evt.method === 'BurnTransactionSignatureAdded') {
        if (!currentBatch) {
          currentBatch = {
            block: blockNum,
            type: 'sig_proposal',
            sigCount: 0,
            readyCount: 0,
            failedCount: 0,
            batchType: 'standalone'
          }
        }
        currentBatch.sigCount++
      }
      if (evt.section === 'tftBridgeModule' && evt.method === 'BurnTransactionReady' && currentBatch) {
        currentBatch.readyCount++
      }
      if (evt.section === 'utility' && evt.method === 'ItemFailed' && currentBatch) {
        currentBatch.failedCount++
      }
      if (evt.section === 'utility' && evt.method === 'BatchCompleted' && currentBatch) {
        currentBatch.batchType = 'BatchCompleted'
        batches.push(currentBatch)
        currentBatch = null
      }
      if (evt.section === 'utility' && evt.method === 'BatchCompletedWithErrors' && currentBatch) {
        currentBatch.batchType = 'BatchCompletedWithErrors'
        batches.push(currentBatch)
        currentBatch = null
      }
    }

    if (currentBatch && currentBatch.sigCount > 0) {
      batches.push(currentBatch)
    }
  }

  return batches
}

// ─── Statistics Helpers ─────────────────────────────────────────────────────

function mean (arr) { return arr.length ? arr.reduce((a, b) => a + b, 0) / arr.length : 0 }
function median (arr) {
  if (!arr.length) return 0
  const s = [...arr].sort((a, b) => a - b)
  const mid = Math.floor(s.length / 2)
  return s.length % 2 ? s[mid] : (s[mid - 1] + s[mid]) / 2
}
function percentile (arr, p) {
  if (!arr.length) return 0
  const s = [...arr].sort((a, b) => a - b)
  const idx = Math.ceil(p / 100 * s.length) - 1
  return s[Math.max(0, idx)]
}

// ─── Anomaly Detection ──────────────────────────────────────────────────────

function detectAnomalies (collector, testGroups) {
  const anomalies = []

  // Tests that should have zero expiries
  const noExpiryTests = ['MV1', 'MV2', 'MV3', 'MV4', 'MV6']
  for (const test of noExpiryTests) {
    const group = testGroups[test]
    if (!group) continue

    for (const burnId of group.burns) {
      const rec = analyzeBurn(collector, burnId)
      if (rec && rec.expiryCount > 0) {
        anomalies.push({
          severity: 'warning',
          test,
          message: `Burn ${burnId} expired ${rec.expiryCount} time(s) in ${test} (expected 0)`,
          burnId
        })
      }
    }
    for (const txHash of group.refunds) {
      const rec = analyzeRefund(collector, txHash)
      if (rec && rec.expiryCount > 0) {
        anomalies.push({
          severity: 'warning',
          test,
          message: `Refund ${txHash.slice(0, 16)}... expired ${rec.expiryCount} time(s) in ${test} (expected 0)`,
          txHash
        })
      }
    }
  }

  // MV4 should have exactly 2 sigs per refund (Val3 offline)
  if (testGroups.MV4) {
    for (const txHash of testGroups.MV4.refunds) {
      const rec = analyzeRefund(collector, txHash)
      if (rec && rec.sigCount > 2) {
        anomalies.push({
          severity: 'warning',
          test: 'MV4',
          message: `Refund got ${rec.sigCount} sigs (expected 2, Val3 should be offline)`,
          txHash
        })
      }
    }
  }

  // MV9: check all 50 burns reached processed state
  if (testGroups.MV9) {
    const unprocessed = []
    for (const burnId of testGroups.MV9.burns) {
      const rec = analyzeBurn(collector, burnId)
      if (rec && rec.processedBlock == null) {
        unprocessed.push(burnId)
      }
    }
    if (unprocessed.length > 0) {
      anomalies.push({
        severity: 'error',
        test: 'MV9',
        message: `${unprocessed.length} burns never reached Processed state (event missed by collector)`,
        burnIds: unprocessed
      })
    }
  }

  // Utility ItemFailed events
  const itemFailedCount = collector.utilityEvents.filter(e => e.method === 'ItemFailed').length
  if (itemFailedCount > 0) {
    anomalies.push({
      severity: 'info',
      test: 'global',
      message: `${itemFailedCount} Utility.ItemFailed events total (expected in multi-validator races)`
    })
  }

  return anomalies
}

// ─── Chain-State Reconciliation ─────────────────────────────────────────────

/**
 * Query the on-chain ExecutedBurnTransactions and ExecutedRefundTransactions
 * maps to reconcile any burns/refunds whose Processed events were missed
 * by the finalized-head subscription (e.g. late-arriving blocks).
 *
 * Also waits for all active transaction maps to drain (like MV7), so we
 * know all transactions have been fully processed before generating the report.
 */
async function reconcileFromChain (collector, api) {
  // Wait for active burn/refund maps to drain (up to 60s — same signal as MV7)
  const started = Date.now()
  while (Date.now() - started < 60_000) {
    const burns = await api.query.tftBridgeModule.burnTransactions.entries()
    const refunds = await api.query.tftBridgeModule.refundTransactions.entries()
    if (burns.length === 0 && refunds.length === 0) break
    await new Promise(r => setTimeout(r, 3000))
  }

  // Wait a bit more for any in-flight finalized head events
  await new Promise(r => setTimeout(r, 5000))

  let reconciled = 0

  // Reconcile burns
  for (const [burnId, events] of Object.entries(collector.burnEvents)) {
    if (burnId === '_processed') continue
    const hasProcessed = events.some(e => e.state === 'processed')
    if (hasProcessed) continue

    try {
      const executed = (await api.query.tftBridgeModule.executedBurnTransactions(Number(burnId))).toJSON()
      if (executed && executed.target) {
        events.push({
          block: collector._lastBlock,
          timestamp: Date.now(),
          index: -1,
          section: 'tftBridgeModule',
          method: 'BurnTransactionProcessed',
          data: ['reconciled-from-chain-state'],
          phase: 'reconciled',
          state: 'processed',
          _reconciled: true
        })
        reconciled++
      }
    } catch {}
  }

  // Reconcile refunds
  for (const [txHash, events] of Object.entries(collector.refundEvents)) {
    if (txHash === '_processed') continue
    const hasProcessed = events.some(e => e.state === 'processed')
    if (hasProcessed) continue

    try {
      const entries = await api.query.tftBridgeModule.executedRefundTransactions.entries()
      const found = entries.some(([key]) => key.args[0].toString() === txHash)
      if (found) {
        events.push({
          block: collector._lastBlock,
          timestamp: Date.now(),
          index: -1,
          section: 'tftBridgeModule',
          method: 'RefundTransactionProcessed',
          data: ['reconciled-from-chain-state'],
          phase: 'reconciled',
          state: 'processed',
          _reconciled: true
        })
        reconciled++
      }
    } catch {}
  }

  return reconciled
}

// ─── Report Generator ───────────────────────────────────────────────────────

/**
 * @param {object} collector - The event collector
 * @param {string} outputPath - Path to write the JSON report
 * @param {object} [api] - Optional ApiPromise for chain-state reconciliation
 */
async function generateReport (collector, outputPath, api) {
  // Reconcile missing Processed events from chain state
  let reconciledCount = 0
  if (api) {
    reconciledCount = await reconcileFromChain(collector, api)
    if (reconciledCount > 0) {
      console.log(`[instrumentation] Reconciled ${reconciledCount} Processed events from chain state`)
    }
  } else {
    // Fallback: just wait a bit for in-flight events
    await new Promise(r => setTimeout(r, 3000))
  }

  const testGroups = groupEventsByTest(collector)
  const anomalies = detectAnomalies(collector, testGroups)

  // Build burn analysis
  const allBurnIds = Object.keys(collector.burnEvents).filter(k => k !== '_processed')
  const burnRecords = allBurnIds.map(id => analyzeBurn(collector, id)).filter(Boolean)

  // Build refund analysis
  const allRefundHashes = Object.keys(collector.refundEvents).filter(k => k !== '_processed')
  const refundRecords = allRefundHashes.map(h => analyzeRefund(collector, h)).filter(Boolean)

  // Build mint analysis
  const allMintIds = Object.keys(collector.mintEvents)
  const mintRecords = allMintIds.map(id => analyzeMint(collector, id)).filter(Boolean)

  // Batch analysis
  const executeBatches = analyzeBatches(collector)
  const sigBatches = analyzeSignatureBatches(collector)

  // KPIs
  const blocksToReady = burnRecords.map(b => b.blocksCreatedToReady).filter(v => v != null)
  const blocksToProcessed = burnRecords.map(b => b.blocksCreatedToProcessed).filter(v => v != null)
  const wallClockSecs = burnRecords.map(b => b.wallClockMs).filter(v => v != null).map(v => v / 1000)

  const kpis = {
    burns: {
      total: burnRecords.length,
      avgBlocksToReady: Math.round(mean(blocksToReady) * 10) / 10,
      medianBlocksToReady: median(blocksToReady),
      avgBlocksToProcessed: Math.round(mean(blocksToProcessed) * 10) / 10,
      medianBlocksToProcessed: median(blocksToProcessed),
      p95BlocksToProcessed: percentile(blocksToProcessed, 95),
      avgWallClockSecs: Math.round(mean(wallClockSecs) * 10) / 10,
      totalExpiries: burnRecords.reduce((s, b) => s + b.expiryCount, 0),
      burnsWithExpiry: burnRecords.filter(b => b.expiryCount > 0).length,
      maxExpiryCycles: Math.max(...burnRecords.map(b => b.expiryCount), 0),
      reconciledFromChain: reconciledCount
    },
    refunds: {
      total: refundRecords.length,
      totalExpiries: refundRecords.reduce((s, r) => s + r.expiryCount, 0)
    },
    mints: {
      total: mintRecords.length
    },
    utilityBatches: {
      batchCompleted: collector.utilityEvents.filter(e => e.method === 'BatchCompleted').length,
      batchCompletedWithErrors: collector.utilityEvents.filter(e => e.method === 'BatchCompletedWithErrors').length,
      itemFailed: collector.utilityEvents.filter(e => e.method === 'ItemFailed').length,
      itemCompleted: collector.utilityEvents.filter(e => e.method === 'ItemCompleted').length
    }
  }

  // Per-test summary
  const perTest = {}
  const testOrder = ['MV1', 'MV2', 'MV3', 'MV4', 'MV5', 'MV6', 'MV8', 'MV9', 'MV7']
  for (const test of testOrder) {
    const group = testGroups[test]
    if (!group) { perTest[test] = { burns: 0, refunds: 0, mints: 0, expiries: 0, blocks: 0, wallClockSecs: 0 }; continue }

    const burns = group.burns.map(id => analyzeBurn(collector, id)).filter(Boolean)
    const refunds = group.refunds.map(h => analyzeRefund(collector, h)).filter(Boolean)
    const mints = group.mints.map(id => analyzeMint(collector, id)).filter(Boolean)

    const allTransitions = [
      ...burns.flatMap(b => b.transitions),
      ...refunds.flatMap(r => r.transitions),
      ...mints.flatMap(m => m.transitions)
    ]
    const blocks = allTransitions.map(t => t.block).filter(Boolean)
    const blockSpan = blocks.length ? Math.max(...blocks) - Math.min(...blocks) : 0

    const timestamps = allTransitions.map(t => t.timestamp).filter(Boolean)
    const timeSpan = timestamps.length ? (Math.max(...timestamps) - Math.min(...timestamps)) / 1000 : 0

    perTest[test] = {
      burns: group.burns.length,
      refunds: group.refunds.length,
      mints: group.mints.length,
      expiries: burns.reduce((s, b) => s + b.expiryCount, 0) + refunds.reduce((s, r) => s + r.expiryCount, 0),
      blocks: blockSpan,
      wallClockSecs: Math.round(timeSpan)
    }
  }

  // Build timeline (block-by-block, only blocks with events)
  const timeline = []
  const blockNums = Object.keys(collector.blockEventLog).map(Number).sort((a, b) => a - b)
  for (const blockNum of blockNums) {
    const events = collector.blockEventLog[blockNum]
    timeline.push({
      block: blockNum,
      timestamp: collector.blockTimestamps[blockNum],
      events: events.map(e => ({
        section: e.section,
        method: e.method,
        data: e.data
      }))
    })
  }

  // Test phase markers (for debugging)
  const phaseMarkers = collector.testPhases.map(m => ({
    test: m.test,
    phase: m.phase,
    block: m.block,
    timestamp: m.timestamp
  }))

  // Full report object
  const report = {
    metadata: {
      generatedAt: new Date().toISOString(),
      retryInterval: 20,
      blockTimeSecs: 6,
      validatorCount: 3,
      threshold: 2,
      totalBlocks: collector._lastBlock - collector._firstBlock + 1,
      firstBlock: collector._firstBlock,
      lastBlock: collector._lastBlock,
      totalDurationSecs: Math.round((Date.now() - collector._startTime) / 1000),
      reconciledFromChain: reconciledCount
    },
    kpis,
    perTest,
    anomalies,
    phaseMarkers,
    batchAnalysis: {
      executesBatches: executeBatches,
      signatureBatches: sigBatches
    },
    transactions: {
      burns: burnRecords,
      refunds: refundRecords,
      mints: mintRecords
    },
    timeline
  }

  // Write JSON
  fs.writeFileSync(outputPath, JSON.stringify(report, null, 2))

  // Print human-readable summary
  printSummary(report)

  return report
}

// ─── Console Output ─────────────────────────────────────────────────────────

function printSummary (report) {
  const SEP = '─'.repeat(90)
  const DSEP = '═'.repeat(90)

  console.log(`\n${DSEP}`)
  console.log('  BRIDGE MV TEST ANALYSIS REPORT')
  console.log(DSEP)

  // Metadata
  const meta = report.metadata
  console.log(`  Duration: ${meta.totalDurationSecs}s | Blocks: ${meta.firstBlock}–${meta.lastBlock} (${meta.totalBlocks} blocks)`)
  if (meta.reconciledFromChain > 0) {
    console.log(`  ⚠ ${meta.reconciledFromChain} Processed events reconciled from chain state (missed by event subscription)`)
  }
  console.log(SEP)

  // Per-test table
  console.log('  Test   │ Burns │ Refunds │ Mints │ Expiries │ Block Span │ Time')
  console.log('  ' + '─'.repeat(78))
  const testOrder = ['MV1', 'MV2', 'MV3', 'MV4', 'MV5', 'MV6', 'MV8', 'MV9', 'MV7']
  for (const test of testOrder) {
    const t = report.perTest[test] || {}
    console.log(
      `  ${test.padEnd(6)} │ ${String(t.burns || 0).padStart(5)} │ ${String(t.refunds || 0).padStart(7)} │ ${String(t.mints || 0).padStart(5)} │ ${String(t.expiries || 0).padStart(8)} │ ${String(t.blocks || 0).padStart(10)} │ ${t.wallClockSecs || 0}s`
    )
  }

  console.log(SEP)

  // Burn KPIs
  const bk = report.kpis.burns
  console.log('  Burn KPIs:')
  console.log(`    Created→Ready:     avg=${bk.avgBlocksToReady} blocks, median=${bk.medianBlocksToReady}`)
  console.log(`    Created→Processed: avg=${bk.avgBlocksToProcessed} blocks, median=${bk.medianBlocksToProcessed}, p95=${bk.p95BlocksToProcessed}`)
  console.log(`    Wall-clock:        avg=${bk.avgWallClockSecs}s`)
  console.log(`    Expiries:          total=${bk.totalExpiries}, burns_with_expiry=${bk.burnsWithExpiry}/${bk.total}, max_cycles=${bk.maxExpiryCycles}`)

  // Utility batch stats
  const ub = report.kpis.utilityBatches
  console.log(`  Utility batches:     completed=${ub.batchCompleted}, with_errors=${ub.batchCompletedWithErrors}, item_failed=${ub.itemFailed}, item_completed=${ub.itemCompleted}`)

  console.log(SEP)

  // Batch analysis
  const ba = report.batchAnalysis
  if (ba.executesBatches.length > 0) {
    console.log('  SET_EXECUTED BATCH ANALYSIS:')
    for (const b of ba.executesBatches) {
      const status = b.failedCount > 0
        ? `✓ ${b.processedCount} processed, ${b.failedCount} failed (redundant validators)`
        : `✓ ${b.processedCount} processed`
      console.log(`    Block ${b.block}: [${b.batchType}] ${b.type} — ${status}`)
    }
    console.log(SEP)
  }

  if (ba.signatureBatches.length > 0) {
    console.log('  SIGNATURE PROPOSAL BATCH ANALYSIS:')
    for (const b of ba.signatureBatches) {
      const readyTag = b.readyCount > 0 ? `, ${b.readyCount} → Ready` : ''
      const failTag = b.failedCount > 0 ? `, ${b.failedCount} failed (validator race)` : ''
      console.log(`    Block ${b.block}: [${b.batchType}] ${b.sigCount} sigs${readyTag}${failTag}`)
    }
    console.log(SEP)
  }

  // Anomalies
  if (report.anomalies.length === 0) {
    console.log('  ✅ Anomalies: none')
  } else {
    console.log(`  Anomalies (${report.anomalies.length}):`)
    for (const a of report.anomalies) {
      const icon = a.severity === 'error' ? '\u274c' : a.severity === 'warning' ? '\u26a0\ufe0f' : '\u2139\ufe0f'
      console.log(`    ${icon} [${a.test}] ${a.message}`)
    }
  }

  // Phase markers (for debugging phasing issues)
  console.log(`\n${DSEP}`)
  console.log('  TEST PHASE MARKERS (block assignments)')
  console.log(DSEP)
  for (const m of report.phaseMarkers) {
    console.log(`    ${m.test}.${m.phase} → block ${m.block}`)
  }

  // Print timeline for key tests
  console.log(`\n${DSEP}`)
  console.log('  BLOCK TIMELINE (bridge events only)')
  console.log(DSEP)

  for (const entry of report.timeline) {
    const events = entry.events
    if (events.length === 0) continue
    const ts = entry.timestamp ? new Date(entry.timestamp).toISOString().slice(11, 19) : '??:??:??'
    console.log(`  Block ${entry.block} [${ts}]:`)
    for (const evt of events) {
      const shortData = evt.data.map(d => d.length > 20 ? d.slice(0, 16) + '...' : d).join(', ')
      console.log(`    ${evt.section}.${evt.method}(${shortData})`)
    }
  }

  // Print per-burn lifecycle (compact)
  console.log(`\n${DSEP}`)
  console.log('  BURN LIFECYCLE DETAILS')
  console.log(DSEP)

  for (const burn of report.transactions.burns) {
    const states = burn.transitions.map(t => `${t.state}@${t.block}`).join(' → ')
    const expTag = burn.expiryCount > 0 ? ` [${burn.expiryCount} expiry]` : ''
    const timeTag = burn.wallClockMs != null ? ` (${Math.round(burn.wallClockMs / 1000)}s)` : ''
    const reconTag = burn.transitions.some(t => t.state === 'processed') && burn.processedBlock === burn.transitions[burn.transitions.length - 1].block ? '' : ''
    console.log(`  Burn #${burn.burnId}: ${states}${expTag}${timeTag}${reconTag}`)
  }

  for (const refund of report.transactions.refunds) {
    const hash = typeof refund.txHash === 'string' ? refund.txHash.slice(0, 16) : String(refund.txHash).slice(0, 16)
    const states = refund.transitions.map(t => `${t.state}@${t.block}`).join(' → ')
    console.log(`  Refund ${hash}...: ${states}`)
  }

  for (const mint of report.transactions.mints) {
    const id = typeof mint.txId === 'string' ? mint.txId.slice(0, 16) : String(mint.txId).slice(0, 16)
    const states = mint.transitions.map(t => `${t.state}@${t.block}`).join(' → ')
    console.log(`  Mint ${id}...: ${states}`)
  }

  console.log(DSEP)
  console.log(`  Full report: ${process.env.ANALYSIS_OUTPUT || '/tmp/bridge_mv_analysis.json'}`)
  console.log(DSEP)
}

module.exports = {
  startEventCollector,
  markTestPhase,
  generateReport
}
