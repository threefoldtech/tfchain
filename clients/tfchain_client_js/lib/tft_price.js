async function tftPrice (self) {
  const value = await self.api.query.tftPriceModule.tftPrice()

  return value.toJSON()
}
module.exports = {
  tftPrice
}
