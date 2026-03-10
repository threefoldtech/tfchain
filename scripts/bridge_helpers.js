#!/usr/bin/env node
/**
 * bridge_helpers.js
 *
 * Shared helpers for bridge E2E test scripts. Extracted from bridge_tests.js
 * and bridge_mv_tests.js to eliminate duplication.
 */

'use strict'

const fs = require('fs')
const https = require('https')
const StellarSdk = require('@stellar/stellar-sdk')

// ─── Logging & test result helpers ──────────────────────────────────────────

function log (msg) { console.log(`  ${msg}`) }
function pass (name, counter) { console.log(`\u2705 PASS: ${name}`); counter.passed++ }
function fail (name, reason, counter) { console.error(`\u274c FAIL: ${name} \u2014 ${reason}`); counter.failed++ }

// ─── Env helpers ────────────────────────────────────────────────────────────

/**
 * Load a shell-style env file (export KEY="value") into process.env.
 * @param {string} envFile - Path to the env file
 * @param {string} [label='tests'] - Label for error messages
 */
function loadEnv (envFile, label = 'tests') {
  if (!fs.existsSync(envFile)) {
    console.error(`[${label}] Env file not found: ${envFile}. Run the accounts target first.`)
    process.exit(1)
  }
  const lines = fs.readFileSync(envFile, 'utf8').split('\n')
  for (const line of lines) {
    const m = line.match(/^export\s+(\w+)="([^"]*)"/)
    if (m) process.env[m[1]] = m[2]
  }
}

/**
 * Get a required env var or exit with an error.
 */
function getEnv (key) {
  const val = process.env[key]
  if (!val) { console.error(`Missing env var: ${key}`); process.exit(1) }
  return val
}

// ─── Stellar helpers ────────────────────────────────────────────────────────

/**
 * Get TFT balance for a Stellar address.
 * @param {string} address - Stellar public key
 * @param {object} horizon - Horizon.Server instance
 * @param {string} issuerAddress - TFT issuer public key
 */
async function stellarTFTBalance (address, horizon, issuerAddress) {
  const acc = await horizon.loadAccount(address)
  const tft = acc.balances.find(b => b.asset_code === 'TFT' && b.asset_issuer === issuerAddress)
  return tft ? parseFloat(tft.balance) : 0
}

/**
 * Fund a Stellar address via Friendbot (testnet).
 * Treats HTTP 400 as success (already funded).
 * @param {string} address - Stellar public key
 * @param {string} [friendbotUrl='https://friendbot.stellar.org'] - Friendbot URL
 */
async function friendbot (address, friendbotUrl = 'https://friendbot.stellar.org') {
  return new Promise((resolve, reject) => {
    https.get(`${friendbotUrl}?addr=${address}`, (res) => {
      let data = ''
      res.on('data', c => { data += c })
      res.on('end', () => {
        if (res.statusCode === 200 || res.statusCode === 400) resolve()
        else reject(new Error(`Friendbot ${res.statusCode}: ${data.slice(0, 200)}`))
      })
    }).on('error', reject)
  })
}

/**
 * Wait for a Stellar account to appear on Horizon.
 * @param {string} address - Stellar public key
 * @param {object} server - Horizon.Server instance
 * @param {number} [retries=12] - Max retry attempts (2s apart)
 */
async function waitForAccount (address, server, retries = 12) {
  for (let i = 0; i < retries; i++) {
    try { return await server.loadAccount(address) } catch {}
    if (i < retries - 1) await new Promise(r => setTimeout(r, 2000))
  }
  throw new Error(`Account ${address} not found after ${retries} attempts`)
}

/**
 * Send a Stellar payment (TFT asset) with optional memo.
 * @param {object} horizon - Horizon.Server instance
 * @param {string} issuerAddress - TFT issuer public key
 * @param {string} networkPassphrase - Stellar network passphrase
 * @param {string} fromSecret - Sender's Stellar secret key
 * @param {string} toAddress - Destination Stellar public key
 * @param {string|number} amount - Amount to send (string for Stellar precision)
 * @param {string|null} [memo=null] - Optional text memo (e.g. "twin_<id>")
 * @returns {Promise<object>} Horizon submit result (includes .hash)
 */
async function sendStellarPayment (horizon, issuerAddress, networkPassphrase, fromSecret, toAddress, amount, memo = null) {
  const kp = StellarSdk.Keypair.fromSecret(fromSecret)
  const TFTAsset = new StellarSdk.Asset('TFT', issuerAddress)
  const acc = await horizon.loadAccount(kp.publicKey())

  const builder = new StellarSdk.TransactionBuilder(acc, {
    fee: '1000',
    networkPassphrase
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

// ─── Polling helper ─────────────────────────────────────────────────────────

/**
 * Poll until condition() returns truthy or timeout.
 * @param {function} condition - Async function; return truthy to stop
 * @param {object} opts
 * @param {number} [opts.timeoutMs=180000] - Timeout in ms
 * @param {number} [opts.intervalMs=3000] - Poll interval in ms
 * @param {string} [opts.desc=''] - Description for timeout error
 */
async function waitUntil (condition, { timeoutMs = 180_000, intervalMs = 3000, desc = '' } = {}) {
  const deadline = Date.now() + timeoutMs
  while (Date.now() < deadline) {
    const result = await condition()
    if (result) return result
    await new Promise(r => setTimeout(r, intervalMs))
  }
  throw new Error(`Timeout waiting for: ${desc}`)
}

// ─── TFChain helpers ────────────────────────────────────────────────────────

// TFT has 7 decimal places: 1 TFT = 10_000_000 base units
const TFT_DECIMALS = 1e7
const TFT = (amount) => Math.round(amount * TFT_DECIMALS)

/**
 * Get free TFT balance for a TFChain address (returns float in TFT units).
 * @param {object} api - ApiPromise instance
 * @param {string} address - TFChain SS58 address
 */
async function tfchainBalance (api, address) {
  const { data } = await api.query.system.account(address)
  return Number(data.free) / TFT_DECIMALS
}

/**
 * Submit a swapToStellar extrinsic on TFChain.
 * @param {object} api - ApiPromise instance
 * @param {object} signer - Keyring pair (e.g. alice)
 * @param {number} amountTFT - Amount in whole TFT
 * @param {object} opts
 * @param {string} opts.userAddress - Stellar destination address
 * @param {number} [opts.nonce=-1] - Explicit nonce (-1 = auto)
 */
async function swapToStellar (api, signer, amountTFT, { userAddress, nonce = -1 } = {}) {
  return new Promise((resolve, reject) => {
    api.tx.tftBridgeModule.swapToStellar(userAddress, TFT(amountTFT))
      .signAndSend(signer, { nonce }, ({ status, dispatchError, events }) => {
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

module.exports = {
  log,
  pass,
  fail,
  loadEnv,
  getEnv,
  stellarTFTBalance,
  tfchainBalance,
  friendbot,
  waitForAccount,
  waitUntil,
  sendStellarPayment,
  swapToStellar,
  TFT,
  TFT_DECIMALS
}
