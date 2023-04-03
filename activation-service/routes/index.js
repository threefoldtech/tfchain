const express = require('express')
const router = express.Router()
const { activate, createEntity } = require('../controllers/substrate')
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

router.post('/create-entity', validateBodyMiddleware('create-entity'), (req, res, next) => {
  const { body } = req

  res.setHeader('Content-Type', 'text/html; charset=utf-8')
  res.setHeader('Tranfer-Encoding', 'chunked')

  createEntity(body, res, next)
})

module.exports = router
