const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api')

async function main() {
  const net = process.argv[2];
  const providerId = process.args[3];
  const mnemonic = process.args[4];

  let network = ''
  if (net === 'dev' || net === 'qa' || net === 'test') {
    network = net + '.'
  } else if (net === 'main') {
    network = ''
  } else {
    throw new Error('Invalid network');
  }

  const provider = new WsProvider('wss://tfchain.' + network + 'grid.tf')
  const api = await ApiPromise.create({ provider, types })

  const keyring = new Keyring()
  let key
  try {
    key = keyring.addFromMnemonic(mnemonic)
  } catch (error) {
    throw new Error('Invalid mnemonic')
  }
  
  console.log(`key with address ${key.address} loaded on ${net} network`)`)`

  const contracts = await api.query.smartContractModule.contracts.entries()
  const parsedContracts = contracts.map(c => c[1].toJSON())

  const solutionProvider = await api.query.smartContractModule.solutionProivders(providerId)
  solutionProvider = solutionProvider.toJSON()

  if (!solutionProvider.approved) {
    throw new Error('Provider is not approved')
  }

  const res = await api.query.tfgridModule.twinIdByAccountID(key.address)
  const twinId = res.toJSON()
  if (twinId === 0) {
    throw Error(`Couldn't find a twin id for this account id: ${accountId}`)
  }

  const filteredContracts = parsedContracts.filter(c => c.twinId === twinId)
  const attachCalls = filteredContracts.map(c => {
    return api.tx.smartContractModule.attachSolutionProviderId(c.id, providerId)
  })

  // Estimate the fees as RuntimeDispatchInfo, using the signer (either
  // address or locked/unlocked keypair) 
  const info = await api.tx.utility
    .batch(attachCalls)
    .paymentInfo(key);

  console.log(`estimated fees: ${info}`);

  // Construct the batch and send the transactions
  api.tx.utility
    .batch(attachCalls)
    .signAndSend(key, ({ status }) => {
      if (status.isInBlock) {
        console.log(`included in ${status.asInBlock}`);
      }
    });

  // Disconnect the API
  api.disconnect();

  console.log('done')
  process.exit(0)
}

main()
