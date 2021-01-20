const { getEntity } = require('./entity')
const { getTwin } = require('./twin')

async function signEntityTwinID (self, entityID, twinID) {
  if (isNaN(entityID) || isNaN(twinID)) {
    throw Error('You must pass an ID')
  }

  const twin = await getTwin(self, twinID)
  if (twin.id !== twinID) {
    throw Error(`twin with id ${twinID} does not exist`)
  }

  const entity = await getEntity(self, entityID)
  if (entity.id !== entityID) {
    throw Error(`entity with id ${entityID} does not exist`)
  }

  const arr = new ArrayBuffer(8)
  const view = new DataView(arr)
  const num = BigInt(entityID)
  view.setBigUint64(0, num, false)

  const arr1 = new ArrayBuffer(8)
  const view1 = new DataView(arr1)
  const num1 = BigInt(twinID)
  view1.setBigUint64(0, num1, false)

  const tmp = new Uint8Array(arr.byteLength + arr1.byteLength)
  tmp.set(new Uint8Array(arr), 0)
  tmp.set(new Uint8Array(arr1), arr1.byteLength)

  const message = new Uint16Array(tmp)

  const signedMessage = self.key.sign(message)

  return Buffer.from(signedMessage).toString('hex')
}

module.exports = {
  signEntityTwinID
}
