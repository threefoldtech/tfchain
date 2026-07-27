#!/usr/bin/env node
/**
 * bridge_accounts.js
 *
 * Sets up all Stellar accounts needed for a local bridge dev environment:
 *   - Issuer account (mints local TFT)
 *   - Bridge account (multi-sig wallet; holds TFT float)
 *   - User account (sends/receives TFT)
 *
 * Steps:
 *   1. Generate fresh keypairs for issuer, bridge, user
 *   2. Fund all three via Stellar testnet Friendbot
 *   3. Create TFT trustlines on bridge and user accounts
 *   4. Issue TFT from issuer → bridge (via path_payment_strict_send so the bridge
 *      deposit monitor ignores it — it only watches `payment` ops)
 *   5. Issue TFT from issuer → user (regular payment; user is not monitored by bridge)
 *   6. Write /tmp/bridge_local_env.sh for sourcing by Make targets and other scripts
 *
 * Usage:
 *   node scripts/bridge_accounts.js
 *
 * Override any account by setting env vars before running:
 *   BRIDGE_SECRET=S... BRIDGE_ADDRESS=G... node scripts/bridge_accounts.js
 */

'use strict'

const StellarSdk = require('@stellar/stellar-sdk')
const fs = require('fs')
const { friendbot, waitForAccount } = require('./bridge_helpers')

const HORIZON_URL = process.env.STELLAR_HORIZON_URL || 'https://horizon-testnet.stellar.org'
const NETWORK_PASSPHRASE = StellarSdk.Networks.TESTNET
const ENV_FILE = process.env.BRIDGE_ENV_FILE || '/tmp/bridge_local_env.sh'

const BRIDGE_TFT_FLOAT = process.env.BRIDGE_TFT_FLOAT || '20000'
const USER_TFT_AMOUNT = process.env.USER_TFT_AMOUNT || '1000'
const TFT_ASSET_CODE = 'TFT'

const server = new StellarSdk.Horizon.Server(HORIZON_URL)

function log (msg) { console.log(`[accounts] ${msg}`) }
function err (msg) { console.error(`[accounts] ERROR: ${msg}`); process.exit(1) }

async function main () {
  // 1. Generate or reuse keypairs
  const issuerKp = process.env.ISSUER_SECRET
    ? StellarSdk.Keypair.fromSecret(process.env.ISSUER_SECRET)
    : StellarSdk.Keypair.random()

  const bridgeKp = process.env.BRIDGE_SECRET
    ? StellarSdk.Keypair.fromSecret(process.env.BRIDGE_SECRET)
    : StellarSdk.Keypair.random()

  const userKp = process.env.USER_SECRET
    ? StellarSdk.Keypair.fromSecret(process.env.USER_SECRET)
    : StellarSdk.Keypair.random()

  log(`Issuer:  ${issuerKp.publicKey()}`)
  log(`Bridge:  ${bridgeKp.publicKey()}`)
  log(`User:    ${userKp.publicKey()}`)

  const TFT = new StellarSdk.Asset(TFT_ASSET_CODE, issuerKp.publicKey())

  // 2. Fund all three via Friendbot
  log('Funding accounts via Friendbot...')
  await Promise.all([
    friendbot(issuerKp.publicKey()),
    friendbot(bridgeKp.publicKey()),
    friendbot(userKp.publicKey())
  ])
  log('Friendbot done. Waiting for accounts to appear on Horizon...')

  const [, bridgeAcc, userAcc] = await Promise.all([
    waitForAccount(issuerKp.publicKey(), server),
    waitForAccount(bridgeKp.publicKey(), server),
    waitForAccount(userKp.publicKey(), server)
  ])

  // 3. Create TFT trustlines on bridge and user
  log('Creating TFT trustlines on bridge and user accounts...')

  async function addTrustline (kp, acc) {
    const tx = new StellarSdk.TransactionBuilder(acc, {
      fee: '1000',
      networkPassphrase: NETWORK_PASSPHRASE
    })
      .addOperation(StellarSdk.Operation.changeTrust({ asset: TFT }))
      .setTimeout(30)
      .build()
    tx.sign(kp)
    await server.submitTransaction(tx)
  }

  await Promise.all([
    addTrustline(bridgeKp, bridgeAcc),
    addTrustline(userKp, userAcc)
  ])
  log('Trustlines created.')

  // Reload accounts after trustline txs
  const [issuerAcc2] = await Promise.all([
    waitForAccount(issuerKp.publicKey(), server),
    waitForAccount(bridgeKp.publicKey(), server),
    waitForAccount(userKp.publicKey(), server)
  ])

  // 4. Fund bridge via path_payment_strict_send (invisible to bridge deposit monitor)
  log(`Issuing ${BRIDGE_TFT_FLOAT} TFT to bridge via path_payment_strict_send...`)
  const bridgeFundTx = new StellarSdk.TransactionBuilder(issuerAcc2, {
    fee: '1000',
    networkPassphrase: NETWORK_PASSPHRASE
  })
    .addOperation(StellarSdk.Operation.pathPaymentStrictSend({
      sendAsset: TFT,
      sendAmount: BRIDGE_TFT_FLOAT,
      destination: bridgeKp.publicKey(),
      destAsset: TFT,
      destMin: String(Number(BRIDGE_TFT_FLOAT) - 1),
      path: []
    }))
    .setTimeout(30)
    .build()
  bridgeFundTx.sign(issuerKp)
  await server.submitTransaction(bridgeFundTx)
  log(`Bridge funded with ${BRIDGE_TFT_FLOAT} TFT.`)

  // Reload issuer after bridge funding tx
  const issuerAcc3 = await waitForAccount(issuerKp.publicKey(), server)

  // 5. Fund user via regular payment (user account is not monitored by bridge)
  log(`Issuing ${USER_TFT_AMOUNT} TFT to user...`)
  const userFundTx = new StellarSdk.TransactionBuilder(issuerAcc3, {
    fee: '1000',
    networkPassphrase: NETWORK_PASSPHRASE
  })
    .addOperation(StellarSdk.Operation.payment({
      destination: userKp.publicKey(),
      asset: TFT,
      amount: USER_TFT_AMOUNT
    }))
    .setTimeout(30)
    .build()
  userFundTx.sign(issuerKp)
  await server.submitTransaction(userFundTx)
  log(`User funded with ${USER_TFT_AMOUNT} TFT.`)

  // 6. Write env file
  const envContent = `# Auto-generated by bridge_accounts.js — do not edit manually
export ISSUER_ADDRESS="${issuerKp.publicKey()}"
export ISSUER_SECRET="${issuerKp.secret()}"
export BRIDGE_ADDRESS="${bridgeKp.publicKey()}"
export BRIDGE_SECRET="${bridgeKp.secret()}"
export USER_ADDRESS="${userKp.publicKey()}"
export USER_SECRET="${userKp.secret()}"
export TFT_ASSET_CODE="${TFT_ASSET_CODE}"
export STELLAR_HORIZON_URL="${HORIZON_URL}"
export STELLAR_NETWORK="testnet"
`
  fs.writeFileSync(ENV_FILE, envContent)
  log(`Environment written to ${ENV_FILE}`)
  log('Done.')
}

main().catch(e => err(e.message || String(e)))
