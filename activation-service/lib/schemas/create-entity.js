module.exports = {
  type: 'object',
  properties: {
    target: { type: 'string' },
    name: { type: 'string' },
    signature: { type: 'string' },
    countryID: { type: 'number' },
    cityID: { type: 'number' }
  },
  required: ['target', 'name', 'signature', 'countryID', 'cityID'],
  additionalProperties: false
}
