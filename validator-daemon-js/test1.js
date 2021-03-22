const StellarSdk = require('stellar-sdk')
const moment = require('moment')
const { sumBy } = require('lodash')
async function main () {
  const server = new StellarSdk.Server('https://horizon-testnet.stellar.org')

  const ex = 'month1=05/2021,48months,priceunlock=tftvalue>month*0.015+0.15'

  const [start, duration, priceUnlock] = ex.split(',')

  const x = start.split('=')[1]
  const month = parseInt(x.split('/')[0])
  const year = parseInt(x.split('/')[1])
  const startDate = moment().month(month - 1).date(1).hour(0).minute(0).second(0).millisecond(0).set('year', year)

  const transactions = await server.transactions()
    .forAccount('GBUNRT5USJ5GANTQ23HUQGDXQ2DIIVSALSOFVSF4IKXERX377G6X3DLC')
    .call()

  const operations = transactions.records.map(tx => {
    return tx.operations()
  })

  const res = await Promise.all(operations)

  // Calculate the total amount of TFT available on this account
  const total = sumBy(res, r => {
    return sumBy(r.records, record => {
      // gather all payment before the startdate of the vesting to calculate the amount of tokens that
      // can be withdrawn each month
      if (moment(record.created_at) < startDate) {
        if (record.type === 'payment' && record.to === 'GBUNRT5USJ5GANTQ23HUQGDXQ2DIIVSALSOFVSF4IKXERX377G6X3DLC' && record.asset_code === 'TFT') {
          return parseFloat(record.amount)
        }
      }
    })
  }) || 0

  // if no funds are present on the escrow account, just return
  if (!total) return Error('no funds are present on escrow account!')

  const numberOfMonths = parseInt(duration.slice(0, 2))

  const monthlyWithdrawable = total / numberOfMonths

  // calculate how much already has been withdrawn from this account
  // in order to have replay protection
  const totalWithdrawn = sumBy(res, r => {
    return sumBy(r.records, record => {
      if (moment(record.created_at) < startDate) {
        if (record.type === 'payment' && record.from === 'GBUNRT5USJ5GANTQ23HUQGDXQ2DIIVSALSOFVSF4IKXERX377G6X3DLC') {
          return parseFloat(record.amount)
        }
      }
    })
  }) || 0

  const now = moment().month(5)
  const monthsBetweenStartAndNow = now.diff(startDate, 'months')

  if (monthsBetweenStartAndNow <= 0) throw Error('cannot withdraw yet!')

  const avgTftPrice = 0.55

  const parsedPriceUnlockCondition = priceUnlock.split('*')[1]
  const [multiplier, basePrice] = parsedPriceUnlockCondition.split('+')

  const multiplierFloat = parseFloat(multiplier)
  const basePriceFloat = parseFloat(basePrice)

  // tftvalue>month*0.015+0.15

  const unlockCondition = (monthsBetweenStartAndNow * multiplierFloat) + basePriceFloat
  const canUnlock = avgTftPrice > unlockCondition

  if (canUnlock) {
    const amountWithdrawable = (monthlyWithdrawable * monthsBetweenStartAndNow) - totalWithdrawn
    console.log(`amount withdrawable: ${amountWithdrawable}`)
  } else {
    throw Error('cannot withdraw funds yet')
  }
}

main()
