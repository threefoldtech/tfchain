#!/usr/bin/env node
/**
 * bridge_mv_setup.js
 *
 * Configures TFChain for multi-validator bridge dev testing.
 *
 * Genesis pre-configures only validator 1 (dev key 1). This script uses
 * council governance (Alice + Bob are genesis council members) to add
 * validator 2 (dev key 2) to the bridge validator set.
 *
 * TFChain bridge validator dev seeds (from chain_spec.rs):
 *   Val1: "quarter between satisfy three sphere six soda boss cute decade old trend"   (genesis)
 *   Val2: "employ split promote annual couple elder remain cricket company fitness senior fiscal" (added via council)
 *
 * Council flow: Alice proposes → Bob votes → Alice closes → call executes.
 *
 * Usage:
 *   node scripts/bridge_mv_setup.js
 */

'use strict'

const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api')

const TFCHAIN_URL = process.env.TFCHAIN_URL || 'ws://localhost:9944'

// Bridge validator dev seeds (from substrate-node/node/src/chain_spec.rs)
const VAL2_SEED = 'employ split promote annual couple elder remain cricket company fitness senior fiscal'

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

/** Wait for a block to be finalized */
async function waitBlocks (api, n = 2) {
  return new Promise((resolve) => {
    let count = 0
    const unsub = api.rpc.chain.subscribeNewHeads(header => {
      count++
      if (count >= n) {
        unsub.then(fn => fn())
        resolve()
      }
    })
  })
}

async function main () {
  log(`Connecting to TFChain at ${TFCHAIN_URL}...`)
  const api = await ApiPromise.create({ provider: new WsProvider(TFCHAIN_URL) })
  const keyring = new Keyring({ type: 'sr25519' })

  const alice = keyring.addFromUri('//Alice')
  const bob = keyring.addFromUri('//Bob')
  const val2 = keyring.addFromUri(VAL2_SEED)

  // 1. Print current state
  const validators = await api.query.tftBridgeModule.validators()
  const valList = validators.toHuman()
  log(`Current validators: ${JSON.stringify(valList)}`)

  if (valList.includes(val2.address)) {
    log(`Val2 (${val2.address}) already registered. Nothing to do.`)
    await api.disconnect()
    return
  }

  log(`Adding Val2 (${val2.address}) via council governance...`)

  // 2. Build the addBridgeValidator call
  const addVal2Call = api.tx.tftBridgeModule.addBridgeValidator(val2.address)
  const encodedCall = addVal2Call.method.toHex()
  const callLen = encodedCall.length / 2 - 1 // bytes

  // 3. Alice proposes with threshold=2 (Alice + Bob must both vote)
  log('Alice proposing addBridgeValidator(val2)...')
  const { events: proposeEvents } = await signAndWait(
    api,
    api.tx.council.propose(2, addVal2Call, callLen),
    alice
  )

  // Extract proposal hash and index from Proposed event
  let proposalHash, proposalIndex
  for (const { event } of proposeEvents) {
    if (api.events.council.Proposed.is(event)) {
      proposalHash = event.data[2].toHex() // hash is 3rd field
      proposalIndex = event.data[1].toNumber() // index is 2nd field
      break
    }
  }
  if (!proposalHash) die('Could not extract proposal hash from Proposed event')
  log(`Proposal created: hash=${proposalHash.slice(0, 10)}... index=${proposalIndex}`)

  // 4. Bob votes yes
  log('Bob voting yes...')
  await signAndWait(api, api.tx.council.vote(proposalHash, proposalIndex, true), bob)
  log('Bob voted yes.')

  // 5. Alice votes yes (she didn't automatically vote by proposing in Substrate)
  log('Alice voting yes...')
  await signAndWait(api, api.tx.council.vote(proposalHash, proposalIndex, true), alice)
  log('Alice voted yes.')

  // 6. Close the proposal (executes the call)
  log('Closing proposal...')
  const maxWeight = { refTime: BigInt(1_000_000_000), proofSize: BigInt(1_000_000) }
  await signAndWait(api, api.tx.council.close(proposalHash, proposalIndex, maxWeight, callLen), alice)
  log('Proposal closed.')

  // 7. Verify
  await waitBlocks(api, 1)
  const newValidators = await api.query.tftBridgeModule.validators()
  const newValList = newValidators.toHuman()
  log(`Updated validators: ${JSON.stringify(newValList)}`)

  if (!newValList.includes(val2.address)) {
    die('Val2 was not added — council call may have failed (check EnsureRootOrCouncilApproval)')
  }
  log('Val2 ✓ successfully added as bridge validator.')

  await api.disconnect()
}

main().catch(e => die(e.message || String(e)))
