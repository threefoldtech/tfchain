const { STATUS_CODES } = require('node:http')

const express = require('express')
const cors = require('cors')
const httpError = require('http-errors')

// attach env variables to process.env
require('dotenv').config()

const log = require('./lib/logger')

const pinoMiddleware = require('express-pino-logger')({
  logger: log
})

const app = express()

app.use(cors())
app.options('*', cors())
app.use(express.urlencoded({ extended: true }))
app.use(express.json())
app.use(pinoMiddleware)

app.use('/activation', require('./routes'))

app.use((req, res, next) => next(httpError.NotFound()))

app.use(function (err, req, res, next) {
  const status = err.status || err.statusCode || 500

  if (status >= 500) {
    log.error({ err }, 'error happened handling the request')
  } else {
    // The caller sent something unusable; that is not a fault on our side.
    log.warn({ err }, 'rejected the request')
  }

  // http-errors sets `expose` true for 4xx and false for 5xx: a client caused a
  // 4xx and needs to know why, whereas a 5xx message describes our internals.
  //
  // That flag used to be computed and then ignored. Both branches of the old
  // NODE_ENV check sent the message regardless — lodash's omit copies `message`
  // through, so the "production" path returned things like the decoded chain
  // error alongside `expose: false`, and the development path (the default,
  // since NODE_ENV is set nowhere) added the stack trace.
  res.status(status).json({
    message: err.expose ? err.message : STATUS_CODES[status]
  })
})

module.exports = app
