async function acceptTermsAndConditions(self, documentLink, documentHash, callback){
  const nonce = await self.api.rpc.system.accountNextIndex(self.address)
  return await self.api.tx.tfgridModule.userAcceptTc(documentLink, documentHash).signAndSend(self.key, { nonce }, callback)
}

module.exports = {
  acceptTermsAndConditions
}