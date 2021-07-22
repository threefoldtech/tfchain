import { Consumption } from '../../generated/graphql-server/model'
import { SmartContractModule } from '../generated/types'
import { hex2a } from './util'

import {
  EventContext,
  StoreContext,
} from '@subsquid/hydra-common'

export async function consumptionReportReceived({
  store,
  event,
  block,
  extrinsic,
}: EventContext & StoreContext) {
  const newConsumptionReport = new Consumption()
  const [consumptionReport] = new SmartContractModule.ConsumptionReportReceivedEvent(event).params

  newConsumptionReport.contractId = consumptionReport.contract_id.toNumber()
  newConsumptionReport.timestamp = consumptionReport.timestamp.toNumber()
  newConsumptionReport.cru = consumptionReport.cru.toNumber()
  newConsumptionReport.sru = consumptionReport.sru.toNumber()
  newConsumptionReport.hru = consumptionReport.hru.toNumber()
  newConsumptionReport.mru = consumptionReport.mru.toNumber()
  newConsumptionReport.nru = consumptionReport.nru.toNumber()

  await store.save<Consumption>(newConsumptionReport)
}
