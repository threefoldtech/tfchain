import 'graphql-import-node'; // Needed so you can import *.graphql files 

import { makeBindingClass, Options } from 'graphql-binding'
import { GraphQLResolveInfo, GraphQLSchema } from 'graphql'
import { IResolvers } from 'graphql-tools/dist/Interfaces'
import * as schema from  './schema.graphql'

export interface Query {
    cities: <T = Array<City>>(args: { offset?: Int | null, limit?: Int | null, where?: CityWhereInput | null, orderBy?: CityOrderByInput | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    city: <T = City | null>(args: { where: CityWhereUniqueInput }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T | null> ,
    citiesConnection: <T = CityConnection>(args: { first?: Int | null, after?: String | null, last?: Int | null, before?: String | null, where?: CityWhereInput | null, orderBy?: CityOrderByInput | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    countries: <T = Array<Country>>(args: { offset?: Int | null, limit?: Int | null, where?: CountryWhereInput | null, orderBy?: CountryOrderByInput | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    country: <T = Country | null>(args: { where: CountryWhereUniqueInput }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T | null> ,
    countriesConnection: <T = CountryConnection>(args: { first?: Int | null, after?: String | null, last?: Int | null, before?: String | null, where?: CountryWhereInput | null, orderBy?: CountryOrderByInput | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    entityProofs: <T = Array<EntityProof>>(args: { offset?: Int | null, limit?: Int | null, where?: EntityProofWhereInput | null, orderBy?: EntityProofOrderByInput | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    entityProof: <T = EntityProof | null>(args: { where: EntityProofWhereUniqueInput }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T | null> ,
    entityProofsConnection: <T = EntityProofConnection>(args: { first?: Int | null, after?: String | null, last?: Int | null, before?: String | null, where?: EntityProofWhereInput | null, orderBy?: EntityProofOrderByInput | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    entities: <T = Array<Entity>>(args: { offset?: Int | null, limit?: Int | null, where?: EntityWhereInput | null, orderBy?: EntityOrderByInput | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    entity: <T = Entity | null>(args: { where: EntityWhereUniqueInput }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T | null> ,
    entitiesConnection: <T = EntityConnection>(args: { first?: Int | null, after?: String | null, last?: Int | null, before?: String | null, where?: EntityWhereInput | null, orderBy?: EntityOrderByInput | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    executedVestingWithdrawals: <T = Array<ExecutedVestingWithdrawal>>(args: { offset?: Int | null, limit?: Int | null, where?: ExecutedVestingWithdrawalWhereInput | null, orderBy?: ExecutedVestingWithdrawalOrderByInput | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    executedVestingWithdrawal: <T = ExecutedVestingWithdrawal | null>(args: { where: ExecutedVestingWithdrawalWhereUniqueInput }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T | null> ,
    executedVestingWithdrawalsConnection: <T = ExecutedVestingWithdrawalConnection>(args: { first?: Int | null, after?: String | null, last?: Int | null, before?: String | null, where?: ExecutedVestingWithdrawalWhereInput | null, orderBy?: ExecutedVestingWithdrawalOrderByInput | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    expiredVestingWithdrawals: <T = Array<ExpiredVestingWithdrawal>>(args: { offset?: Int | null, limit?: Int | null, where?: ExpiredVestingWithdrawalWhereInput | null, orderBy?: ExpiredVestingWithdrawalOrderByInput | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    expiredVestingWithdrawal: <T = ExpiredVestingWithdrawal | null>(args: { where: ExpiredVestingWithdrawalWhereUniqueInput }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T | null> ,
    expiredVestingWithdrawalsConnection: <T = ExpiredVestingWithdrawalConnection>(args: { first?: Int | null, after?: String | null, last?: Int | null, before?: String | null, where?: ExpiredVestingWithdrawalWhereInput | null, orderBy?: ExpiredVestingWithdrawalOrderByInput | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    failedVestingWithdrawals: <T = Array<FailedVestingWithdrawal>>(args: { offset?: Int | null, limit?: Int | null, where?: FailedVestingWithdrawalWhereInput | null, orderBy?: FailedVestingWithdrawalOrderByInput | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    failedVestingWithdrawal: <T = FailedVestingWithdrawal | null>(args: { where: FailedVestingWithdrawalWhereUniqueInput }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T | null> ,
    failedVestingWithdrawalsConnection: <T = FailedVestingWithdrawalConnection>(args: { first?: Int | null, after?: String | null, last?: Int | null, before?: String | null, where?: FailedVestingWithdrawalWhereInput | null, orderBy?: FailedVestingWithdrawalOrderByInput | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    farms: <T = Array<Farm>>(args: { offset?: Int | null, limit?: Int | null, where?: FarmWhereInput | null, orderBy?: FarmOrderByInput | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    farm: <T = Farm | null>(args: { where: FarmWhereUniqueInput }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T | null> ,
    farmsConnection: <T = FarmConnection>(args: { first?: Int | null, after?: String | null, last?: Int | null, before?: String | null, where?: FarmWhereInput | null, orderBy?: FarmOrderByInput | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    locations: <T = Array<Location>>(args: { offset?: Int | null, limit?: Int | null, where?: LocationWhereInput | null, orderBy?: LocationOrderByInput | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    location: <T = Location | null>(args: { where: LocationWhereUniqueInput }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T | null> ,
    locationsConnection: <T = LocationConnection>(args: { first?: Int | null, after?: String | null, last?: Int | null, before?: String | null, where?: LocationWhereInput | null, orderBy?: LocationOrderByInput | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    nodes: <T = Array<Node>>(args: { offset?: Int | null, limit?: Int | null, where?: NodeWhereInput | null, orderBy?: NodeOrderByInput | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    node: <T = Node | null>(args: { where: NodeWhereUniqueInput }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T | null> ,
    nodesConnection: <T = NodeConnection>(args: { first?: Int | null, after?: String | null, last?: Int | null, before?: String | null, where?: NodeWhereInput | null, orderBy?: NodeOrderByInput | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    pricingPolicies: <T = Array<PricingPolicy>>(args: { offset?: Int | null, limit?: Int | null, where?: PricingPolicyWhereInput | null, orderBy?: PricingPolicyOrderByInput | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    pricingPolicy: <T = PricingPolicy | null>(args: { where: PricingPolicyWhereUniqueInput }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T | null> ,
    pricingPoliciesConnection: <T = PricingPolicyConnection>(args: { first?: Int | null, after?: String | null, last?: Int | null, before?: String | null, where?: PricingPolicyWhereInput | null, orderBy?: PricingPolicyOrderByInput | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    commentSearch: <T = Array<CommentSearchFTSOutput>>(args: { limit?: Int | null, text: String }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    transfers: <T = Array<Transfer>>(args: { offset?: Int | null, limit?: Int | null, where?: TransferWhereInput | null, orderBy?: TransferOrderByInput | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    transfer: <T = Transfer | null>(args: { where: TransferWhereUniqueInput }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T | null> ,
    transfersConnection: <T = TransferConnection>(args: { first?: Int | null, after?: String | null, last?: Int | null, before?: String | null, where?: TransferWhereInput | null, orderBy?: TransferOrderByInput | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    twins: <T = Array<Twin>>(args: { offset?: Int | null, limit?: Int | null, where?: TwinWhereInput | null, orderBy?: TwinOrderByInput | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    twin: <T = Twin | null>(args: { where: TwinWhereUniqueInput }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T | null> ,
    twinsConnection: <T = TwinConnection>(args: { first?: Int | null, after?: String | null, last?: Int | null, before?: String | null, where?: TwinWhereInput | null, orderBy?: TwinOrderByInput | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> 
  }

export interface Mutation {}

export interface Subscription {}

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

export type CertificationType =   'None' |
  'Silver' |
  'Gold'

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

export type CountryOrderByInput =   'createdAt_ASC' |
  'createdAt_DESC' |
  'updatedAt_ASC' |
  'updatedAt_DESC' |
  'deletedAt_ASC' |
  'deletedAt_DESC' |
  'code_ASC' |
  'code_DESC' |
  'name_ASC' |
  'name_DESC'

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
  'address_ASC' |
  'address_DESC'

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
  'twinRelId_ASC' |
  'twinRelId_DESC'

export type ExecutedVestingWithdrawalOrderByInput =   'createdAt_ASC' |
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
  'txXdr_ASC' |
  'txXdr_DESC' |
  'block_ASC' |
  'block_DESC'

export type ExpiredVestingWithdrawalOrderByInput =   'createdAt_ASC' |
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
  'txXdr_ASC' |
  'txXdr_DESC' |
  'block_ASC' |
  'block_DESC'

export type FailedVestingWithdrawalOrderByInput =   'createdAt_ASC' |
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
  'txXdr_ASC' |
  'txXdr_DESC' |
  'block_ASC' |
  'block_DESC' |
  'reason_ASC' |
  'reason_DESC'

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
  'locationId_ASC' |
  'locationId_DESC' |
  'countryId_ASC' |
  'countryId_DESC' |
  'cityId_ASC' |
  'cityId_DESC' |
  'address_ASC' |
  'address_DESC' |
  'pubKey_ASC' |
  'pubKey_DESC' |
  'hru_ASC' |
  'hru_DESC' |
  'sru_ASC' |
  'sru_DESC' |
  'cru_ASC' |
  'cru_DESC' |
  'mru_ASC' |
  'mru_DESC' |
  'role_ASC' |
  'role_DESC'

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
  'nu_DESC'

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
  'address_ASC' |
  'address_DESC' |
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
}

export interface CityWhereUniqueInput {
  id: ID_Output
}

export interface CountryCreateInput {
  code: String
  name: String
}

export interface CountryUpdateInput {
  code?: String | null
  name?: String | null
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
  address: String
}

export interface EntityProofCreateInput {
  entityId: Float
  signature: String
  twinRelId: ID_Output
}

export interface EntityProofUpdateInput {
  entityId?: Float | null
  signature?: String | null
  twinRelId?: ID_Input | null
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
  twinRelId_eq?: ID_Input | null
  twinRelId_in?: ID_Output[] | ID_Output | null
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
  address?: String | null
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
  address_eq?: String | null
  address_contains?: String | null
  address_startsWith?: String | null
  address_endsWith?: String | null
  address_in?: String[] | String | null
}

export interface EntityWhereUniqueInput {
  id: ID_Output
}

export interface ExecutedVestingWithdrawalCreateInput {
  from: String
  to: String
  value: Float
  txXdr: String
  block: Float
}

export interface ExecutedVestingWithdrawalUpdateInput {
  from?: String | null
  to?: String | null
  value?: Float | null
  txXdr?: String | null
  block?: Float | null
}

export interface ExecutedVestingWithdrawalWhereInput {
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
  value_eq?: Int | null
  value_gt?: Int | null
  value_gte?: Int | null
  value_lt?: Int | null
  value_lte?: Int | null
  value_in?: Int[] | Int | null
  txXdr_eq?: String | null
  txXdr_contains?: String | null
  txXdr_startsWith?: String | null
  txXdr_endsWith?: String | null
  txXdr_in?: String[] | String | null
  block_eq?: Int | null
  block_gt?: Int | null
  block_gte?: Int | null
  block_lt?: Int | null
  block_lte?: Int | null
  block_in?: Int[] | Int | null
}

export interface ExecutedVestingWithdrawalWhereUniqueInput {
  id: ID_Output
}

export interface ExpiredVestingWithdrawalCreateInput {
  from: String
  to: String
  value: Float
  txXdr: String
  block: Float
}

export interface ExpiredVestingWithdrawalUpdateInput {
  from?: String | null
  to?: String | null
  value?: Float | null
  txXdr?: String | null
  block?: Float | null
}

export interface ExpiredVestingWithdrawalWhereInput {
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
  value_eq?: Int | null
  value_gt?: Int | null
  value_gte?: Int | null
  value_lt?: Int | null
  value_lte?: Int | null
  value_in?: Int[] | Int | null
  txXdr_eq?: String | null
  txXdr_contains?: String | null
  txXdr_startsWith?: String | null
  txXdr_endsWith?: String | null
  txXdr_in?: String[] | String | null
  block_eq?: Int | null
  block_gt?: Int | null
  block_gte?: Int | null
  block_lt?: Int | null
  block_lte?: Int | null
  block_in?: Int[] | Int | null
}

export interface ExpiredVestingWithdrawalWhereUniqueInput {
  id: ID_Output
}

export interface FailedVestingWithdrawalCreateInput {
  from: String
  to: String
  value: Float
  txXdr: String
  block: Float
  reason: String
}

export interface FailedVestingWithdrawalUpdateInput {
  from?: String | null
  to?: String | null
  value?: Float | null
  txXdr?: String | null
  block?: Float | null
  reason?: String | null
}

export interface FailedVestingWithdrawalWhereInput {
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
  value_eq?: Int | null
  value_gt?: Int | null
  value_gte?: Int | null
  value_lt?: Int | null
  value_lte?: Int | null
  value_in?: Int[] | Int | null
  txXdr_eq?: String | null
  txXdr_contains?: String | null
  txXdr_startsWith?: String | null
  txXdr_endsWith?: String | null
  txXdr_in?: String[] | String | null
  block_eq?: Int | null
  block_gt?: Int | null
  block_gte?: Int | null
  block_lt?: Int | null
  block_lte?: Int | null
  block_in?: Int[] | Int | null
  reason_eq?: String | null
  reason_contains?: String | null
  reason_startsWith?: String | null
  reason_endsWith?: String | null
  reason_in?: String[] | String | null
}

export interface FailedVestingWithdrawalWhereUniqueInput {
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
}

export interface LocationWhereUniqueInput {
  id: ID_Output
}

export interface NodeCreateInput {
  gridVersion: Float
  nodeId: Float
  farmId: Float
  locationId: ID_Output
  countryId?: Float | null
  cityId?: Float | null
  address: String
  pubKey: String
  hru?: Float | null
  sru?: Float | null
  cru?: Float | null
  mru?: Float | null
  role: String
}

export interface NodeUpdateInput {
  gridVersion?: Float | null
  nodeId?: Float | null
  farmId?: Float | null
  locationId?: ID_Input | null
  countryId?: Float | null
  cityId?: Float | null
  address?: String | null
  pubKey?: String | null
  hru?: Float | null
  sru?: Float | null
  cru?: Float | null
  mru?: Float | null
  role?: String | null
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
  locationId_eq?: ID_Input | null
  locationId_in?: ID_Output[] | ID_Output | null
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
  address_eq?: String | null
  address_contains?: String | null
  address_startsWith?: String | null
  address_endsWith?: String | null
  address_in?: String[] | String | null
  pubKey_eq?: String | null
  pubKey_contains?: String | null
  pubKey_startsWith?: String | null
  pubKey_endsWith?: String | null
  pubKey_in?: String[] | String | null
  hru_eq?: Int | null
  hru_gt?: Int | null
  hru_gte?: Int | null
  hru_lt?: Int | null
  hru_lte?: Int | null
  hru_in?: Int[] | Int | null
  sru_eq?: Int | null
  sru_gt?: Int | null
  sru_gte?: Int | null
  sru_lt?: Int | null
  sru_lte?: Int | null
  sru_in?: Int[] | Int | null
  cru_eq?: Int | null
  cru_gt?: Int | null
  cru_gte?: Int | null
  cru_lt?: Int | null
  cru_lte?: Int | null
  cru_in?: Int[] | Int | null
  mru_eq?: Int | null
  mru_gt?: Int | null
  mru_gte?: Int | null
  mru_lt?: Int | null
  mru_lte?: Int | null
  mru_in?: Int[] | Int | null
  role_eq?: String | null
  role_contains?: String | null
  role_startsWith?: String | null
  role_endsWith?: String | null
  role_in?: String[] | String | null
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
}

export interface PricingPolicyUpdateInput {
  gridVersion?: Float | null
  pricingPolicyId?: Float | null
  name?: String | null
  currency?: String | null
  su?: Float | null
  cu?: Float | null
  nu?: Float | null
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
}

export interface PricingPolicyWhereUniqueInput {
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
}

export interface TransferWhereUniqueInput {
  id: ID_Output
}

export interface TwinCreateInput {
  gridVersion: Float
  twinId: Float
  address: String
  ip: String
}

export interface TwinUpdateInput {
  gridVersion?: Float | null
  twinId?: Float | null
  address?: String | null
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
  address_eq?: String | null
  address_contains?: String | null
  address_startsWith?: String | null
  address_endsWith?: String | null
  address_in?: String[] | String | null
  ip_eq?: String | null
  ip_contains?: String | null
  ip_startsWith?: String | null
  ip_endsWith?: String | null
  ip_in?: String[] | String | null
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
  address: String
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

export interface ExecutedVestingWithdrawal extends BaseGraphQLObject {
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
  value: Int
  txXdr: String
  block: Int
}

export interface ExecutedVestingWithdrawalConnection {
  totalCount: Int
  edges: Array<ExecutedVestingWithdrawalEdge>
  pageInfo: PageInfo
}

export interface ExecutedVestingWithdrawalEdge {
  node: ExecutedVestingWithdrawal
  cursor: String
}

export interface ExpiredVestingWithdrawal extends BaseGraphQLObject {
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
  value: Int
  txXdr: String
  block: Int
}

export interface ExpiredVestingWithdrawalConnection {
  totalCount: Int
  edges: Array<ExpiredVestingWithdrawalEdge>
  pageInfo: PageInfo
}

export interface ExpiredVestingWithdrawalEdge {
  node: ExpiredVestingWithdrawal
  cursor: String
}

export interface FailedVestingWithdrawal extends BaseGraphQLObject {
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
  value: Int
  txXdr: String
  block: Int
  reason: String
}

export interface FailedVestingWithdrawalConnection {
  totalCount: Int
  edges: Array<FailedVestingWithdrawalEdge>
  pageInfo: PageInfo
}

export interface FailedVestingWithdrawalEdge {
  node: FailedVestingWithdrawal
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
  location: Location
  locationId: String
  countryId?: Int | null
  cityId?: Int | null
  address: String
  pubKey: String
  hru?: Int | null
  sru?: Int | null
  cru?: Int | null
  mru?: Int | null
  role: String
}

export interface NodeConnection {
  totalCount: Int
  edges: Array<NodeEdge>
  pageInfo: PageInfo
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
  address: String
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