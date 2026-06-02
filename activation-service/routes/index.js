const express = require('express')
const router = express.Router()
const { activate } = require('../controllers/substrate')
const path = require('path')

const { validateBodyMiddleware } = require('../middleware/validator')

const root = path.join(__dirname, '../build')
router.use(express.static(root))
// Handles any requests that don't match the ones above
router.get('*', (req, res) => {
  res.sendFile('index.html', { root })
})

router.post('/activate', validateBodyMiddleware('activate'), (req, res, next) => {
  const { body } = req

  activate(body)
    .then(() => res.send(body))
    .catch(next)
})

// NOTE: the POST /create-entity route was removed. It was an unauthenticated
// relayer that signed and submitted a fee-paying `createEntity` extrinsic from
// the service wallet on every call, and it had already been broken on the
// current runtime (the client's entity lookups queried renamed/removed storage,
// so every call threw and crashed the service). It served no current flow (the
// UI only calls /activate, entities are a legacy concept), so it is dropped
// rather than fixed to remove the crash + wallet-drain attack surface.

module.exports = router
