const { hex2a } = require('./util')

async function getPricingPolicyById (self, policyId) {
  const value = await self.api.query.tfgridModule.pricingPolicies(policyId)

  const res = value.toJSON()
  // The policy `name` is byte-encoded; decode it to a string. A non-existent
  // policy decodes to null, so guard before touching fields.
  if (res && res.name) {
    res.name = hex2a(res.name)
  }
  return res
}
module.exports = {
  getPricingPolicyById
}
