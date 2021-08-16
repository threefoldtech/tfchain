import 'graphql-import-node'; // Needed so you can import *.graphql files 

import { makeBindingClass, Options } from 'graphql-binding'
import { GraphQLResolveInfo, GraphQLSchema } from 'graphql'
import { IResolvers } from 'graphql-tools/dist/Interfaces'
import * as schema from  './schema.graphql'

export interface Query {
    cities: <T = Array<City>>(args: { offset?: Int | null, limit?: Int | null, where?: CityWhereInput | null, orderBy?: Array<CityOrderByInput> | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    cityByUniqueInput: <T = City | null>(args: { where: CityWhereUniqueInput }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T | null> ,
    citiesConnection: <T = CityConnection>(args: { first?: Int | null, after?: String | null, last?: Int | null, before?: String | null, where?: CityWhereInput | null, orderBy?: Array<CityOrderByInput> | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    consumptions: <T = Array<Consumption>>(args: { offset?: Int | null, limit?: Int | null, where?: ConsumptionWhereInput | null, orderBy?: Array<ConsumptionOrderByInput> | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    consumptionByUniqueInput: <T = Consumption | null>(args: { where: ConsumptionWhereUniqueInput }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T | null> ,
    consumptionsConnection: <T = ConsumptionConnection>(args: { first?: Int | null, after?: String | null, last?: Int | null, before?: String | null, where?: ConsumptionWhereInput | null, orderBy?: Array<ConsumptionOrderByInput> | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    contractBillReports: <T = Array<ContractBillReport>>(args: { offset?: Int | null, limit?: Int | null, where?: ContractBillReportWhereInput | null, orderBy?: Array<ContractBillReportOrderByInput> | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    contractBillReportByUniqueInput: <T = ContractBillReport | null>(args: { where: ContractBillReportWhereUniqueInput }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T | null> ,
    contractBillReportsConnection: <T = ContractBillReportConnection>(args: { first?: Int | null, after?: String | null, last?: Int | null, before?: String | null, where?: ContractBillReportWhereInput | null, orderBy?: Array<ContractBillReportOrderByInput> | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    countries: <T = Array<Country>>(args: { offset?: Int | null, limit?: Int | null, where?: CountryWhereInput | null, orderBy?: Array<CountryOrderByInput> | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    countryByUniqueInput: <T = Country | null>(args: { where: CountryWhereUniqueInput }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T | null> ,
    countriesConnection: <T = CountryConnection>(args: { first?: Int | null, after?: String | null, last?: Int | null, before?: String | null, where?: CountryWhereInput | null, orderBy?: Array<CountryOrderByInput> | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    entityProofs: <T = Array<EntityProof>>(args: { offset?: Int | null, limit?: Int | null, where?: EntityProofWhereInput | null, orderBy?: Array<EntityProofOrderByInput> | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    entityProofByUniqueInput: <T = EntityProof | null>(args: { where: EntityProofWhereUniqueInput }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T | null> ,
    entityProofsConnection: <T = EntityProofConnection>(args: { first?: Int | null, after?: String | null, last?: Int | null, before?: String | null, where?: EntityProofWhereInput | null, orderBy?: Array<EntityProofOrderByInput> | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    entities: <T = Array<Entity>>(args: { offset?: Int | null, limit?: Int | null, where?: EntityWhereInput | null, orderBy?: Array<EntityOrderByInput> | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    entityByUniqueInput: <T = Entity | null>(args: { where: EntityWhereUniqueInput }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T | null> ,
    entitiesConnection: <T = EntityConnection>(args: { first?: Int | null, after?: String | null, last?: Int | null, before?: String | null, where?: EntityWhereInput | null, orderBy?: Array<EntityOrderByInput> | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    farms: <T = Array<Farm>>(args: { offset?: Int | null, limit?: Int | null, where?: FarmWhereInput | null, orderBy?: Array<FarmOrderByInput> | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    farmByUniqueInput: <T = Farm | null>(args: { where: FarmWhereUniqueInput }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T | null> ,
    farmsConnection: <T = FarmConnection>(args: { first?: Int | null, after?: String | null, last?: Int | null, before?: String | null, where?: FarmWhereInput | null, orderBy?: Array<FarmOrderByInput> | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    farmingPolicies: <T = Array<FarmingPolicy>>(args: { offset?: Int | null, limit?: Int | null, where?: FarmingPolicyWhereInput | null, orderBy?: Array<FarmingPolicyOrderByInput> | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    farmingPolicyByUniqueInput: <T = FarmingPolicy | null>(args: { where: FarmingPolicyWhereUniqueInput }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T | null> ,
    farmingPoliciesConnection: <T = FarmingPolicyConnection>(args: { first?: Int | null, after?: String | null, last?: Int | null, before?: String | null, where?: FarmingPolicyWhereInput | null, orderBy?: Array<FarmingPolicyOrderByInput> | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    locations: <T = Array<Location>>(args: { offset?: Int | null, limit?: Int | null, where?: LocationWhereInput | null, orderBy?: Array<LocationOrderByInput> | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    locationByUniqueInput: <T = Location | null>(args: { where: LocationWhereUniqueInput }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T | null> ,
    locationsConnection: <T = LocationConnection>(args: { first?: Int | null, after?: String | null, last?: Int | null, before?: String | null, where?: LocationWhereInput | null, orderBy?: Array<LocationOrderByInput> | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    nodeContracts: <T = Array<NodeContract>>(args: { offset?: Int | null, limit?: Int | null, where?: NodeContractWhereInput | null, orderBy?: Array<NodeContractOrderByInput> | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    nodeContractByUniqueInput: <T = NodeContract | null>(args: { where: NodeContractWhereUniqueInput }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T | null> ,
    nodeContractsConnection: <T = NodeContractConnection>(args: { first?: Int | null, after?: String | null, last?: Int | null, before?: String | null, where?: NodeContractWhereInput | null, orderBy?: Array<NodeContractOrderByInput> | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    nodes: <T = Array<Node>>(args: { offset?: Int | null, limit?: Int | null, where?: NodeWhereInput | null, orderBy?: Array<NodeOrderByInput> | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    nodeByUniqueInput: <T = Node | null>(args: { where: NodeWhereUniqueInput }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T | null> ,
    nodesConnection: <T = NodeConnection>(args: { first?: Int | null, after?: String | null, last?: Int | null, before?: String | null, where?: NodeWhereInput | null, orderBy?: Array<NodeOrderByInput> | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    pricingPolicies: <T = Array<PricingPolicy>>(args: { offset?: Int | null, limit?: Int | null, where?: PricingPolicyWhereInput | null, orderBy?: Array<PricingPolicyOrderByInput> | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    pricingPolicyByUniqueInput: <T = PricingPolicy | null>(args: { where: PricingPolicyWhereUniqueInput }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T | null> ,
    pricingPoliciesConnection: <T = PricingPolicyConnection>(args: { first?: Int | null, after?: String | null, last?: Int | null, before?: String | null, where?: PricingPolicyWhereInput | null, orderBy?: Array<PricingPolicyOrderByInput> | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    publicConfigs: <T = Array<PublicConfig>>(args: { offset?: Int | null, limit?: Int | null, where?: PublicConfigWhereInput | null, orderBy?: Array<PublicConfigOrderByInput> | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    publicConfigByUniqueInput: <T = PublicConfig | null>(args: { where: PublicConfigWhereUniqueInput }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T | null> ,
    publicConfigsConnection: <T = PublicConfigConnection>(args: { first?: Int | null, after?: String | null, last?: Int | null, before?: String | null, where?: PublicConfigWhereInput | null, orderBy?: Array<PublicConfigOrderByInput> | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    publicIps: <T = Array<PublicIp>>(args: { offset?: Int | null, limit?: Int | null, where?: PublicIpWhereInput | null, orderBy?: Array<PublicIpOrderByInput> | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    publicIpByUniqueInput: <T = PublicIp | null>(args: { where: PublicIpWhereUniqueInput }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T | null> ,
    publicIpsConnection: <T = PublicIpConnection>(args: { first?: Int | null, after?: String | null, last?: Int | null, before?: String | null, where?: PublicIpWhereInput | null, orderBy?: Array<PublicIpOrderByInput> | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    commentSearch: <T = Array<CommentSearchFTSOutput>>(args: { whereTransfer?: TransferWhereInput | null, skip?: Int | null, limit?: Int | null, text: String }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    transfers: <T = Array<Transfer>>(args: { offset?: Int | null, limit?: Int | null, where?: TransferWhereInput | null, orderBy?: Array<TransferOrderByInput> | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    transferByUniqueInput: <T = Transfer | null>(args: { where: TransferWhereUniqueInput }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T | null> ,
    transfersConnection: <T = TransferConnection>(args: { first?: Int | null, after?: String | null, last?: Int | null, before?: String | null, where?: TransferWhereInput | null, orderBy?: Array<TransferOrderByInput> | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    twins: <T = Array<Twin>>(args: { offset?: Int | null, limit?: Int | null, where?: TwinWhereInput | null, orderBy?: Array<TwinOrderByInput> | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    twinByUniqueInput: <T = Twin | null>(args: { where: TwinWhereUniqueInput }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T | null> ,
    twinsConnection: <T = TwinConnection>(args: { first?: Int | null, after?: String | null, last?: Int | null, before?: String | null, where?: TwinWhereInput | null, orderBy?: Array<TwinOrderByInput> | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> 
  }

export interface Mutation {}

export interface Subscription {
    stateSubscription: <T = ProcessorState>(args?: {}, info?: GraphQLResolveInfo | string, options?: Options) => Promise<AsyncIterator<T>> 
  }

export interface Binding {
  query: Query
  mutation: Mutation
  subscription: Subscription
  request: <T = any>(query: string, variables?: {[key: string]: any}) => Promise<T>
  delegate(operation: 'query' | 'mutation', fieldName: string, args: {
      [key: string]: any;
  }, infoOrQuery?: GraphQLResolveInfo | string, options?: Options): Promise<any>;
  delegateSubscription(fieldName: string, args?: {
      [key: string]: any;
  }, infoOrQuery?: GraphQLResolveInfo | string, options?: Options): Promise<AsyncIterator<any>>;
  getAbstractResolvers(filterSchema?: GraphQLSchema | string): IResolvers;
}

export interface BindingConstructor<T> {
  new(...args: any[]): T
}

export const Binding = makeBindingClass<BindingConstructor<Binding>>({ schema: schema as any })

/**
 * Types
*/

export type CertificationType =   'Diy' |
  'Certified'

export type CityOrderByInput =   'createdAt_ASC' |
  'createdAt_DESC' |
  'updatedAt_ASC' |
  'updatedAt_DESC' |
  'deletedAt_ASC' |
  'deletedAt_DESC' |
  'countryId_ASC' |
  'countryId_DESC' |
  'name_ASC' |
  'name_DESC'

export type ConsumptionOrderByInput =   'createdAt_ASC' |
  'createdAt_DESC' |
  'updatedAt_ASC' |
  'updatedAt_DESC' |
  'deletedAt_ASC' |
  'deletedAt_DESC' |
  'contractId_ASC' |
  'contractId_DESC' |
  'timestamp_ASC' |
  'timestamp_DESC' |
  'cru_ASC' |
  'cru_DESC' |
  'sru_ASC' |
  'sru_DESC' |
  'hru_ASC' |
  'hru_DESC' |
  'mru_ASC' |
  'mru_DESC' |
  'nru_ASC' |
  'nru_DESC'

export type ContractBillReportOrderByInput =   'createdAt_ASC' |
  'createdAt_DESC' |
  'updatedAt_ASC' |
  'updatedAt_DESC' |
  'deletedAt_ASC' |
  'deletedAt_DESC' |
  'contractId_ASC' |
  'contractId_DESC' |
  'discountReceived_ASC' |
  'discountReceived_DESC' |
  'amountBilled_ASC' |
  'amountBilled_DESC'

export type ContractState =   'Created' |
  'Deleted' |
  'OutOfFunds'

export type CountryOrderByInput =   'createdAt_ASC' |
  'createdAt_DESC' |
  'updatedAt_ASC' |
  'updatedAt_DESC' |
  'deletedAt_ASC' |
  'deletedAt_DESC' |
  'code_ASC' |
  'code_DESC' |
  'name_ASC' |
  'name_DESC' |
  'region_ASC' |
  'region_DESC' |
  'subregion_ASC' |
  'subregion_DESC'

export type DiscountLevel =   'None' |
  'Default' |
  'Bronze' |
  'Silver' |
  'Gold'

export type EntityOrderByInput =   'createdAt_ASC' |
  'createdAt_DESC' |
  'updatedAt_ASC' |
  'updatedAt_DESC' |
  'deletedAt_ASC' |
  'deletedAt_DESC' |
  'gridVersion_ASC' |
  'gridVersion_DESC' |
  'entityId_ASC' |
  'entityId_DESC' |
  'name_ASC' |
  'name_DESC' |
  'countryId_ASC' |
  'countryId_DESC' |
  'cityId_ASC' |
  'cityId_DESC' |
  'accountId_ASC' |
  'accountId_DESC'

export type EntityProofOrderByInput =   'createdAt_ASC' |
  'createdAt_DESC' |
  'updatedAt_ASC' |
  'updatedAt_DESC' |
  'deletedAt_ASC' |
  'deletedAt_DESC' |
  'entityId_ASC' |
  'entityId_DESC' |
  'signature_ASC' |
  'signature_DESC' |
  'twinRel_ASC' |
  'twinRel_DESC'

export type FarmingPolicyOrderByInput =   'createdAt_ASC' |
  'createdAt_DESC' |
  'updatedAt_ASC' |
  'updatedAt_DESC' |
  'deletedAt_ASC' |
  'deletedAt_DESC' |
  'version_ASC' |
  'version_DESC' |
  'name_ASC' |
  'name_DESC' |
  'cu_ASC' |
  'cu_DESC' |
  'su_ASC' |
  'su_DESC' |
  'nu_ASC' |
  'nu_DESC' |
  'ipv4_ASC' |
  'ipv4_DESC' |
  'timestamp_ASC' |
  'timestamp_DESC' |
  'certificationType_ASC' |
  'certificationType_DESC'

export type FarmOrderByInput =   'createdAt_ASC' |
  'createdAt_DESC' |
  'updatedAt_ASC' |
  'updatedAt_DESC' |
  'deletedAt_ASC' |
  'deletedAt_DESC' |
  'gridVersion_ASC' |
  'gridVersion_DESC' |
  'farmId_ASC' |
  'farmId_DESC' |
  'name_ASC' |
  'name_DESC' |
  'twinId_ASC' |
  'twinId_DESC' |
  'pricingPolicyId_ASC' |
  'pricingPolicyId_DESC' |
  'certificationType_ASC' |
  'certificationType_DESC' |
  'countryId_ASC' |
  'countryId_DESC' |
  'cityId_ASC' |
  'cityId_DESC'

export type LocationOrderByInput =   'createdAt_ASC' |
  'createdAt_DESC' |
  'updatedAt_ASC' |
  'updatedAt_DESC' |
  'deletedAt_ASC' |
  'deletedAt_DESC' |
  'longitude_ASC' |
  'longitude_DESC' |
  'latitude_ASC' |
  'latitude_DESC'

export type NodeContractOrderByInput =   'createdAt_ASC' |
  'createdAt_DESC' |
  'updatedAt_ASC' |
  'updatedAt_DESC' |
  'deletedAt_ASC' |
  'deletedAt_DESC' |
  'version_ASC' |
  'version_DESC' |
  'contractId_ASC' |
  'contractId_DESC' |
  'twinId_ASC' |
  'twinId_DESC' |
  'nodeId_ASC' |
  'nodeId_DESC' |
  'deploymentData_ASC' |
  'deploymentData_DESC' |
  'deploymentHash_ASC' |
  'deploymentHash_DESC' |
  'numberOfPublicIPs_ASC' |
  'numberOfPublicIPs_DESC' |
  'state_ASC' |
  'state_DESC'

export type NodeOrderByInput =   'createdAt_ASC' |
  'createdAt_DESC' |
  'updatedAt_ASC' |
  'updatedAt_DESC' |
  'deletedAt_ASC' |
  'deletedAt_DESC' |
  'gridVersion_ASC' |
  'gridVersion_DESC' |
  'nodeId_ASC' |
  'nodeId_DESC' |
  'farmId_ASC' |
  'farmId_DESC' |
  'twinId_ASC' |
  'twinId_DESC' |
  'location_ASC' |
  'location_DESC' |
  'countryId_ASC' |
  'countryId_DESC' |
  'cityId_ASC' |
  'cityId_DESC' |
  'hru_ASC' |
  'hru_DESC' |
  'sru_ASC' |
  'sru_DESC' |
  'cru_ASC' |
  'cru_DESC' |
  'mru_ASC' |
  'mru_DESC' |
  'publicConfig_ASC' |
  'publicConfig_DESC' |
  'uptime_ASC' |
  'uptime_DESC' |
  'created_ASC' |
  'created_DESC' |
  'farmingPolicyId_ASC' |
  'farmingPolicyId_DESC'

export type PricingPolicyOrderByInput =   'createdAt_ASC' |
  'createdAt_DESC' |
  'updatedAt_ASC' |
  'updatedAt_DESC' |
  'deletedAt_ASC' |
  'deletedAt_DESC' |
  'gridVersion_ASC' |
  'gridVersion_DESC' |
  'pricingPolicyId_ASC' |
  'pricingPolicyId_DESC' |
  'name_ASC' |
  'name_DESC' |
  'currency_ASC' |
  'currency_DESC' |
  'su_ASC' |
  'su_DESC' |
  'cu_ASC' |
  'cu_DESC' |
  'nu_ASC' |
  'nu_DESC' |
  'foundationAccount_ASC' |
  'foundationAccount_DESC' |
  'certifiedSalesAccount_ASC' |
  'certifiedSalesAccount_DESC'

export type PublicConfigOrderByInput =   'createdAt_ASC' |
  'createdAt_DESC' |
  'updatedAt_ASC' |
  'updatedAt_DESC' |
  'deletedAt_ASC' |
  'deletedAt_DESC' |
  'ipv4_ASC' |
  'ipv4_DESC' |
  'ipv6_ASC' |
  'ipv6_DESC' |
  'gw4_ASC' |
  'gw4_DESC' |
  'gw6_ASC' |
  'gw6_DESC'

export type PublicIpOrderByInput =   'createdAt_ASC' |
  'createdAt_DESC' |
  'updatedAt_ASC' |
  'updatedAt_DESC' |
  'deletedAt_ASC' |
  'deletedAt_DESC' |
  'farm_ASC' |
  'farm_DESC' |
  'gateway_ASC' |
  'gateway_DESC' |
  'ip_ASC' |
  'ip_DESC' |
  'contractId_ASC' |
  'contractId_DESC'

export type TransferOrderByInput =   'createdAt_ASC' |
  'createdAt_DESC' |
  'updatedAt_ASC' |
  'updatedAt_DESC' |
  'deletedAt_ASC' |
  'deletedAt_DESC' |
  'from_ASC' |
  'from_DESC' |
  'to_ASC' |
  'to_DESC' |
  'value_ASC' |
  'value_DESC' |
  'comment_ASC' |
  'comment_DESC' |
  'block_ASC' |
  'block_DESC'

export type TwinOrderByInput =   'createdAt_ASC' |
  'createdAt_DESC' |
  'updatedAt_ASC' |
  'updatedAt_DESC' |
  'deletedAt_ASC' |
  'deletedAt_DESC' |
  'gridVersion_ASC' |
  'gridVersion_DESC' |
  'twinId_ASC' |
  'twinId_DESC' |
  'accountId_ASC' |
  'accountId_DESC' |
  'ip_ASC' |
  'ip_DESC'

export interface BaseWhereInput {
  id_eq?: String | null
  id_in?: String[] | String | null
  createdAt_eq?: String | null
  createdAt_lt?: String | null
  createdAt_lte?: String | null
  createdAt_gt?: String | null
  createdAt_gte?: String | null
  createdById_eq?: String | null
  updatedAt_eq?: String | null
  updatedAt_lt?: String | null
  updatedAt_lte?: String | null
  updatedAt_gt?: String | null
  updatedAt_gte?: String | null
  updatedById_eq?: String | null
  deletedAt_all?: Boolean | null
  deletedAt_eq?: String | null
  deletedAt_lt?: String | null
  deletedAt_lte?: String | null
  deletedAt_gt?: String | null
  deletedAt_gte?: String | null
  deletedById_eq?: String | null
}

export interface CityCreateInput {
  countryId: Float
  name: String
}

export interface CityUpdateInput {
  countryId?: Float | null
  name?: String | null
}

export interface CityWhereInput {
  id_eq?: ID_Input | null
  id_in?: ID_Output[] | ID_Output | null
  createdAt_eq?: DateTime | null
  createdAt_lt?: DateTime | null
  createdAt_lte?: DateTime | null
  createdAt_gt?: DateTime | null
  createdAt_gte?: DateTime | null
  createdById_eq?: ID_Input | null
  createdById_in?: ID_Output[] | ID_Output | null
  updatedAt_eq?: DateTime | null
  updatedAt_lt?: DateTime | null
  updatedAt_lte?: DateTime | null
  updatedAt_gt?: DateTime | null
  updatedAt_gte?: DateTime | null
  updatedById_eq?: ID_Input | null
  updatedById_in?: ID_Output[] | ID_Output | null
  deletedAt_all?: Boolean | null
  deletedAt_eq?: DateTime | null
  deletedAt_lt?: DateTime | null
  deletedAt_lte?: DateTime | null
  deletedAt_gt?: DateTime | null
  deletedAt_gte?: DateTime | null
  deletedById_eq?: ID_Input | null
  deletedById_in?: ID_Output[] | ID_Output | null
  countryId_eq?: Int | null
  countryId_gt?: Int | null
  countryId_gte?: Int | null
  countryId_lt?: Int | null
  countryId_lte?: Int | null
  countryId_in?: Int[] | Int | null
  name_eq?: String | null
  name_contains?: String | null
  name_startsWith?: String | null
  name_endsWith?: String | null
  name_in?: String[] | String | null
  AND?: CityWhereInput[] | CityWhereInput | null
  OR?: CityWhereInput[] | CityWhereInput | null
}

export interface CityWhereUniqueInput {
  id: ID_Output
}

export interface ConsumptionCreateInput {
  contractId: Float
  timestamp: Float
  cru?: BigInt | null
  sru?: BigInt | null
  hru?: BigInt | null
  mru?: BigInt | null
  nru?: BigInt | null
}

export interface ConsumptionUpdateInput {
  contractId?: Float | null
  timestamp?: Float | null
  cru?: BigInt | null
  sru?: BigInt | null
  hru?: BigInt | null
  mru?: BigInt | null
  nru?: BigInt | null
}

export interface ConsumptionWhereInput {
  id_eq?: ID_Input | null
  id_in?: ID_Output[] | ID_Output | null
  createdAt_eq?: DateTime | null
  createdAt_lt?: DateTime | null
  createdAt_lte?: DateTime | null
  createdAt_gt?: DateTime | null
  createdAt_gte?: DateTime | null
  createdById_eq?: ID_Input | null
  createdById_in?: ID_Output[] | ID_Output | null
  updatedAt_eq?: DateTime | null
  updatedAt_lt?: DateTime | null
  updatedAt_lte?: DateTime | null
  updatedAt_gt?: DateTime | null
  updatedAt_gte?: DateTime | null
  updatedById_eq?: ID_Input | null
  updatedById_in?: ID_Output[] | ID_Output | null
  deletedAt_all?: Boolean | null
  deletedAt_eq?: DateTime | null
  deletedAt_lt?: DateTime | null
  deletedAt_lte?: DateTime | null
  deletedAt_gt?: DateTime | null
  deletedAt_gte?: DateTime | null
  deletedById_eq?: ID_Input | null
  deletedById_in?: ID_Output[] | ID_Output | null
  contractId_eq?: Int | null
  contractId_gt?: Int | null
  contractId_gte?: Int | null
  contractId_lt?: Int | null
  contractId_lte?: Int | null
  contractId_in?: Int[] | Int | null
  timestamp_eq?: Int | null
  timestamp_gt?: Int | null
  timestamp_gte?: Int | null
  timestamp_lt?: Int | null
  timestamp_lte?: Int | null
  timestamp_in?: Int[] | Int | null
  cru_eq?: BigInt | null
  cru_gt?: BigInt | null
  cru_gte?: BigInt | null
  cru_lt?: BigInt | null
  cru_lte?: BigInt | null
  cru_in?: BigInt[] | BigInt | null
  sru_eq?: BigInt | null
  sru_gt?: BigInt | null
  sru_gte?: BigInt | null
  sru_lt?: BigInt | null
  sru_lte?: BigInt | null
  sru_in?: BigInt[] | BigInt | null
  hru_eq?: BigInt | null
  hru_gt?: BigInt | null
  hru_gte?: BigInt | null
  hru_lt?: BigInt | null
  hru_lte?: BigInt | null
  hru_in?: BigInt[] | BigInt | null
  mru_eq?: BigInt | null
  mru_gt?: BigInt | null
  mru_gte?: BigInt | null
  mru_lt?: BigInt | null
  mru_lte?: BigInt | null
  mru_in?: BigInt[] | BigInt | null
  nru_eq?: BigInt | null
  nru_gt?: BigInt | null
  nru_gte?: BigInt | null
  nru_lt?: BigInt | null
  nru_lte?: BigInt | null
  nru_in?: BigInt[] | BigInt | null
  AND?: ConsumptionWhereInput[] | ConsumptionWhereInput | null
  OR?: ConsumptionWhereInput[] | ConsumptionWhereInput | null
}

export interface ConsumptionWhereUniqueInput {
  id: ID_Output
}

export interface ContractBillReportCreateInput {
  contractId: Float
  discountReceived: DiscountLevel
  amountBilled: Float
}

export interface ContractBillReportUpdateInput {
  contractId?: Float | null
  discountReceived?: DiscountLevel | null
  amountBilled?: Float | null
}

export interface ContractBillReportWhereInput {
  id_eq?: ID_Input | null
  id_in?: ID_Output[] | ID_Output | null
  createdAt_eq?: DateTime | null
  createdAt_lt?: DateTime | null
  createdAt_lte?: DateTime | null
  createdAt_gt?: DateTime | null
  createdAt_gte?: DateTime | null
  createdById_eq?: ID_Input | null
  createdById_in?: ID_Output[] | ID_Output | null
  updatedAt_eq?: DateTime | null
  updatedAt_lt?: DateTime | null
  updatedAt_lte?: DateTime | null
  updatedAt_gt?: DateTime | null
  updatedAt_gte?: DateTime | null
  updatedById_eq?: ID_Input | null
  updatedById_in?: ID_Output[] | ID_Output | null
  deletedAt_all?: Boolean | null
  deletedAt_eq?: DateTime | null
  deletedAt_lt?: DateTime | null
  deletedAt_lte?: DateTime | null
  deletedAt_gt?: DateTime | null
  deletedAt_gte?: DateTime | null
  deletedById_eq?: ID_Input | null
  deletedById_in?: ID_Output[] | ID_Output | null
  contractId_eq?: Int | null
  contractId_gt?: Int | null
  contractId_gte?: Int | null
  contractId_lt?: Int | null
  contractId_lte?: Int | null
  contractId_in?: Int[] | Int | null
  discountReceived_eq?: DiscountLevel | null
  discountReceived_in?: DiscountLevel[] | DiscountLevel | null
  amountBilled_eq?: Int | null
  amountBilled_gt?: Int | null
  amountBilled_gte?: Int | null
  amountBilled_lt?: Int | null
  amountBilled_lte?: Int | null
  amountBilled_in?: Int[] | Int | null
  AND?: ContractBillReportWhereInput[] | ContractBillReportWhereInput | null
  OR?: ContractBillReportWhereInput[] | ContractBillReportWhereInput | null
}

export interface ContractBillReportWhereUniqueInput {
  id: ID_Output
}

export interface CountryCreateInput {
  code: String
  name: String
  region: String
  subregion: String
}

export interface CountryUpdateInput {
  code?: String | null
  name?: String | null
  region?: String | null
  subregion?: String | null
}

export interface CountryWhereInput {
  id_eq?: ID_Input | null
  id_in?: ID_Output[] | ID_Output | null
  createdAt_eq?: DateTime | null
  createdAt_lt?: DateTime | null
  createdAt_lte?: DateTime | null
  createdAt_gt?: DateTime | null
  createdAt_gte?: DateTime | null
  createdById_eq?: ID_Input | null
  createdById_in?: ID_Output[] | ID_Output | null
  updatedAt_eq?: DateTime | null
  updatedAt_lt?: DateTime | null
  updatedAt_lte?: DateTime | null
  updatedAt_gt?: DateTime | null
  updatedAt_gte?: DateTime | null
  updatedById_eq?: ID_Input | null
  updatedById_in?: ID_Output[] | ID_Output | null
  deletedAt_all?: Boolean | null
  deletedAt_eq?: DateTime | null
  deletedAt_lt?: DateTime | null
  deletedAt_lte?: DateTime | null
  deletedAt_gt?: DateTime | null
  deletedAt_gte?: DateTime | null
  deletedById_eq?: ID_Input | null
  deletedById_in?: ID_Output[] | ID_Output | null
  code_eq?: String | null
  code_contains?: String | null
  code_startsWith?: String | null
  code_endsWith?: String | null
  code_in?: String[] | String | null
  name_eq?: String | null
  name_contains?: String | null
  name_startsWith?: String | null
  name_endsWith?: String | null
  name_in?: String[] | String | null
  region_eq?: String | null
  region_contains?: String | null
  region_startsWith?: String | null
  region_endsWith?: String | null
  region_in?: String[] | String | null
  subregion_eq?: String | null
  subregion_contains?: String | null
  subregion_startsWith?: String | null
  subregion_endsWith?: String | null
  subregion_in?: String[] | String | null
  AND?: CountryWhereInput[] | CountryWhereInput | null
  OR?: CountryWhereInput[] | CountryWhereInput | null
}

export interface CountryWhereUniqueInput {
  id: ID_Output
}

export interface EntityCreateInput {
  gridVersion: Float
  entityId: Float
  name: String
  countryId?: Float | null
  cityId?: Float | null
  accountId: String
}

export interface EntityProofCreateInput {
  entityId: Float
  signature: String
  twinRel: ID_Output
}

export interface EntityProofUpdateInput {
  entityId?: Float | null
  signature?: String | null
  twinRel?: ID_Input | null
}

export interface EntityProofWhereInput {
  id_eq?: ID_Input | null
  id_in?: ID_Output[] | ID_Output | null
  createdAt_eq?: DateTime | null
  createdAt_lt?: DateTime | null
  createdAt_lte?: DateTime | null
  createdAt_gt?: DateTime | null
  createdAt_gte?: DateTime | null
  createdById_eq?: ID_Input | null
  createdById_in?: ID_Output[] | ID_Output | null
  updatedAt_eq?: DateTime | null
  updatedAt_lt?: DateTime | null
  updatedAt_lte?: DateTime | null
  updatedAt_gt?: DateTime | null
  updatedAt_gte?: DateTime | null
  updatedById_eq?: ID_Input | null
  updatedById_in?: ID_Output[] | ID_Output | null
  deletedAt_all?: Boolean | null
  deletedAt_eq?: DateTime | null
  deletedAt_lt?: DateTime | null
  deletedAt_lte?: DateTime | null
  deletedAt_gt?: DateTime | null
  deletedAt_gte?: DateTime | null
  deletedById_eq?: ID_Input | null
  deletedById_in?: ID_Output[] | ID_Output | null
  entityId_eq?: Int | null
  entityId_gt?: Int | null
  entityId_gte?: Int | null
  entityId_lt?: Int | null
  entityId_lte?: Int | null
  entityId_in?: Int[] | Int | null
  signature_eq?: String | null
  signature_contains?: String | null
  signature_startsWith?: String | null
  signature_endsWith?: String | null
  signature_in?: String[] | String | null
  twinRel_eq?: ID_Input | null
  twinRel_in?: ID_Output[] | ID_Output | null
  twinRel?: TwinWhereInput | null
  AND?: EntityProofWhereInput[] | EntityProofWhereInput | null
  OR?: EntityProofWhereInput[] | EntityProofWhereInput | null
}

export interface EntityProofWhereUniqueInput {
  id: ID_Output
}

export interface EntityUpdateInput {
  gridVersion?: Float | null
  entityId?: Float | null
  name?: String | null
  countryId?: Float | null
  cityId?: Float | null
  accountId?: String | null
}

export interface EntityWhereInput {
  id_eq?: ID_Input | null
  id_in?: ID_Output[] | ID_Output | null
  createdAt_eq?: DateTime | null
  createdAt_lt?: DateTime | null
  createdAt_lte?: DateTime | null
  createdAt_gt?: DateTime | null
  createdAt_gte?: DateTime | null
  createdById_eq?: ID_Input | null
  createdById_in?: ID_Output[] | ID_Output | null
  updatedAt_eq?: DateTime | null
  updatedAt_lt?: DateTime | null
  updatedAt_lte?: DateTime | null
  updatedAt_gt?: DateTime | null
  updatedAt_gte?: DateTime | null
  updatedById_eq?: ID_Input | null
  updatedById_in?: ID_Output[] | ID_Output | null
  deletedAt_all?: Boolean | null
  deletedAt_eq?: DateTime | null
  deletedAt_lt?: DateTime | null
  deletedAt_lte?: DateTime | null
  deletedAt_gt?: DateTime | null
  deletedAt_gte?: DateTime | null
  deletedById_eq?: ID_Input | null
  deletedById_in?: ID_Output[] | ID_Output | null
  gridVersion_eq?: Int | null
  gridVersion_gt?: Int | null
  gridVersion_gte?: Int | null
  gridVersion_lt?: Int | null
  gridVersion_lte?: Int | null
  gridVersion_in?: Int[] | Int | null
  entityId_eq?: Int | null
  entityId_gt?: Int | null
  entityId_gte?: Int | null
  entityId_lt?: Int | null
  entityId_lte?: Int | null
  entityId_in?: Int[] | Int | null
  name_eq?: String | null
  name_contains?: String | null
  name_startsWith?: String | null
  name_endsWith?: String | null
  name_in?: String[] | String | null
  countryId_eq?: Int | null
  countryId_gt?: Int | null
  countryId_gte?: Int | null
  countryId_lt?: Int | null
  countryId_lte?: Int | null
  countryId_in?: Int[] | Int | null
  cityId_eq?: Int | null
  cityId_gt?: Int | null
  cityId_gte?: Int | null
  cityId_lt?: Int | null
  cityId_lte?: Int | null
  cityId_in?: Int[] | Int | null
  accountId_eq?: String | null
  accountId_contains?: String | null
  accountId_startsWith?: String | null
  accountId_endsWith?: String | null
  accountId_in?: String[] | String | null
  AND?: EntityWhereInput[] | EntityWhereInput | null
  OR?: EntityWhereInput[] | EntityWhereInput | null
}

export interface EntityWhereUniqueInput {
  id: ID_Output
}

export interface FarmCreateInput {
  gridVersion: Float
  farmId: Float
  name: String
  twinId: Float
  pricingPolicyId: Float
  certificationType: CertificationType
  countryId?: Float | null
  cityId?: Float | null
}

export interface FarmingPolicyCreateInput {
  version: Float
  name: String
  cu: Float
  su: Float
  nu: Float
  ipv4: Float
  timestamp: Float
  certificationType: CertificationType
}

export interface FarmingPolicyUpdateInput {
  version?: Float | null
  name?: String | null
  cu?: Float | null
  su?: Float | null
  nu?: Float | null
  ipv4?: Float | null
  timestamp?: Float | null
  certificationType?: CertificationType | null
}

export interface FarmingPolicyWhereInput {
  id_eq?: ID_Input | null
  id_in?: ID_Output[] | ID_Output | null
  createdAt_eq?: DateTime | null
  createdAt_lt?: DateTime | null
  createdAt_lte?: DateTime | null
  createdAt_gt?: DateTime | null
  createdAt_gte?: DateTime | null
  createdById_eq?: ID_Input | null
  createdById_in?: ID_Output[] | ID_Output | null
  updatedAt_eq?: DateTime | null
  updatedAt_lt?: DateTime | null
  updatedAt_lte?: DateTime | null
  updatedAt_gt?: DateTime | null
  updatedAt_gte?: DateTime | null
  updatedById_eq?: ID_Input | null
  updatedById_in?: ID_Output[] | ID_Output | null
  deletedAt_all?: Boolean | null
  deletedAt_eq?: DateTime | null
  deletedAt_lt?: DateTime | null
  deletedAt_lte?: DateTime | null
  deletedAt_gt?: DateTime | null
  deletedAt_gte?: DateTime | null
  deletedById_eq?: ID_Input | null
  deletedById_in?: ID_Output[] | ID_Output | null
  version_eq?: Int | null
  version_gt?: Int | null
  version_gte?: Int | null
  version_lt?: Int | null
  version_lte?: Int | null
  version_in?: Int[] | Int | null
  name_eq?: String | null
  name_contains?: String | null
  name_startsWith?: String | null
  name_endsWith?: String | null
  name_in?: String[] | String | null
  cu_eq?: Int | null
  cu_gt?: Int | null
  cu_gte?: Int | null
  cu_lt?: Int | null
  cu_lte?: Int | null
  cu_in?: Int[] | Int | null
  su_eq?: Int | null
  su_gt?: Int | null
  su_gte?: Int | null
  su_lt?: Int | null
  su_lte?: Int | null
  su_in?: Int[] | Int | null
  nu_eq?: Int | null
  nu_gt?: Int | null
  nu_gte?: Int | null
  nu_lt?: Int | null
  nu_lte?: Int | null
  nu_in?: Int[] | Int | null
  ipv4_eq?: Int | null
  ipv4_gt?: Int | null
  ipv4_gte?: Int | null
  ipv4_lt?: Int | null
  ipv4_lte?: Int | null
  ipv4_in?: Int[] | Int | null
  timestamp_eq?: Int | null
  timestamp_gt?: Int | null
  timestamp_gte?: Int | null
  timestamp_lt?: Int | null
  timestamp_lte?: Int | null
  timestamp_in?: Int[] | Int | null
  certificationType_eq?: CertificationType | null
  certificationType_in?: CertificationType[] | CertificationType | null
  AND?: FarmingPolicyWhereInput[] | FarmingPolicyWhereInput | null
  OR?: FarmingPolicyWhereInput[] | FarmingPolicyWhereInput | null
}

export interface FarmingPolicyWhereUniqueInput {
  id: ID_Output
}

export interface FarmUpdateInput {
  gridVersion?: Float | null
  farmId?: Float | null
  name?: String | null
  twinId?: Float | null
  pricingPolicyId?: Float | null
  certificationType?: CertificationType | null
  countryId?: Float | null
  cityId?: Float | null
}

export interface FarmWhereInput {
  id_eq?: ID_Input | null
  id_in?: ID_Output[] | ID_Output | null
  createdAt_eq?: DateTime | null
  createdAt_lt?: DateTime | null
  createdAt_lte?: DateTime | null
  createdAt_gt?: DateTime | null
  createdAt_gte?: DateTime | null
  createdById_eq?: ID_Input | null
  createdById_in?: ID_Output[] | ID_Output | null
  updatedAt_eq?: DateTime | null
  updatedAt_lt?: DateTime | null
  updatedAt_lte?: DateTime | null
  updatedAt_gt?: DateTime | null
  updatedAt_gte?: DateTime | null
  updatedById_eq?: ID_Input | null
  updatedById_in?: ID_Output[] | ID_Output | null
  deletedAt_all?: Boolean | null
  deletedAt_eq?: DateTime | null
  deletedAt_lt?: DateTime | null
  deletedAt_lte?: DateTime | null
  deletedAt_gt?: DateTime | null
  deletedAt_gte?: DateTime | null
  deletedById_eq?: ID_Input | null
  deletedById_in?: ID_Output[] | ID_Output | null
  gridVersion_eq?: Int | null
  gridVersion_gt?: Int | null
  gridVersion_gte?: Int | null
  gridVersion_lt?: Int | null
  gridVersion_lte?: Int | null
  gridVersion_in?: Int[] | Int | null
  farmId_eq?: Int | null
  farmId_gt?: Int | null
  farmId_gte?: Int | null
  farmId_lt?: Int | null
  farmId_lte?: Int | null
  farmId_in?: Int[] | Int | null
  name_eq?: String | null
  name_contains?: String | null
  name_startsWith?: String | null
  name_endsWith?: String | null
  name_in?: String[] | String | null
  twinId_eq?: Int | null
  twinId_gt?: Int | null
  twinId_gte?: Int | null
  twinId_lt?: Int | null
  twinId_lte?: Int | null
  twinId_in?: Int[] | Int | null
  pricingPolicyId_eq?: Int | null
  pricingPolicyId_gt?: Int | null
  pricingPolicyId_gte?: Int | null
  pricingPolicyId_lt?: Int | null
  pricingPolicyId_lte?: Int | null
  pricingPolicyId_in?: Int[] | Int | null
  certificationType_eq?: CertificationType | null
  certificationType_in?: CertificationType[] | CertificationType | null
  countryId_eq?: Int | null
  countryId_gt?: Int | null
  countryId_gte?: Int | null
  countryId_lt?: Int | null
  countryId_lte?: Int | null
  countryId_in?: Int[] | Int | null
  cityId_eq?: Int | null
  cityId_gt?: Int | null
  cityId_gte?: Int | null
  cityId_lt?: Int | null
  cityId_lte?: Int | null
  cityId_in?: Int[] | Int | null
  publicIPs_none?: PublicIpWhereInput | null
  publicIPs_some?: PublicIpWhereInput | null
  publicIPs_every?: PublicIpWhereInput | null
  AND?: FarmWhereInput[] | FarmWhereInput | null
  OR?: FarmWhereInput[] | FarmWhereInput | null
}

export interface FarmWhereUniqueInput {
  id: ID_Output
}

export interface LocationCreateInput {
  longitude: String
  latitude: String
}

export interface LocationUpdateInput {
  longitude?: String | null
  latitude?: String | null
}

export interface LocationWhereInput {
  id_eq?: ID_Input | null
  id_in?: ID_Output[] | ID_Output | null
  createdAt_eq?: DateTime | null
  createdAt_lt?: DateTime | null
  createdAt_lte?: DateTime | null
  createdAt_gt?: DateTime | null
  createdAt_gte?: DateTime | null
  createdById_eq?: ID_Input | null
  createdById_in?: ID_Output[] | ID_Output | null
  updatedAt_eq?: DateTime | null
  updatedAt_lt?: DateTime | null
  updatedAt_lte?: DateTime | null
  updatedAt_gt?: DateTime | null
  updatedAt_gte?: DateTime | null
  updatedById_eq?: ID_Input | null
  updatedById_in?: ID_Output[] | ID_Output | null
  deletedAt_all?: Boolean | null
  deletedAt_eq?: DateTime | null
  deletedAt_lt?: DateTime | null
  deletedAt_lte?: DateTime | null
  deletedAt_gt?: DateTime | null
  deletedAt_gte?: DateTime | null
  deletedById_eq?: ID_Input | null
  deletedById_in?: ID_Output[] | ID_Output | null
  longitude_eq?: String | null
  longitude_contains?: String | null
  longitude_startsWith?: String | null
  longitude_endsWith?: String | null
  longitude_in?: String[] | String | null
  latitude_eq?: String | null
  latitude_contains?: String | null
  latitude_startsWith?: String | null
  latitude_endsWith?: String | null
  latitude_in?: String[] | String | null
  nodelocation_none?: NodeWhereInput | null
  nodelocation_some?: NodeWhereInput | null
  nodelocation_every?: NodeWhereInput | null
  AND?: LocationWhereInput[] | LocationWhereInput | null
  OR?: LocationWhereInput[] | LocationWhereInput | null
}

export interface LocationWhereUniqueInput {
  id: ID_Output
}

export interface NodeContractCreateInput {
  version: Float
  contractId: Float
  twinId: Float
  nodeId: Float
  deploymentData: String
  deploymentHash: String
  numberOfPublicIPs: Float
  state: ContractState
}

export interface NodeContractUpdateInput {
  version?: Float | null
  contractId?: Float | null
  twinId?: Float | null
  nodeId?: Float | null
  deploymentData?: String | null
  deploymentHash?: String | null
  numberOfPublicIPs?: Float | null
  state?: ContractState | null
}

export interface NodeContractWhereInput {
  id_eq?: ID_Input | null
  id_in?: ID_Output[] | ID_Output | null
  createdAt_eq?: DateTime | null
  createdAt_lt?: DateTime | null
  createdAt_lte?: DateTime | null
  createdAt_gt?: DateTime | null
  createdAt_gte?: DateTime | null
  createdById_eq?: ID_Input | null
  createdById_in?: ID_Output[] | ID_Output | null
  updatedAt_eq?: DateTime | null
  updatedAt_lt?: DateTime | null
  updatedAt_lte?: DateTime | null
  updatedAt_gt?: DateTime | null
  updatedAt_gte?: DateTime | null
  updatedById_eq?: ID_Input | null
  updatedById_in?: ID_Output[] | ID_Output | null
  deletedAt_all?: Boolean | null
  deletedAt_eq?: DateTime | null
  deletedAt_lt?: DateTime | null
  deletedAt_lte?: DateTime | null
  deletedAt_gt?: DateTime | null
  deletedAt_gte?: DateTime | null
  deletedById_eq?: ID_Input | null
  deletedById_in?: ID_Output[] | ID_Output | null
  version_eq?: Int | null
  version_gt?: Int | null
  version_gte?: Int | null
  version_lt?: Int | null
  version_lte?: Int | null
  version_in?: Int[] | Int | null
  contractId_eq?: Int | null
  contractId_gt?: Int | null
  contractId_gte?: Int | null
  contractId_lt?: Int | null
  contractId_lte?: Int | null
  contractId_in?: Int[] | Int | null
  twinId_eq?: Int | null
  twinId_gt?: Int | null
  twinId_gte?: Int | null
  twinId_lt?: Int | null
  twinId_lte?: Int | null
  twinId_in?: Int[] | Int | null
  nodeId_eq?: Int | null
  nodeId_gt?: Int | null
  nodeId_gte?: Int | null
  nodeId_lt?: Int | null
  nodeId_lte?: Int | null
  nodeId_in?: Int[] | Int | null
  deploymentData_eq?: String | null
  deploymentData_contains?: String | null
  deploymentData_startsWith?: String | null
  deploymentData_endsWith?: String | null
  deploymentData_in?: String[] | String | null
  deploymentHash_eq?: String | null
  deploymentHash_contains?: String | null
  deploymentHash_startsWith?: String | null
  deploymentHash_endsWith?: String | null
  deploymentHash_in?: String[] | String | null
  numberOfPublicIPs_eq?: Int | null
  numberOfPublicIPs_gt?: Int | null
  numberOfPublicIPs_gte?: Int | null
  numberOfPublicIPs_lt?: Int | null
  numberOfPublicIPs_lte?: Int | null
  numberOfPublicIPs_in?: Int[] | Int | null
  state_eq?: ContractState | null
  state_in?: ContractState[] | ContractState | null
  AND?: NodeContractWhereInput[] | NodeContractWhereInput | null
  OR?: NodeContractWhereInput[] | NodeContractWhereInput | null
}

export interface NodeContractWhereUniqueInput {
  id: ID_Output
}

export interface NodeCreateInput {
  gridVersion: Float
  nodeId: Float
  farmId: Float
  twinId: Float
  location: ID_Output
  countryId?: Float | null
  cityId?: Float | null
  hru?: BigInt | null
  sru?: BigInt | null
  cru?: BigInt | null
  mru?: BigInt | null
  publicConfig?: ID_Input | null
  uptime?: Float | null
  created: Float
  farmingPolicyId: Float
}

export interface NodeUpdateInput {
  gridVersion?: Float | null
  nodeId?: Float | null
  farmId?: Float | null
  twinId?: Float | null
  location?: ID_Input | null
  countryId?: Float | null
  cityId?: Float | null
  hru?: BigInt | null
  sru?: BigInt | null
  cru?: BigInt | null
  mru?: BigInt | null
  publicConfig?: ID_Input | null
  uptime?: Float | null
  created?: Float | null
  farmingPolicyId?: Float | null
}

export interface NodeWhereInput {
  id_eq?: ID_Input | null
  id_in?: ID_Output[] | ID_Output | null
  createdAt_eq?: DateTime | null
  createdAt_lt?: DateTime | null
  createdAt_lte?: DateTime | null
  createdAt_gt?: DateTime | null
  createdAt_gte?: DateTime | null
  createdById_eq?: ID_Input | null
  createdById_in?: ID_Output[] | ID_Output | null
  updatedAt_eq?: DateTime | null
  updatedAt_lt?: DateTime | null
  updatedAt_lte?: DateTime | null
  updatedAt_gt?: DateTime | null
  updatedAt_gte?: DateTime | null
  updatedById_eq?: ID_Input | null
  updatedById_in?: ID_Output[] | ID_Output | null
  deletedAt_all?: Boolean | null
  deletedAt_eq?: DateTime | null
  deletedAt_lt?: DateTime | null
  deletedAt_lte?: DateTime | null
  deletedAt_gt?: DateTime | null
  deletedAt_gte?: DateTime | null
  deletedById_eq?: ID_Input | null
  deletedById_in?: ID_Output[] | ID_Output | null
  gridVersion_eq?: Int | null
  gridVersion_gt?: Int | null
  gridVersion_gte?: Int | null
  gridVersion_lt?: Int | null
  gridVersion_lte?: Int | null
  gridVersion_in?: Int[] | Int | null
  nodeId_eq?: Int | null
  nodeId_gt?: Int | null
  nodeId_gte?: Int | null
  nodeId_lt?: Int | null
  nodeId_lte?: Int | null
  nodeId_in?: Int[] | Int | null
  farmId_eq?: Int | null
  farmId_gt?: Int | null
  farmId_gte?: Int | null
  farmId_lt?: Int | null
  farmId_lte?: Int | null
  farmId_in?: Int[] | Int | null
  twinId_eq?: Int | null
  twinId_gt?: Int | null
  twinId_gte?: Int | null
  twinId_lt?: Int | null
  twinId_lte?: Int | null
  twinId_in?: Int[] | Int | null
  location_eq?: ID_Input | null
  location_in?: ID_Output[] | ID_Output | null
  countryId_eq?: Int | null
  countryId_gt?: Int | null
  countryId_gte?: Int | null
  countryId_lt?: Int | null
  countryId_lte?: Int | null
  countryId_in?: Int[] | Int | null
  cityId_eq?: Int | null
  cityId_gt?: Int | null
  cityId_gte?: Int | null
  cityId_lt?: Int | null
  cityId_lte?: Int | null
  cityId_in?: Int[] | Int | null
  hru_eq?: BigInt | null
  hru_gt?: BigInt | null
  hru_gte?: BigInt | null
  hru_lt?: BigInt | null
  hru_lte?: BigInt | null
  hru_in?: BigInt[] | BigInt | null
  sru_eq?: BigInt | null
  sru_gt?: BigInt | null
  sru_gte?: BigInt | null
  sru_lt?: BigInt | null
  sru_lte?: BigInt | null
  sru_in?: BigInt[] | BigInt | null
  cru_eq?: BigInt | null
  cru_gt?: BigInt | null
  cru_gte?: BigInt | null
  cru_lt?: BigInt | null
  cru_lte?: BigInt | null
  cru_in?: BigInt[] | BigInt | null
  mru_eq?: BigInt | null
  mru_gt?: BigInt | null
  mru_gte?: BigInt | null
  mru_lt?: BigInt | null
  mru_lte?: BigInt | null
  mru_in?: BigInt[] | BigInt | null
  publicConfig_eq?: ID_Input | null
  publicConfig_in?: ID_Output[] | ID_Output | null
  uptime_eq?: Int | null
  uptime_gt?: Int | null
  uptime_gte?: Int | null
  uptime_lt?: Int | null
  uptime_lte?: Int | null
  uptime_in?: Int[] | Int | null
  created_eq?: Int | null
  created_gt?: Int | null
  created_gte?: Int | null
  created_lt?: Int | null
  created_lte?: Int | null
  created_in?: Int[] | Int | null
  farmingPolicyId_eq?: Int | null
  farmingPolicyId_gt?: Int | null
  farmingPolicyId_gte?: Int | null
  farmingPolicyId_lt?: Int | null
  farmingPolicyId_lte?: Int | null
  farmingPolicyId_in?: Int[] | Int | null
  location?: LocationWhereInput | null
  publicConfig?: PublicConfigWhereInput | null
  AND?: NodeWhereInput[] | NodeWhereInput | null
  OR?: NodeWhereInput[] | NodeWhereInput | null
}

export interface NodeWhereUniqueInput {
  id: ID_Output
}

export interface PricingPolicyCreateInput {
  gridVersion: Float
  pricingPolicyId: Float
  name: String
  currency: String
  su: Float
  cu: Float
  nu: Float
  foundationAccount: String
  certifiedSalesAccount: String
}

export interface PricingPolicyUpdateInput {
  gridVersion?: Float | null
  pricingPolicyId?: Float | null
  name?: String | null
  currency?: String | null
  su?: Float | null
  cu?: Float | null
  nu?: Float | null
  foundationAccount?: String | null
  certifiedSalesAccount?: String | null
}

export interface PricingPolicyWhereInput {
  id_eq?: ID_Input | null
  id_in?: ID_Output[] | ID_Output | null
  createdAt_eq?: DateTime | null
  createdAt_lt?: DateTime | null
  createdAt_lte?: DateTime | null
  createdAt_gt?: DateTime | null
  createdAt_gte?: DateTime | null
  createdById_eq?: ID_Input | null
  createdById_in?: ID_Output[] | ID_Output | null
  updatedAt_eq?: DateTime | null
  updatedAt_lt?: DateTime | null
  updatedAt_lte?: DateTime | null
  updatedAt_gt?: DateTime | null
  updatedAt_gte?: DateTime | null
  updatedById_eq?: ID_Input | null
  updatedById_in?: ID_Output[] | ID_Output | null
  deletedAt_all?: Boolean | null
  deletedAt_eq?: DateTime | null
  deletedAt_lt?: DateTime | null
  deletedAt_lte?: DateTime | null
  deletedAt_gt?: DateTime | null
  deletedAt_gte?: DateTime | null
  deletedById_eq?: ID_Input | null
  deletedById_in?: ID_Output[] | ID_Output | null
  gridVersion_eq?: Int | null
  gridVersion_gt?: Int | null
  gridVersion_gte?: Int | null
  gridVersion_lt?: Int | null
  gridVersion_lte?: Int | null
  gridVersion_in?: Int[] | Int | null
  pricingPolicyId_eq?: Int | null
  pricingPolicyId_gt?: Int | null
  pricingPolicyId_gte?: Int | null
  pricingPolicyId_lt?: Int | null
  pricingPolicyId_lte?: Int | null
  pricingPolicyId_in?: Int[] | Int | null
  name_eq?: String | null
  name_contains?: String | null
  name_startsWith?: String | null
  name_endsWith?: String | null
  name_in?: String[] | String | null
  currency_eq?: String | null
  currency_contains?: String | null
  currency_startsWith?: String | null
  currency_endsWith?: String | null
  currency_in?: String[] | String | null
  su_eq?: Int | null
  su_gt?: Int | null
  su_gte?: Int | null
  su_lt?: Int | null
  su_lte?: Int | null
  su_in?: Int[] | Int | null
  cu_eq?: Int | null
  cu_gt?: Int | null
  cu_gte?: Int | null
  cu_lt?: Int | null
  cu_lte?: Int | null
  cu_in?: Int[] | Int | null
  nu_eq?: Int | null
  nu_gt?: Int | null
  nu_gte?: Int | null
  nu_lt?: Int | null
  nu_lte?: Int | null
  nu_in?: Int[] | Int | null
  foundationAccount_eq?: String | null
  foundationAccount_contains?: String | null
  foundationAccount_startsWith?: String | null
  foundationAccount_endsWith?: String | null
  foundationAccount_in?: String[] | String | null
  certifiedSalesAccount_eq?: String | null
  certifiedSalesAccount_contains?: String | null
  certifiedSalesAccount_startsWith?: String | null
  certifiedSalesAccount_endsWith?: String | null
  certifiedSalesAccount_in?: String[] | String | null
  AND?: PricingPolicyWhereInput[] | PricingPolicyWhereInput | null
  OR?: PricingPolicyWhereInput[] | PricingPolicyWhereInput | null
}

export interface PricingPolicyWhereUniqueInput {
  id: ID_Output
}

export interface PublicConfigCreateInput {
  ipv4: String
  ipv6: String
  gw4: String
  gw6: String
}

export interface PublicConfigUpdateInput {
  ipv4?: String | null
  ipv6?: String | null
  gw4?: String | null
  gw6?: String | null
}

export interface PublicConfigWhereInput {
  id_eq?: ID_Input | null
  id_in?: ID_Output[] | ID_Output | null
  createdAt_eq?: DateTime | null
  createdAt_lt?: DateTime | null
  createdAt_lte?: DateTime | null
  createdAt_gt?: DateTime | null
  createdAt_gte?: DateTime | null
  createdById_eq?: ID_Input | null
  createdById_in?: ID_Output[] | ID_Output | null
  updatedAt_eq?: DateTime | null
  updatedAt_lt?: DateTime | null
  updatedAt_lte?: DateTime | null
  updatedAt_gt?: DateTime | null
  updatedAt_gte?: DateTime | null
  updatedById_eq?: ID_Input | null
  updatedById_in?: ID_Output[] | ID_Output | null
  deletedAt_all?: Boolean | null
  deletedAt_eq?: DateTime | null
  deletedAt_lt?: DateTime | null
  deletedAt_lte?: DateTime | null
  deletedAt_gt?: DateTime | null
  deletedAt_gte?: DateTime | null
  deletedById_eq?: ID_Input | null
  deletedById_in?: ID_Output[] | ID_Output | null
  ipv4_eq?: String | null
  ipv4_contains?: String | null
  ipv4_startsWith?: String | null
  ipv4_endsWith?: String | null
  ipv4_in?: String[] | String | null
  ipv6_eq?: String | null
  ipv6_contains?: String | null
  ipv6_startsWith?: String | null
  ipv6_endsWith?: String | null
  ipv6_in?: String[] | String | null
  gw4_eq?: String | null
  gw4_contains?: String | null
  gw4_startsWith?: String | null
  gw4_endsWith?: String | null
  gw4_in?: String[] | String | null
  gw6_eq?: String | null
  gw6_contains?: String | null
  gw6_startsWith?: String | null
  gw6_endsWith?: String | null
  gw6_in?: String[] | String | null
  nodepublicConfig_none?: NodeWhereInput | null
  nodepublicConfig_some?: NodeWhereInput | null
  nodepublicConfig_every?: NodeWhereInput | null
  AND?: PublicConfigWhereInput[] | PublicConfigWhereInput | null
  OR?: PublicConfigWhereInput[] | PublicConfigWhereInput | null
}

export interface PublicConfigWhereUniqueInput {
  id: ID_Output
}

export interface PublicIpCreateInput {
  farm: ID_Output
  gateway: String
  ip: String
  contractId: Float
}

export interface PublicIpUpdateInput {
  farm?: ID_Input | null
  gateway?: String | null
  ip?: String | null
  contractId?: Float | null
}

export interface PublicIpWhereInput {
  id_eq?: ID_Input | null
  id_in?: ID_Output[] | ID_Output | null
  createdAt_eq?: DateTime | null
  createdAt_lt?: DateTime | null
  createdAt_lte?: DateTime | null
  createdAt_gt?: DateTime | null
  createdAt_gte?: DateTime | null
  createdById_eq?: ID_Input | null
  createdById_in?: ID_Output[] | ID_Output | null
  updatedAt_eq?: DateTime | null
  updatedAt_lt?: DateTime | null
  updatedAt_lte?: DateTime | null
  updatedAt_gt?: DateTime | null
  updatedAt_gte?: DateTime | null
  updatedById_eq?: ID_Input | null
  updatedById_in?: ID_Output[] | ID_Output | null
  deletedAt_all?: Boolean | null
  deletedAt_eq?: DateTime | null
  deletedAt_lt?: DateTime | null
  deletedAt_lte?: DateTime | null
  deletedAt_gt?: DateTime | null
  deletedAt_gte?: DateTime | null
  deletedById_eq?: ID_Input | null
  deletedById_in?: ID_Output[] | ID_Output | null
  farm_eq?: ID_Input | null
  farm_in?: ID_Output[] | ID_Output | null
  gateway_eq?: String | null
  gateway_contains?: String | null
  gateway_startsWith?: String | null
  gateway_endsWith?: String | null
  gateway_in?: String[] | String | null
  ip_eq?: String | null
  ip_contains?: String | null
  ip_startsWith?: String | null
  ip_endsWith?: String | null
  ip_in?: String[] | String | null
  contractId_eq?: Int | null
  contractId_gt?: Int | null
  contractId_gte?: Int | null
  contractId_lt?: Int | null
  contractId_lte?: Int | null
  contractId_in?: Int[] | Int | null
  farm?: FarmWhereInput | null
  AND?: PublicIpWhereInput[] | PublicIpWhereInput | null
  OR?: PublicIpWhereInput[] | PublicIpWhereInput | null
}

export interface PublicIpWhereUniqueInput {
  id: ID_Output
}

export interface TransferCreateInput {
  from: String
  to: String
  value: BigInt
  comment?: String | null
  block: Float
}

export interface TransferUpdateInput {
  from?: String | null
  to?: String | null
  value?: BigInt | null
  comment?: String | null
  block?: Float | null
}

export interface TransferWhereInput {
  id_eq?: ID_Input | null
  id_in?: ID_Output[] | ID_Output | null
  createdAt_eq?: DateTime | null
  createdAt_lt?: DateTime | null
  createdAt_lte?: DateTime | null
  createdAt_gt?: DateTime | null
  createdAt_gte?: DateTime | null
  createdById_eq?: ID_Input | null
  createdById_in?: ID_Output[] | ID_Output | null
  updatedAt_eq?: DateTime | null
  updatedAt_lt?: DateTime | null
  updatedAt_lte?: DateTime | null
  updatedAt_gt?: DateTime | null
  updatedAt_gte?: DateTime | null
  updatedById_eq?: ID_Input | null
  updatedById_in?: ID_Output[] | ID_Output | null
  deletedAt_all?: Boolean | null
  deletedAt_eq?: DateTime | null
  deletedAt_lt?: DateTime | null
  deletedAt_lte?: DateTime | null
  deletedAt_gt?: DateTime | null
  deletedAt_gte?: DateTime | null
  deletedById_eq?: ID_Input | null
  deletedById_in?: ID_Output[] | ID_Output | null
  from_eq?: String | null
  from_contains?: String | null
  from_startsWith?: String | null
  from_endsWith?: String | null
  from_in?: String[] | String | null
  to_eq?: String | null
  to_contains?: String | null
  to_startsWith?: String | null
  to_endsWith?: String | null
  to_in?: String[] | String | null
  value_eq?: BigInt | null
  value_gt?: BigInt | null
  value_gte?: BigInt | null
  value_lt?: BigInt | null
  value_lte?: BigInt | null
  value_in?: BigInt[] | BigInt | null
  comment_eq?: String | null
  comment_contains?: String | null
  comment_startsWith?: String | null
  comment_endsWith?: String | null
  comment_in?: String[] | String | null
  block_eq?: Int | null
  block_gt?: Int | null
  block_gte?: Int | null
  block_lt?: Int | null
  block_lte?: Int | null
  block_in?: Int[] | Int | null
  AND?: TransferWhereInput[] | TransferWhereInput | null
  OR?: TransferWhereInput[] | TransferWhereInput | null
}

export interface TransferWhereUniqueInput {
  id: ID_Output
}

export interface TwinCreateInput {
  gridVersion: Float
  twinId: Float
  accountId: String
  ip: String
}

export interface TwinUpdateInput {
  gridVersion?: Float | null
  twinId?: Float | null
  accountId?: String | null
  ip?: String | null
}

export interface TwinWhereInput {
  id_eq?: ID_Input | null
  id_in?: ID_Output[] | ID_Output | null
  createdAt_eq?: DateTime | null
  createdAt_lt?: DateTime | null
  createdAt_lte?: DateTime | null
  createdAt_gt?: DateTime | null
  createdAt_gte?: DateTime | null
  createdById_eq?: ID_Input | null
  createdById_in?: ID_Output[] | ID_Output | null
  updatedAt_eq?: DateTime | null
  updatedAt_lt?: DateTime | null
  updatedAt_lte?: DateTime | null
  updatedAt_gt?: DateTime | null
  updatedAt_gte?: DateTime | null
  updatedById_eq?: ID_Input | null
  updatedById_in?: ID_Output[] | ID_Output | null
  deletedAt_all?: Boolean | null
  deletedAt_eq?: DateTime | null
  deletedAt_lt?: DateTime | null
  deletedAt_lte?: DateTime | null
  deletedAt_gt?: DateTime | null
  deletedAt_gte?: DateTime | null
  deletedById_eq?: ID_Input | null
  deletedById_in?: ID_Output[] | ID_Output | null
  gridVersion_eq?: Int | null
  gridVersion_gt?: Int | null
  gridVersion_gte?: Int | null
  gridVersion_lt?: Int | null
  gridVersion_lte?: Int | null
  gridVersion_in?: Int[] | Int | null
  twinId_eq?: Int | null
  twinId_gt?: Int | null
  twinId_gte?: Int | null
  twinId_lt?: Int | null
  twinId_lte?: Int | null
  twinId_in?: Int[] | Int | null
  accountId_eq?: String | null
  accountId_contains?: String | null
  accountId_startsWith?: String | null
  accountId_endsWith?: String | null
  accountId_in?: String[] | String | null
  ip_eq?: String | null
  ip_contains?: String | null
  ip_startsWith?: String | null
  ip_endsWith?: String | null
  ip_in?: String[] | String | null
  entityprooftwinRel_none?: EntityProofWhereInput | null
  entityprooftwinRel_some?: EntityProofWhereInput | null
  entityprooftwinRel_every?: EntityProofWhereInput | null
  AND?: TwinWhereInput[] | TwinWhereInput | null
  OR?: TwinWhereInput[] | TwinWhereInput | null
}

export interface TwinWhereUniqueInput {
  id: ID_Output
}

export interface BaseGraphQLObject {
  id: ID_Output
  createdAt: DateTime
  createdById: String
  updatedAt?: DateTime | null
  updatedById?: String | null
  deletedAt?: DateTime | null
  deletedById?: String | null
  version: Int
}

export interface DeleteResponse {
  id: ID_Output
}

export interface BaseModel extends BaseGraphQLObject {
  id: ID_Output
  createdAt: DateTime
  createdById: String
  updatedAt?: DateTime | null
  updatedById?: String | null
  deletedAt?: DateTime | null
  deletedById?: String | null
  version: Int
}

export interface BaseModelUUID extends BaseGraphQLObject {
  id: ID_Output
  createdAt: DateTime
  createdById: String
  updatedAt?: DateTime | null
  updatedById?: String | null
  deletedAt?: DateTime | null
  deletedById?: String | null
  version: Int
}

export interface City extends BaseGraphQLObject {
  id: ID_Output
  createdAt: DateTime
  createdById: String
  updatedAt?: DateTime | null
  updatedById?: String | null
  deletedAt?: DateTime | null
  deletedById?: String | null
  version: Int
  countryId: Int
  name: String
}

export interface CityConnection {
  totalCount: Int
  edges: Array<CityEdge>
  pageInfo: PageInfo
}

export interface CityEdge {
  node: City
  cursor: String
}

export interface CommentSearchFTSOutput {
  item: CommentSearchSearchResult
  rank: Float
  isTypeOf: String
  highlight: String
}

export interface Consumption extends BaseGraphQLObject {
  id: ID_Output
  createdAt: DateTime
  createdById: String
  updatedAt?: DateTime | null
  updatedById?: String | null
  deletedAt?: DateTime | null
  deletedById?: String | null
  version: Int
  contractId: Int
  timestamp: Int
  cru?: BigInt | null
  sru?: BigInt | null
  hru?: BigInt | null
  mru?: BigInt | null
  nru?: BigInt | null
}

export interface ConsumptionConnection {
  totalCount: Int
  edges: Array<ConsumptionEdge>
  pageInfo: PageInfo
}

export interface ConsumptionEdge {
  node: Consumption
  cursor: String
}

export interface ContractBillReport extends BaseGraphQLObject {
  id: ID_Output
  createdAt: DateTime
  createdById: String
  updatedAt?: DateTime | null
  updatedById?: String | null
  deletedAt?: DateTime | null
  deletedById?: String | null
  version: Int
  contractId: Int
  discountReceived: DiscountLevel
  amountBilled: Int
}

export interface ContractBillReportConnection {
  totalCount: Int
  edges: Array<ContractBillReportEdge>
  pageInfo: PageInfo
}

export interface ContractBillReportEdge {
  node: ContractBillReport
  cursor: String
}

export interface Country extends BaseGraphQLObject {
  id: ID_Output
  createdAt: DateTime
  createdById: String
  updatedAt?: DateTime | null
  updatedById?: String | null
  deletedAt?: DateTime | null
  deletedById?: String | null
  version: Int
  code: String
  name: String
  region: String
  subregion: String
}

export interface CountryConnection {
  totalCount: Int
  edges: Array<CountryEdge>
  pageInfo: PageInfo
}

export interface CountryEdge {
  node: Country
  cursor: String
}

export interface Entity extends BaseGraphQLObject {
  id: ID_Output
  createdAt: DateTime
  createdById: String
  updatedAt?: DateTime | null
  updatedById?: String | null
  deletedAt?: DateTime | null
  deletedById?: String | null
  version: Int
  gridVersion: Int
  entityId: Int
  name: String
  countryId?: Int | null
  cityId?: Int | null
  accountId: String
}

export interface EntityConnection {
  totalCount: Int
  edges: Array<EntityEdge>
  pageInfo: PageInfo
}

export interface EntityEdge {
  node: Entity
  cursor: String
}

export interface EntityProof extends BaseGraphQLObject {
  id: ID_Output
  createdAt: DateTime
  createdById: String
  updatedAt?: DateTime | null
  updatedById?: String | null
  deletedAt?: DateTime | null
  deletedById?: String | null
  version: Int
  entityId: Int
  signature: String
  twinRel: Twin
  twinRelId: String
}

export interface EntityProofConnection {
  totalCount: Int
  edges: Array<EntityProofEdge>
  pageInfo: PageInfo
}

export interface EntityProofEdge {
  node: EntityProof
  cursor: String
}

export interface Farm extends BaseGraphQLObject {
  id: ID_Output
  createdAt: DateTime
  createdById: String
  updatedAt?: DateTime | null
  updatedById?: String | null
  deletedAt?: DateTime | null
  deletedById?: String | null
  version: Int
  gridVersion: Int
  farmId: Int
  name: String
  twinId: Int
  pricingPolicyId: Int
  certificationType: CertificationType
  countryId?: Int | null
  cityId?: Int | null
  publicIPs: Array<PublicIp>
}

export interface FarmConnection {
  totalCount: Int
  edges: Array<FarmEdge>
  pageInfo: PageInfo
}

export interface FarmEdge {
  node: Farm
  cursor: String
}

export interface FarmingPolicy extends BaseGraphQLObject {
  id: ID_Output
  createdAt: DateTime
  createdById: String
  updatedAt?: DateTime | null
  updatedById?: String | null
  deletedAt?: DateTime | null
  deletedById?: String | null
  version: Int
  name: String
  cu: Int
  su: Int
  nu: Int
  ipv4: Int
  timestamp: Int
  certificationType: CertificationType
}

export interface FarmingPolicyConnection {
  totalCount: Int
  edges: Array<FarmingPolicyEdge>
  pageInfo: PageInfo
}

export interface FarmingPolicyEdge {
  node: FarmingPolicy
  cursor: String
}

export interface Location extends BaseGraphQLObject {
  id: ID_Output
  createdAt: DateTime
  createdById: String
  updatedAt?: DateTime | null
  updatedById?: String | null
  deletedAt?: DateTime | null
  deletedById?: String | null
  version: Int
  longitude: String
  latitude: String
  nodelocation?: Array<Node> | null
}

export interface LocationConnection {
  totalCount: Int
  edges: Array<LocationEdge>
  pageInfo: PageInfo
}

export interface LocationEdge {
  node: Location
  cursor: String
}

export interface Node extends BaseGraphQLObject {
  id: ID_Output
  createdAt: DateTime
  createdById: String
  updatedAt?: DateTime | null
  updatedById?: String | null
  deletedAt?: DateTime | null
  deletedById?: String | null
  version: Int
  gridVersion: Int
  nodeId: Int
  farmId: Int
  twinId: Int
  location: Location
  locationId: String
  countryId?: Int | null
  cityId?: Int | null
  hru?: BigInt | null
  sru?: BigInt | null
  cru?: BigInt | null
  mru?: BigInt | null
  publicConfig?: PublicConfig | null
  publicConfigId?: String | null
  uptime?: Int | null
  created: Int
  farmingPolicyId: Int
}

export interface NodeConnection {
  totalCount: Int
  edges: Array<NodeEdge>
  pageInfo: PageInfo
}

export interface NodeContract extends BaseGraphQLObject {
  id: ID_Output
  createdAt: DateTime
  createdById: String
  updatedAt?: DateTime | null
  updatedById?: String | null
  deletedAt?: DateTime | null
  deletedById?: String | null
  version: Int
  contractId: Int
  twinId: Int
  nodeId: Int
  deploymentData: String
  deploymentHash: String
  numberOfPublicIPs: Int
  state: ContractState
}

export interface NodeContractConnection {
  totalCount: Int
  edges: Array<NodeContractEdge>
  pageInfo: PageInfo
}

export interface NodeContractEdge {
  node: NodeContract
  cursor: String
}

export interface NodeEdge {
  node: Node
  cursor: String
}

export interface PageInfo {
  hasNextPage: Boolean
  hasPreviousPage: Boolean
  startCursor?: String | null
  endCursor?: String | null
}

export interface PricingPolicy extends BaseGraphQLObject {
  id: ID_Output
  createdAt: DateTime
  createdById: String
  updatedAt?: DateTime | null
  updatedById?: String | null
  deletedAt?: DateTime | null
  deletedById?: String | null
  version: Int
  gridVersion: Int
  pricingPolicyId: Int
  name: String
  currency: String
  su: Int
  cu: Int
  nu: Int
  foundationAccount: String
  certifiedSalesAccount: String
}

export interface PricingPolicyConnection {
  totalCount: Int
  edges: Array<PricingPolicyEdge>
  pageInfo: PageInfo
}

export interface PricingPolicyEdge {
  node: PricingPolicy
  cursor: String
}

export interface ProcessorState {
  lastCompleteBlock: Float
  lastProcessedEvent: String
  indexerHead: Float
  chainHead: Float
}

export interface PublicConfig extends BaseGraphQLObject {
  id: ID_Output
  createdAt: DateTime
  createdById: String
  updatedAt?: DateTime | null
  updatedById?: String | null
  deletedAt?: DateTime | null
  deletedById?: String | null
  version: Int
  ipv4: String
  ipv6: String
  gw4: String
  gw6: String
  nodepublicConfig?: Array<Node> | null
}

export interface PublicConfigConnection {
  totalCount: Int
  edges: Array<PublicConfigEdge>
  pageInfo: PageInfo
}

export interface PublicConfigEdge {
  node: PublicConfig
  cursor: String
}

export interface PublicIp extends BaseGraphQLObject {
  id: ID_Output
  createdAt: DateTime
  createdById: String
  updatedAt?: DateTime | null
  updatedById?: String | null
  deletedAt?: DateTime | null
  deletedById?: String | null
  version: Int
  farm: Farm
  farmId: String
  gateway: String
  ip: String
  contractId: Int
}

export interface PublicIpConnection {
  totalCount: Int
  edges: Array<PublicIpEdge>
  pageInfo: PageInfo
}

export interface PublicIpEdge {
  node: PublicIp
  cursor: String
}

export interface StandardDeleteResponse {
  id: ID_Output
}

/*
 *  All transfers 

 */
export interface Transfer extends BaseGraphQLObject {
  id: ID_Output
  createdAt: DateTime
  createdById: String
  updatedAt?: DateTime | null
  updatedById?: String | null
  deletedAt?: DateTime | null
  deletedById?: String | null
  version: Int
  from: String
  to: String
  value: BigInt
  comment?: String | null
  block: Int
}

export interface TransferConnection {
  totalCount: Int
  edges: Array<TransferEdge>
  pageInfo: PageInfo
}

export interface TransferEdge {
  node: Transfer
  cursor: String
}

export interface Twin extends BaseGraphQLObject {
  id: ID_Output
  createdAt: DateTime
  createdById: String
  updatedAt?: DateTime | null
  updatedById?: String | null
  deletedAt?: DateTime | null
  deletedById?: String | null
  version: Int
  gridVersion: Int
  twinId: Int
  accountId: String
  ip: String
  entityprooftwinRel?: Array<EntityProof> | null
}

export interface TwinConnection {
  totalCount: Int
  edges: Array<TwinEdge>
  pageInfo: PageInfo
}

export interface TwinEdge {
  node: Twin
  cursor: String
}

/*
GraphQL representation of BigInt
*/
export type BigInt = string

/*
The `Boolean` scalar type represents `true` or `false`.
*/
export type Boolean = boolean

/*
The javascript `Date` as string. Type represents date and time as the ISO Date string.
*/
export type DateTime = Date | string

/*
The `Float` scalar type represents signed double-precision fractional values as specified by [IEEE 754](https://en.wikipedia.org/wiki/IEEE_floating_point).
*/
export type Float = number

/*
The `ID` scalar type represents a unique identifier, often used to refetch an object or as key for a cache. The ID type appears in a JSON response as a String; however, it is not intended to be human-readable. When expected as an input type, any string (such as `"4"`) or integer (such as `4`) input value will be accepted as an ID.
*/
export type ID_Input = string | number
export type ID_Output = string

/*
The `Int` scalar type represents non-fractional signed whole numeric values. Int can represent values between -(2^31) and 2^31 - 1.
*/
export type Int = number

/*
The `String` scalar type represents textual data, represented as UTF-8 character sequences. The String type is most often used by GraphQL to represent free-form human-readable text.
*/
export type String = string

export type CommentSearchSearchResult = Transfer