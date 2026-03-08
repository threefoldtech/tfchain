#!/usr/bin/env node
/**
 * wait_for_node.js
 *
 * Polls a WebSocket endpoint until it accepts connections, then exits 0.
 * Used by Make targets to block until TFChain is ready before running setup.
 *
 * Usage:
 *   node scripts/wait_for_node.js
 *   TFCHAIN_URL=ws://localhost:9944 WAIT_TIMEOUT_MS=30000 node scripts/wait_for_node.js
 */

'use strict'

const { ApiPromise, WsProvider } = require('@polkadot/api')

const url = process.env.TFCHAIN_URL || 'ws://localhost:9944'
const timeoutMs = parseInt(process.env.WAIT_TIMEOUT_MS || '60000')
const intervalMs = 1500

async function tryConnect () {
  return new Promise((resolve) => {
    const provider = new WsProvider(url, false)
    const timer = setTimeout(() => { provider.disconnect(); resolve(false) }, 4000)
    provider.on('connected', async () => {
      clearTimeout(timer)
      try {
        const api = await ApiPromise.create({ provider, noInitWarn: true })
        await api.disconnect()
        resolve(true)
      } catch {
        resolve(false)
      }
    })
    provider.on('error', () => { clearTimeout(timer); resolve(false) })
    provider.connect()
  })
}

async function main () {
  const deadline = Date.now() + timeoutMs
  process.stdout.write(`[wait] Waiting for TFChain at ${url}`)
  while (Date.now() < deadline) {
    if (await tryConnect()) {
      console.log(' ready.')
      process.exit(0)
    }
    process.stdout.write('.')
    await new Promise(r => setTimeout(r, intervalMs))
  }
  console.log('\n[wait] Timed out waiting for TFChain.')
  process.exit(1)
}

main()
