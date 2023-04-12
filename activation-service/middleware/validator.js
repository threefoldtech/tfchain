const httpError = require('http-errors')
const validate = require('../lib/validate')

function validateBodyMiddleware (schema) {
  if (!schema) throw new Error('schema has to be provided')
  return (req, res, next) => {
    const { body } = req
    validate(schema, body)
      .then(_ => next())
      .catch(err => next(httpError(400, err.message)))
  }
}

module.exports = {
  validateBodyMiddleware
}
