#!/usr/bin/env node
/**
 * bridge_mv_setup.js
 *
 * Configures TFChain for multi-validator bridge dev testing.
 *
 * Genesis pre-configures only validator 1 (dev key 1). This script uses
 * council governance (Alice + Bob are genesis council members, 2-of-2) to
 * add validators 2 and 3 to the bridge validator set.
 *
 * TFChain bridge validator dev seeds (from chain_spec.rs):
 *   Val1: "quarter between satisfy three sphere six soda boss cute decade old trend"              (genesis)
 *   Val2: "employ split promote annual couple elder remain cricket company fitness senior fiscal" (added here)
 *   Val3: "remind bird banner word spread volume card keep want faith insect mind"               (added here)
 *
 * Council flow per validator: Alice proposes → Bob votes yes → Alice votes yes → Alice closes.
 *
 * Usage:
 *   node scripts/bridge_mv_setup.js
 */

'use strict'

const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api')

const TFCHAIN_URL = process.env.TFCHAIN_URL || 'ws://localhost:9944'

// Bridge validator dev seeds (from substrate-node/node/src/chain_spec.rs)
const VAL2_SEED = 'employ split promote annual couple elder remain cricket company fitness senior fiscal'
const VAL3_SEED = 'remind bird banner word spread volume card keep want faith insect mind'

function log (msg) { console.log(`[mv-setup] ${msg}`) }
function die (msg) { console.error(`[mv-setup] FATAL: ${msg}`); process.exit(1) }

/** Sign a tx, wait for InBlock, and throw on dispatch error */
function signAndWait (api, tx, signer) {
  return new Promise((resolve, reject) => {
    let unsub
    tx.signAndSend(signer, ({ status, dispatchError, events }) => {
      if (!status.isInBlock && !status.isFinalized) return
      if (dispatchError) {
        let msg = dispatchError.toString()
        if (dispatchError.isModule) {
          try {
            const decoded = api.registry.findMetaError(dispatchError.asModule)
            msg = `${decoded.section}.${decoded.name}: ${decoded.docs}`
          } catch {}
        }
        if (unsub) unsub()
        reject(new Error(msg))
        return
      }
      if (unsub) unsub()
      resolve({ status, events })
    }).then(u => { unsub = u }).catch(reject)
  })
}

/** Wait for N new blocks */
async function waitBlocks (api, n = 1) {
  return new Promise((resolve) => {
    let count = 0
    api.rpc.chain.subscribeNewHeads(header => {
      count++
      if (count >= n) resolve()
    })
  })
}

/** Add a bridge validator via council governance (Alice proposes, both vote, Alice closes) */
async function addValidatorViaCouncil (api, alice, bob, validatorAddress, label) {
  // Check if already registered
  const validators = await api.query.tftBridgeModule.validators()
  const valList = validators.toHuman()
  log(`Current validators: ${JSON.stringify(valList)}`)

  if (valList.includes(validatorAddress)) {
    log(`${label} (${validatorAddress}) already registered. Skipping.`)
    return
  }

  log(`Adding ${label} (${validatorAddress}) via council governance...`)

  // Build the addBridgeValidator call
  const addValCall = api.tx.tftBridgeModule.addBridgeValidator(validatorAddress)
  const encodedCall = addValCall.method.toHex()
  const callLen = encodedCall.length / 2 - 1 // bytes

  // Alice proposes with threshold=2 (Alice + Bob must both vote)
  log(`Alice proposing addBridgeValidator(${label})...`)
  const { events: proposeEvents } = await signAndWait(
    api,
    api.tx.council.propose(2, addValCall, callLen),
    alice
  )

  // Extract proposal hash and index from Proposed event
  let proposalHash, proposalIndex
  for (const { event } of proposeEvents) {
    if (api.events.council.Proposed.is(event)) {
      proposalHash = event.data[2].toHex()
      proposalIndex = event.data[1].toNumber()
      break
    }
  }
  if (!proposalHash) die('Could not extract proposal hash from Proposed event')
  log(`Proposal: hash=${proposalHash.slice(0, 12)}... index=${proposalIndex}`)

  // Bob votes yes
  log('Bob voting yes...')
  await signAndWait(api, api.tx.council.vote(proposalHash, proposalIndex, true), bob)
  log('Bob voted yes.')

  // Alice votes yes
  log('Alice voting yes...')
  await signAndWait(api, api.tx.council.vote(proposalHash, proposalIndex, true), alice)
  log('Alice voted yes.')

  // Close the proposal (executes the call)
  log('Closing proposal...')
  const maxWeight = { refTime: BigInt(1_000_000_000), proofSize: BigInt(1_000_000) }
  await signAndWait(api, api.tx.council.close(proposalHash, proposalIndex, maxWeight, callLen), alice)
  log('Proposal closed.')

  // Verify
  await waitBlocks(api, 1)
  const newValidators = await api.query.tftBridgeModule.validators()
  const newValList = newValidators.toHuman()
  log(`Updated validators: ${JSON.stringify(newValList)}`)

  if (!newValList.includes(validatorAddress)) {
    die(`${label} was not added — council call may have failed`)
  }
  log(`${label} ✓ successfully added as bridge validator.`)
}

async function main () {
  log(`Connecting to TFChain at ${TFCHAIN_URL}...`)
  const api = await ApiPromise.create({ provider: new WsProvider(TFCHAIN_URL) })
  const keyring = new Keyring({ type: 'sr25519' })

  const alice = keyring.addFromUri('//Alice')
  const bob = keyring.addFromUri('//Bob')
  const val2 = keyring.addFromUri(VAL2_SEED)
  const val3 = keyring.addFromUri(VAL3_SEED)

  log(`Val2 address: ${val2.address}`)
  log(`Val3 address: ${val3.address}`)

  // Add val2 via council governance
  await addValidatorViaCouncil(api, alice, bob, val2.address, 'Val2')

  // Add val3 via council governance
  await addValidatorViaCouncil(api, alice, bob, val3.address, 'Val3')

  // Create Alice's twin (needed for MV2 deposit test)
  const aliceTwinOpt = await api.query.tfgridModule.twinIdByAccountID(alice.address)
  const aliceTwinId = aliceTwinOpt.toJSON()
  if (!aliceTwinId) {
    log('Creating Alice twin for deposit tests...')
    await signAndWait(api, api.tx.tfgridModule.createTwin(null, null), alice)
    const newTwin = await api.query.tfgridModule.twinIdByAccountID(alice.address)
    log(`Alice twin created (ID: ${newTwin.toJSON()})`)
  } else {
    log(`Alice twin already exists (ID: ${aliceTwinId})`)
  }

  // Final state
  const finalValidators = await api.query.tftBridgeModule.validators()
  log(`Final validators: ${JSON.stringify(finalValidators.toHuman())}`)

  await api.disconnect()
  log('Setup complete.')
}

main().catch(e => die(e.message || String(e)))
