import 'graphql-import-node'; // Needed so you can import *.graphql files 

import { makeBindingClass, Options } from 'graphql-binding'
import { GraphQLResolveInfo, GraphQLSchema } from 'graphql'
import { IResolvers } from 'graphql-tools/dist/Interfaces'
import * as schema from  './schema.graphql'

export interface Query {
    entities: <T = Array<Entity>>(args: { offset?: Int | null, limit?: Int | null, where?: EntityWhereInput | null, orderBy?: EntityOrderByInput | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    entity: <T = Entity | null>(args: { where: EntityWhereUniqueInput }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T | null> ,
    entitiesConnection: <T = EntityConnection>(args: { first?: Int | null, after?: String | null, last?: Int | null, before?: String | null, where?: EntityWhereInput | null, orderBy?: EntityOrderByInput | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    commentSearch: <T = Array<CommentSearchFTSOutput>>(args: { limit?: Int | null, text: String }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    transfers: <T = Array<Transfer>>(args: { offset?: Int | null, limit?: Int | null, where?: TransferWhereInput | null, orderBy?: TransferOrderByInput | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> ,
    transfer: <T = Transfer | null>(args: { where: TransferWhereUniqueInput }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T | null> ,
    transfersConnection: <T = TransferConnection>(args: { first?: Int | null, after?: String | null, last?: Int | null, before?: String | null, where?: TransferWhereInput | null, orderBy?: TransferOrderByInput | null }, info?: GraphQLResolveInfo | string, options?: Options) => Promise<T> 
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

export interface EntityCreateInput {
  entityId: BigInt
  name?: String | null
  countryId: BigInt
  cityId: BigInt
  pubKey?: String | null
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

export interface CommentSearchFTSOutput {
  item: CommentSearchSearchResult
  rank: Float
  isTypeOf: String
  highlight: String
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
  name?: String | null
  countryId: BigInt
  cityId: BigInt
  pubKey?: String | null
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

export interface PageInfo {
  hasNextPage: Boolean
  hasPreviousPage: Boolean
  startCursor?: String | null
  endCursor?: String | null
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