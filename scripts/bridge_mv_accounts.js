#!/usr/bin/env node
/**
 * bridge_mv_accounts.js
 *
 * Sets up Stellar accounts for a 3-validator bridge dev environment.
 *
 * Multi-sig configuration (2-of-3):
 *   - Bridge account = Val1 Stellar keypair (master key, weight=1)
 *   - Val2 Stellar keypair added as signer (weight=1)
 *   - Val3 Stellar keypair added as signer (weight=1)
 *   - Thresholds: low=1, med=2 (any 2 of 3 can sign TFT payments), high=2
 *
 * TFChain genesis bridge validators (from substrate-node/node/src/chain_spec.rs):
 *   Val1: "quarter between satisfy three sphere six soda boss cute decade old trend"   (genesis)
 *   Val2: "employ split promote annual couple elder remain cricket company fitness senior fiscal" (added via council)
 *   Val3: "remind bird banner word spread volume card keep want faith insect mind"     (added via council)
 *
 * Steps:
 *   1. Generate keypairs: val1 (bridge master), val2, val3, user, issuer
 *   2. Fund all via Stellar Friendbot
 *   3. Create TFT trustline on bridge (val1) and user
 *   4. Fund bridge via path_payment_strict_send (invisible to deposit monitor)
 *   5. Fund user via regular payment
 *   6. Configure bridge as 2-of-3 multi-sig (val1 master + val2 + val3 signers)
 *   7. Write env file
 *
 * Usage:
 *   node scripts/bridge_mv_accounts.js
 */

'use strict'

const StellarSdk = require('@stellar/stellar-sdk')
const fs = require('fs')
const { friendbot, waitForAccount } = require('./bridge_helpers')

const HORIZON_URL = process.env.STELLAR_HORIZON_URL || 'https://horizon-testnet.stellar.org'
const NETWORK_PASSPHRASE = StellarSdk.Networks.TESTNET
const ENV_FILE = process.env.BRIDGE_MV_ENV_FILE || '/tmp/bridge_mv_env.sh'

const BRIDGE_TFT_FLOAT = process.env.BRIDGE_TFT_FLOAT || '20000'
const USER_TFT_AMOUNT = process.env.USER_TFT_AMOUNT || '1000'
const TFT_ASSET_CODE = 'TFT'

const server = new StellarSdk.Horizon.Server(HORIZON_URL)

function log (msg) { console.log(`[mv-accounts] ${msg}`) }
function die (msg) { console.error(`[mv-accounts] ERROR: ${msg}`); process.exit(1) }

async function submitTx (kp, acc, ops) {
  const builder = new StellarSdk.TransactionBuilder(acc, {
    fee: '1000',
    networkPassphrase: NETWORK_PASSPHRASE
  })
  for (const op of ops) builder.addOperation(op)
  const tx = builder.setTimeout(30).build()
  tx.sign(kp)
  return server.submitTransaction(tx)
}

async function main () {
  // 1. Generate keypairs
  const val1Kp = process.env.VAL1_STELLAR_SECRET
    ? StellarSdk.Keypair.fromSecret(process.env.VAL1_STELLAR_SECRET)
    : StellarSdk.Keypair.random()

  const val2Kp = process.env.VAL2_STELLAR_SECRET
    ? StellarSdk.Keypair.fromSecret(process.env.VAL2_STELLAR_SECRET)
    : StellarSdk.Keypair.random()

  const val3Kp = process.env.VAL3_STELLAR_SECRET
    ? StellarSdk.Keypair.fromSecret(process.env.VAL3_STELLAR_SECRET)
    : StellarSdk.Keypair.random()

  const userKp = process.env.USER_SECRET
    ? StellarSdk.Keypair.fromSecret(process.env.USER_SECRET)
    : StellarSdk.Keypair.random()

  const issuerKp = process.env.ISSUER_SECRET
    ? StellarSdk.Keypair.fromSecret(process.env.ISSUER_SECRET)
    : StellarSdk.Keypair.random()

  const bridgeAddress = val1Kp.publicKey()

  log(`Issuer:        ${issuerKp.publicKey()}`)
  log(`Val1 / Bridge: ${val1Kp.publicKey()}`)
  log(`Val2:          ${val2Kp.publicKey()}`)
  log(`Val3:          ${val3Kp.publicKey()}`)
  log(`User:          ${userKp.publicKey()}`)

  const TFT = new StellarSdk.Asset(TFT_ASSET_CODE, issuerKp.publicKey())

  // 2. Fund via Friendbot
  log('Funding accounts via Friendbot...')
  await Promise.all([
    friendbot(issuerKp.publicKey()),
    friendbot(val1Kp.publicKey()),
    friendbot(val2Kp.publicKey()),
    friendbot(val3Kp.publicKey()),
    friendbot(userKp.publicKey())
  ])
  log('Friendbot done. Waiting for accounts...')

  const [, bridgeAcc, , , userAcc] = await Promise.all([
    waitForAccount(issuerKp.publicKey(), server),
    waitForAccount(val1Kp.publicKey(), server),
    waitForAccount(val2Kp.publicKey(), server),
    waitForAccount(val3Kp.publicKey(), server),
    waitForAccount(userKp.publicKey(), server)
  ])

  // 3. TFT trustlines on bridge and user only (val2/val3 are signers, not holders)
  log('Creating TFT trustlines on bridge and user...')
  await Promise.all([
    submitTx(val1Kp, bridgeAcc, [StellarSdk.Operation.changeTrust({ asset: TFT })]),
    submitTx(userKp, userAcc, [StellarSdk.Operation.changeTrust({ asset: TFT })])
  ])
  log('Trustlines created.')

  const [issuerAcc2] = await Promise.all([
    waitForAccount(issuerKp.publicKey(), server),
    waitForAccount(val1Kp.publicKey(), server),
    waitForAccount(val2Kp.publicKey(), server),
    waitForAccount(val3Kp.publicKey(), server),
    waitForAccount(userKp.publicKey(), server)
  ])

  // 4. Fund bridge via path_payment_strict_send (invisible to deposit monitor)
  log(`Issuing ${BRIDGE_TFT_FLOAT} TFT to bridge via path_payment_strict_send...`)
  await submitTx(issuerKp, issuerAcc2, [
    StellarSdk.Operation.pathPaymentStrictSend({
      sendAsset: TFT,
      sendAmount: BRIDGE_TFT_FLOAT,
      destination: bridgeAddress,
      destAsset: TFT,
      destMin: String(Number(BRIDGE_TFT_FLOAT) - 1),
      path: []
    })
  ])
  log(`Bridge funded with ${BRIDGE_TFT_FLOAT} TFT.`)

  // 5. Fund user
  const issuerAcc3 = await waitForAccount(issuerKp.publicKey(), server)
  log(`Issuing ${USER_TFT_AMOUNT} TFT to user...`)
  await submitTx(issuerKp, issuerAcc3, [
    StellarSdk.Operation.payment({
      destination: userKp.publicKey(),
      asset: TFT,
      amount: USER_TFT_AMOUNT
    })
  ])
  log(`User funded with ${USER_TFT_AMOUNT} TFT.`)

  // 6. Configure bridge as 2-of-3 multi-sig
  // Val1 is master (weight=1), val2 and val3 added as signers (weight=1 each)
  // Any 2 of 3 signers needed for med ops (TFT payments)
  const bridgeAcc3 = await waitForAccount(val1Kp.publicKey(), server)
  log('Configuring bridge as 2-of-3 multi-sig (low=1, med=2, high=2)...')
  await submitTx(val1Kp, bridgeAcc3, [
    StellarSdk.Operation.setOptions({
      signer: { ed25519PublicKey: val2Kp.publicKey(), weight: 1 }
    }),
    StellarSdk.Operation.setOptions({
      signer: { ed25519PublicKey: val3Kp.publicKey(), weight: 1 }
    }),
    StellarSdk.Operation.setOptions({
      lowThreshold: 1,
      medThreshold: 2,
      highThreshold: 2
    })
  ])

  const finalAcc = await waitForAccount(bridgeAddress, server)
  log(`Bridge thresholds: low=${finalAcc.thresholds.low_threshold} med=${finalAcc.thresholds.med_threshold} high=${finalAcc.thresholds.high_threshold}`)
  log(`Bridge signers: ${finalAcc.signers.length} (expected 3: val1 master + val2 + val3)`)

  // 7. Write env file
  const envContent = `# Auto-generated by bridge_mv_accounts.js — do not edit manually
# Multi-sig: 2-of-3 (val1 master + val2 + val3 all weight=1, med threshold=2)
# TFChain validator seeds (from substrate-node/node/src/chain_spec.rs):
#   Val1: "quarter between satisfy three sphere six soda boss cute decade old trend"               (genesis)
#   Val2: "employ split promote annual couple elder remain cricket company fitness senior fiscal"  (added via council)
#   Val3: "remind bird banner word spread volume card keep want faith insect mind"                (added via council)
export ISSUER_ADDRESS="${issuerKp.publicKey()}"
export ISSUER_SECRET="${issuerKp.secret()}"
export BRIDGE_ADDRESS="${bridgeAddress}"
export VAL1_STELLAR_SECRET="${val1Kp.secret()}"
export VAL1_STELLAR_ADDRESS="${val1Kp.publicKey()}"
export VAL2_STELLAR_SECRET="${val2Kp.secret()}"
export VAL2_STELLAR_ADDRESS="${val2Kp.publicKey()}"
export VAL3_STELLAR_SECRET="${val3Kp.secret()}"
export VAL3_STELLAR_ADDRESS="${val3Kp.publicKey()}"
export USER_ADDRESS="${userKp.publicKey()}"
export USER_SECRET="${userKp.secret()}"
export TFT_ASSET_CODE="${TFT_ASSET_CODE}"
export STELLAR_HORIZON_URL="${HORIZON_URL}"
export STELLAR_NETWORK="testnet"
export MV_MED_THRESHOLD="2"
`
  fs.writeFileSync(ENV_FILE, envContent)
  log(`Environment written to ${ENV_FILE}`)
  log('Done.')
}

main().catch(e => die(e.message || String(e)))
