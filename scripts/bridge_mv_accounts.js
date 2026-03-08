#!/usr/bin/env node
/**
 * bridge_mv_accounts.js
 *
 * Sets up all Stellar accounts for a multi-validator (3-of-3 signers, threshold=2) bridge:
 *
 *   - Val1 keypair  — acts as the bridge account master key AND first validator signer
 *   - Val2 keypair  — second validator signer (added to bridge multi-sig)
 *   - Val3 keypair  — third validator signer (added to bridge multi-sig)
 *   - User keypair  — test user (sends/receives TFT)
 *   - Issuer keypair — mints local TFT
 *
 * Multi-sig configuration:
 *   - Bridge account = Val1's Stellar account (master key)
 *   - Val2 and Val3 added as signers with weight=1 each
 *   - Thresholds: low=1, med=2 (tx signing), high=3
 *   - Any 2 of 3 validators can sign a transaction
 *
 * Steps:
 *   1. Generate keypairs (or reuse from env)
 *   2. Fund all accounts via Friendbot
 *   3. Create TFT trustlines on bridge (val1) and user
 *   4. Fund bridge via path_payment_strict_send (invisible to deposit monitor)
 *   5. Fund user via regular payment
 *   6. Configure bridge account as 2-of-3 multi-sig
 *   7. Write /tmp/bridge_mv_env.sh
 *
 * Usage:
 *   node scripts/bridge_mv_accounts.js
 */

'use strict'

const StellarSdk = require('@stellar/stellar-sdk')
const https = require('https')
const fs = require('fs')

const HORIZON_URL = process.env.STELLAR_HORIZON_URL || 'https://horizon-testnet.stellar.org'
const NETWORK_PASSPHRASE = StellarSdk.Networks.TESTNET
const FRIENDBOT_URL = 'https://friendbot.stellar.org'
const ENV_FILE = process.env.BRIDGE_MV_ENV_FILE || '/tmp/bridge_mv_env.sh'

const BRIDGE_TFT_FLOAT = process.env.BRIDGE_TFT_FLOAT || '20000'
const USER_TFT_AMOUNT = process.env.USER_TFT_AMOUNT || '1000'
const TFT_ASSET_CODE = 'TFT'
const MED_THRESHOLD = 2  // signatures required for TFT payments
const LOW_THRESHOLD = 1
const HIGH_THRESHOLD = 3

const server = new StellarSdk.Horizon.Server(HORIZON_URL)

function log (msg) { console.log(`[mv-accounts] ${msg}`) }
function die (msg) { console.error(`[mv-accounts] ERROR: ${msg}`); process.exit(1) }

async function friendbot (address) {
  return new Promise((resolve, reject) => {
    https.get(`${FRIENDBOT_URL}?addr=${address}`, (res) => {
      let data = ''
      res.on('data', c => { data += c })
      res.on('end', () => {
        // 400 = already funded — fine
        if (res.statusCode === 200 || res.statusCode === 400) resolve()
        else reject(new Error(`Friendbot ${res.statusCode}: ${data.slice(0, 200)}`))
      })
    }).on('error', reject)
  })
}

async function waitForAccount (address, retries = 12) {
  for (let i = 0; i < retries; i++) {
    try { return await server.loadAccount(address) } catch {}
    await new Promise(r => setTimeout(r, 2000))
  }
  die(`Account ${address} not found after ${retries} attempts`)
}

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
  // 1. Generate or reuse keypairs
  const issuerKp = process.env.ISSUER_SECRET
    ? StellarSdk.Keypair.fromSecret(process.env.ISSUER_SECRET)
    : StellarSdk.Keypair.random()

  // Val1 key IS the bridge account master key
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

  const bridgeAddress = val1Kp.publicKey()  // bridge account = val1

  log(`Issuer:  ${issuerKp.publicKey()}`)
  log(`Val1 / Bridge: ${val1Kp.publicKey()}`)
  log(`Val2:    ${val2Kp.publicKey()}`)
  log(`Val3:    ${val3Kp.publicKey()}`)
  log(`User:    ${userKp.publicKey()}`)

  const TFT = new StellarSdk.Asset(TFT_ASSET_CODE, issuerKp.publicKey())

  // 2. Fund all accounts via Friendbot
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
    waitForAccount(issuerKp.publicKey()),
    waitForAccount(val1Kp.publicKey()),
    waitForAccount(val2Kp.publicKey()),
    waitForAccount(val3Kp.publicKey()),
    waitForAccount(userKp.publicKey())
  ])

  // 3. Create TFT trustlines on bridge (val1) and user
  log('Creating TFT trustlines on bridge and user...')
  await Promise.all([
    submitTx(val1Kp, bridgeAcc, [StellarSdk.Operation.changeTrust({ asset: TFT })]),
    submitTx(userKp, userAcc, [StellarSdk.Operation.changeTrust({ asset: TFT })])
  ])
  log('Trustlines created.')

  // Reload accounts
  const [issuerAcc2, bridgeAcc2, , , userAcc2] = await Promise.all([
    waitForAccount(issuerKp.publicKey()),
    waitForAccount(val1Kp.publicKey()),
    waitForAccount(val2Kp.publicKey()),
    waitForAccount(val3Kp.publicKey()),
    waitForAccount(userKp.publicKey())
  ])

  // 4. Fund bridge via path_payment_strict_send (invisible to bridge deposit monitor)
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

  // Reload issuer
  const issuerAcc3 = await waitForAccount(issuerKp.publicKey())

  // 5. Fund user via regular payment
  log(`Issuing ${USER_TFT_AMOUNT} TFT to user...`)
  await submitTx(issuerKp, issuerAcc3, [
    StellarSdk.Operation.payment({
      destination: userKp.publicKey(),
      asset: TFT,
      amount: USER_TFT_AMOUNT
    })
  ])
  log(`User funded with ${USER_TFT_AMOUNT} TFT.`)

  // Reload bridge account for multi-sig setup
  const bridgeAcc3 = await waitForAccount(val1Kp.publicKey())

  // 6. Configure bridge account as 2-of-3 multi-sig
  // Val1 is the master key (weight 1 by default), add val2 and val3 as signers (weight 1 each)
  // After this: any 2 signatures meet the med threshold (2) required for TFT payments
  log(`Configuring bridge as ${MED_THRESHOLD}-of-3 multi-sig (low=${LOW_THRESHOLD}, med=${MED_THRESHOLD}, high=${HIGH_THRESHOLD})...`)
  await submitTx(val1Kp, bridgeAcc3, [
    StellarSdk.Operation.setOptions({
      signer: { ed25519PublicKey: val2Kp.publicKey(), weight: 1 }
    }),
    StellarSdk.Operation.setOptions({
      signer: { ed25519PublicKey: val3Kp.publicKey(), weight: 1 }
    }),
    StellarSdk.Operation.setOptions({
      lowThreshold: LOW_THRESHOLD,
      medThreshold: MED_THRESHOLD,
      highThreshold: HIGH_THRESHOLD
    })
  ])
  log('Multi-sig configured.')

  // Verify
  const finalAcc = await waitForAccount(bridgeAddress)
  log(`Bridge thresholds: low=${finalAcc.thresholds.low_threshold} med=${finalAcc.thresholds.med_threshold} high=${finalAcc.thresholds.high_threshold}`)
  log(`Bridge signers: ${finalAcc.signers.length} (expected 3)`)

  // 7. Write env file
  const envContent = `# Auto-generated by bridge_mv_accounts.js — do not edit manually
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
export MV_MED_THRESHOLD="${MED_THRESHOLD}"
`
  fs.writeFileSync(ENV_FILE, envContent)
  log(`Environment written to ${ENV_FILE}`)
  log('Done.')
}

main().catch(e => die(e.message || String(e)))
