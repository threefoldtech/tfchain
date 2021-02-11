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
    resources: <T = Array<Resource>>(args: { offset?: Int | null, limit?: Int | null, where?: ResourceWhereInput | null, orderBy?: ResourceOrderByInput | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    resource: <T = Resource | null>(args: { where: ResourceWhereUniqueInput }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T | null> ,
    resourcesConnection: <T = ResourceConnection>(args: { first?: Int | null, after?: String | null, last?: Int | null, before?: String | null, where?: ResourceWhereInput | null, orderBy?: ResourceOrderByInput | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
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
  'entityId_ASC' |
  'entityId_DESC' |
  'name_ASC' |
  'name_DESC' |
  'countryId_ASC' |
  'countryId_DESC' |
  'cityId_ASC' |
  'cityId_DESC' |
  'pubKey_ASC' |
  'pubKey_DESC'

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

export type FarmOrderByInput =   'createdAt_ASC' |
  'createdAt_DESC' |
  'updatedAt_ASC' |
  'updatedAt_DESC' |
  'deletedAt_ASC' |
  'deletedAt_DESC' |
  'farmId_ASC' |
  'farmId_DESC' |
  'name_ASC' |
  'name_DESC' |
  'entityId_ASC' |
  'entityId_DESC' |
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
  'nodeId_ASC' |
  'nodeId_DESC' |
  'farmId_ASC' |
  'farmId_DESC' |
  'twinId_ASC' |
  'twinId_DESC' |
  'resourcesId_ASC' |
  'resourcesId_DESC' |
  'locationId_ASC' |
  'locationId_DESC' |
  'countryId_ASC' |
  'countryId_DESC' |
  'cityId_ASC' |
  'cityId_DESC'

export type PricingPolicyOrderByInput =   'createdAt_ASC' |
  'createdAt_DESC' |
  'updatedAt_ASC' |
  'updatedAt_DESC' |
  'deletedAt_ASC' |
  'deletedAt_DESC' |
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

export type ResourceOrderByInput =   'createdAt_ASC' |
  'createdAt_DESC' |
  'updatedAt_ASC' |
  'updatedAt_DESC' |
  'deletedAt_ASC' |
  'deletedAt_DESC' |
  'hru_ASC' |
  'hru_DESC' |
  'sru_ASC' |
  'sru_DESC' |
  'cru_ASC' |
  'cru_DESC' |
  'mru_ASC' |
  'mru_DESC'

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
  'twinId_ASC' |
  'twinId_DESC' |
  'pubKey_ASC' |
  'pubKey_DESC' |
  'peerId_ASC' |
  'peerId_DESC'

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
  countryId: BigInt
  name: String
}

export interface CityUpdateInput {
  countryId?: BigInt | null
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
  countryId_eq?: BigInt | null
  countryId_gt?: BigInt | null
  countryId_gte?: BigInt | null
  countryId_lt?: BigInt | null
  countryId_lte?: BigInt | null
  countryId_in?: BigInt[] | BigInt | null
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
  entityId: BigInt
  name: String
  countryId?: BigInt | null
  cityId?: BigInt | null
  pubKey: String
}

export interface EntityProofCreateInput {
  entityId: BigInt
  signature: String
  twinRelId: ID_Output
}

export interface EntityProofUpdateInput {
  entityId?: BigInt | null
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
  entityId_eq?: BigInt | null
  entityId_gt?: BigInt | null
  entityId_gte?: BigInt | null
  entityId_lt?: BigInt | null
  entityId_lte?: BigInt | null
  entityId_in?: BigInt[] | BigInt | null
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
  entityId?: BigInt | null
  name?: String | null
  countryId?: BigInt | null
  cityId?: BigInt | null
  pubKey?: String | null
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
  entityId_eq?: BigInt | null
  entityId_gt?: BigInt | null
  entityId_gte?: BigInt | null
  entityId_lt?: BigInt | null
  entityId_lte?: BigInt | null
  entityId_in?: BigInt[] | BigInt | null
  name_eq?: String | null
  name_contains?: String | null
  name_startsWith?: String | null
  name_endsWith?: String | null
  name_in?: String[] | String | null
  countryId_eq?: BigInt | null
  countryId_gt?: BigInt | null
  countryId_gte?: BigInt | null
  countryId_lt?: BigInt | null
  countryId_lte?: BigInt | null
  countryId_in?: BigInt[] | BigInt | null
  cityId_eq?: BigInt | null
  cityId_gt?: BigInt | null
  cityId_gte?: BigInt | null
  cityId_lt?: BigInt | null
  cityId_lte?: BigInt | null
  cityId_in?: BigInt[] | BigInt | null
  pubKey_eq?: String | null
  pubKey_contains?: String | null
  pubKey_startsWith?: String | null
  pubKey_endsWith?: String | null
  pubKey_in?: String[] | String | null
}

export interface EntityWhereUniqueInput {
  id: ID_Output
}

export interface FarmCreateInput {
  farmId: BigInt
  name: String
  entityId: BigInt
  twinId: BigInt
  pricingPolicyId: BigInt
  certificationType: CertificationType
  countryId?: BigInt | null
  cityId?: BigInt | null
}

export interface FarmUpdateInput {
  farmId?: BigInt | null
  name?: String | null
  entityId?: BigInt | null
  twinId?: BigInt | null
  pricingPolicyId?: BigInt | null
  certificationType?: CertificationType | null
  countryId?: BigInt | null
  cityId?: BigInt | null
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
  farmId_eq?: BigInt | null
  farmId_gt?: BigInt | null
  farmId_gte?: BigInt | null
  farmId_lt?: BigInt | null
  farmId_lte?: BigInt | null
  farmId_in?: BigInt[] | BigInt | null
  name_eq?: String | null
  name_contains?: String | null
  name_startsWith?: String | null
  name_endsWith?: String | null
  name_in?: String[] | String | null
  entityId_eq?: BigInt | null
  entityId_gt?: BigInt | null
  entityId_gte?: BigInt | null
  entityId_lt?: BigInt | null
  entityId_lte?: BigInt | null
  entityId_in?: BigInt[] | BigInt | null
  twinId_eq?: BigInt | null
  twinId_gt?: BigInt | null
  twinId_gte?: BigInt | null
  twinId_lt?: BigInt | null
  twinId_lte?: BigInt | null
  twinId_in?: BigInt[] | BigInt | null
  pricingPolicyId_eq?: BigInt | null
  pricingPolicyId_gt?: BigInt | null
  pricingPolicyId_gte?: BigInt | null
  pricingPolicyId_lt?: BigInt | null
  pricingPolicyId_lte?: BigInt | null
  pricingPolicyId_in?: BigInt[] | BigInt | null
  certificationType_eq?: CertificationType | null
  certificationType_in?: CertificationType[] | CertificationType | null
  countryId_eq?: BigInt | null
  countryId_gt?: BigInt | null
  countryId_gte?: BigInt | null
  countryId_lt?: BigInt | null
  countryId_lte?: BigInt | null
  countryId_in?: BigInt[] | BigInt | null
  cityId_eq?: BigInt | null
  cityId_gt?: BigInt | null
  cityId_gte?: BigInt | null
  cityId_lt?: BigInt | null
  cityId_lte?: BigInt | null
  cityId_in?: BigInt[] | BigInt | null
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
  nodeId: BigInt
  farmId: BigInt
  twinId: BigInt
  resourcesId: ID_Output
  locationId: ID_Output
  countryId?: BigInt | null
  cityId?: BigInt | null
}

export interface NodeUpdateInput {
  nodeId?: BigInt | null
  farmId?: BigInt | null
  twinId?: BigInt | null
  resourcesId?: ID_Input | null
  locationId?: ID_Input | null
  countryId?: BigInt | null
  cityId?: BigInt | null
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
  nodeId_eq?: BigInt | null
  nodeId_gt?: BigInt | null
  nodeId_gte?: BigInt | null
  nodeId_lt?: BigInt | null
  nodeId_lte?: BigInt | null
  nodeId_in?: BigInt[] | BigInt | null
  farmId_eq?: BigInt | null
  farmId_gt?: BigInt | null
  farmId_gte?: BigInt | null
  farmId_lt?: BigInt | null
  farmId_lte?: BigInt | null
  farmId_in?: BigInt[] | BigInt | null
  twinId_eq?: BigInt | null
  twinId_gt?: BigInt | null
  twinId_gte?: BigInt | null
  twinId_lt?: BigInt | null
  twinId_lte?: BigInt | null
  twinId_in?: BigInt[] | BigInt | null
  resourcesId_eq?: ID_Input | null
  resourcesId_in?: ID_Output[] | ID_Output | null
  locationId_eq?: ID_Input | null
  locationId_in?: ID_Output[] | ID_Output | null
  countryId_eq?: BigInt | null
  countryId_gt?: BigInt | null
  countryId_gte?: BigInt | null
  countryId_lt?: BigInt | null
  countryId_lte?: BigInt | null
  countryId_in?: BigInt[] | BigInt | null
  cityId_eq?: BigInt | null
  cityId_gt?: BigInt | null
  cityId_gte?: BigInt | null
  cityId_lt?: BigInt | null
  cityId_lte?: BigInt | null
  cityId_in?: BigInt[] | BigInt | null
}

export interface NodeWhereUniqueInput {
  id: ID_Output
}

export interface PricingPolicyCreateInput {
  pricingPolicyId: BigInt
  name: String
  currency: String
  su: BigInt
  cu: BigInt
  nu: BigInt
}

export interface PricingPolicyUpdateInput {
  pricingPolicyId?: BigInt | null
  name?: String | null
  currency?: String | null
  su?: BigInt | null
  cu?: BigInt | null
  nu?: BigInt | null
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
  pricingPolicyId_eq?: BigInt | null
  pricingPolicyId_gt?: BigInt | null
  pricingPolicyId_gte?: BigInt | null
  pricingPolicyId_lt?: BigInt | null
  pricingPolicyId_lte?: BigInt | null
  pricingPolicyId_in?: BigInt[] | BigInt | null
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
  su_eq?: BigInt | null
  su_gt?: BigInt | null
  su_gte?: BigInt | null
  su_lt?: BigInt | null
  su_lte?: BigInt | null
  su_in?: BigInt[] | BigInt | null
  cu_eq?: BigInt | null
  cu_gt?: BigInt | null
  cu_gte?: BigInt | null
  cu_lt?: BigInt | null
  cu_lte?: BigInt | null
  cu_in?: BigInt[] | BigInt | null
  nu_eq?: BigInt | null
  nu_gt?: BigInt | null
  nu_gte?: BigInt | null
  nu_lt?: BigInt | null
  nu_lte?: BigInt | null
  nu_in?: BigInt[] | BigInt | null
}

export interface PricingPolicyWhereUniqueInput {
  id: ID_Output
}

export interface ResourceCreateInput {
  hru?: BigInt | null
  sru?: BigInt | null
  cru?: BigInt | null
  mru?: BigInt | null
}

export interface ResourceUpdateInput {
  hru?: BigInt | null
  sru?: BigInt | null
  cru?: BigInt | null
  mru?: BigInt | null
}

export interface ResourceWhereInput {
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
}

export interface ResourceWhereUniqueInput {
  id: ID_Output
}

export interface TransferCreateInput {
  from: Bytes
  to: Bytes
  value: BigInt
  comment?: String | null
  block: Float
}

export interface TransferUpdateInput {
  from?: Bytes | null
  to?: Bytes | null
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
  from_eq?: Bytes | null
  from_in?: Bytes[] | Bytes | null
  to_eq?: Bytes | null
  to_in?: Bytes[] | Bytes | null
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
  twinId: BigInt
  pubKey: String
  peerId: String
}

export interface TwinUpdateInput {
  twinId?: BigInt | null
  pubKey?: String | null
  peerId?: String | null
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
  twinId_eq?: BigInt | null
  twinId_gt?: BigInt | null
  twinId_gte?: BigInt | null
  twinId_lt?: BigInt | null
  twinId_lte?: BigInt | null
  twinId_in?: BigInt[] | BigInt | null
  pubKey_eq?: String | null
  pubKey_contains?: String | null
  pubKey_startsWith?: String | null
  pubKey_endsWith?: String | null
  pubKey_in?: String[] | String | null
  peerId_eq?: String | null
  peerId_contains?: String | null
  peerId_startsWith?: String | null
  peerId_endsWith?: String | null
  peerId_in?: String[] | String | null
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
  countryId: BigInt
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
  entityId: BigInt
  name: String
  countryId?: BigInt | null
  cityId?: BigInt | null
  pubKey: String
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
  entityId: BigInt
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
  farmId: BigInt
  name: String
  entityId: BigInt
  twinId: BigInt
  pricingPolicyId: BigInt
  certificationType: CertificationType
  countryId?: BigInt | null
  cityId?: BigInt | null
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
  nodeId: BigInt
  farmId: BigInt
  twinId: BigInt
  resources: Resource
  resourcesId: String
  location: Location
  locationId: String
  countryId?: BigInt | null
  cityId?: BigInt | null
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
  pricingPolicyId: BigInt
  name: String
  currency: String
  su: BigInt
  cu: BigInt
  nu: BigInt
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

export interface Resource extends BaseGraphQLObject {
  id: ID_Output
  createdAt: DateTime
  createdById: String
  updatedAt?: DateTime | null
  updatedById?: String | null
  deletedAt?: DateTime | null
  deletedById?: String | null
  version: Int
  hru?: BigInt | null
  sru?: BigInt | null
  cru?: BigInt | null
  mru?: BigInt | null
  noderesources?: Array<Node> | null
}

export interface ResourceConnection {
  totalCount: Int
  edges: Array<ResourceEdge>
  pageInfo: PageInfo
}

export interface ResourceEdge {
  node: Resource
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
  from: Bytes
  to: Bytes
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
  twinId: BigInt
  pubKey: String
  peerId: String
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
GraphQL representation of Bytes
*/
export type Bytes = string

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