async function getPricingPolicyById (self, policyId) {
  const value = await self.api.query.tfgridModule.pricingPolicies(policyId)

  return value.toJSON()
}
module.exports = {
  getPricingPolicyById
}
