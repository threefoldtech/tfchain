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
    // Shape only — the value is validated as an account id in controllers/substrate.js.
    substrateAccountID: { type: ['string'], minLength: 1 }
  },
  required: ['substrateAccountID'],
  additionalProperties: false
}
