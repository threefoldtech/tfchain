const validate = require('jsonschema').validate

async function validateP (schemaName, object = {}) {
  return new Promise((resolve, reject) => {
    let schema
    try {
      schema = require(`./schemas/${schemaName}`)
    } catch (err) {
      if (err.code === 'MODULE_NOT_FOUND') return reject(new Error(`no schema defined under name ${schema}`))
      return reject(err)
    }

    const result = validate(object, schema)

    result.valid
      ? resolve(object)
      : reject(new Error(result.errors.toString()))
  })
}

module.exports = validateP
