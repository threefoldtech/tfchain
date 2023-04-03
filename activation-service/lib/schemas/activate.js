module.exports = {
  type: 'object',
  properties: {
    kycSignature: { type: 'string' },
    data: {
      type: 'object',
      properties: {
        name: { type: 'string' },
        email: { type: 'string' }
      },
      required: ['name', 'email']
    },
    substrateAccountID: { type: ['string'] }
  },
  required: ['substrateAccountID'],
  additionalProperties: false
}
