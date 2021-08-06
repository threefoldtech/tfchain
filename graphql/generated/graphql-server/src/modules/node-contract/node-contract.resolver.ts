import {
  Arg,
  Args,
  Mutation,
  Query,
  Root,
  Resolver,
  FieldResolver,
  ObjectType,
  Field,
  Int,
  ArgsType,
  Info,
  Ctx
} from 'type-graphql';
import graphqlFields from 'graphql-fields';
import { Inject } from 'typedi';
import { Min } from 'class-validator';
import { Fields, StandardDeleteResponse, UserId, PageInfo, RawFields, NestedFields, BaseContext } from 'warthog';

import {
  NodeContractCreateInput,
  NodeContractCreateManyArgs,
  NodeContractUpdateArgs,
  NodeContractWhereArgs,
  NodeContractWhereInput,
  NodeContractWhereUniqueInput,
  NodeContractOrderByEnum
} from '../../../generated';

import { NodeContract } from './node-contract.model';
import { NodeContractService } from './node-contract.service';

@ObjectType()
export class NodeContractEdge {
  @Field(() => NodeContract, { nullable: false })
  node!: NodeContract;

  @Field(() => String, { nullable: false })
  cursor!: string;
}

@ObjectType()
export class NodeContractConnection {
  @Field(() => Int, { nullable: false })
  totalCount!: number;

  @Field(() => [NodeContractEdge], { nullable: false })
  edges!: NodeContractEdge[];

  @Field(() => PageInfo, { nullable: false })
  pageInfo!: PageInfo;
}

@ArgsType()
export class ConnectionPageInputOptions {
  @Field(() => Int, { nullable: true })
  @Min(0)
  first?: number;

  @Field(() => String, { nullable: true })
  after?: string; // V3: TODO: should we make a RelayCursor scalar?

  @Field(() => Int, { nullable: true })
  @Min(0)
  last?: number;

  @Field(() => String, { nullable: true })
  before?: string;
}

@ArgsType()
export class NodeContractConnectionWhereArgs extends ConnectionPageInputOptions {
  @Field(() => NodeContractWhereInput, { nullable: true })
  where?: NodeContractWhereInput;

  @Field(() => NodeContractOrderByEnum, { nullable: true })
  orderBy?: [NodeContractOrderByEnum];
}

@Resolver(NodeContract)
export class NodeContractResolver {
  constructor(@Inject('NodeContractService') public readonly service: NodeContractService) {}

  @Query(() => [NodeContract])
  async nodeContracts(
    @Args() { where, orderBy, limit, offset }: NodeContractWhereArgs,
    @Fields() fields: string[]
  ): Promise<NodeContract[]> {
    return this.service.find<NodeContractWhereInput>(where, orderBy, limit, offset, fields);
  }

  @Query(() => NodeContract, { nullable: true })
  async nodeContractByUniqueInput(
    @Arg('where') where: NodeContractWhereUniqueInput,
    @Fields() fields: string[]
  ): Promise<NodeContract | null> {
    const result = await this.service.find(where, undefined, 1, 0, fields);
    return result && result.length >= 1 ? result[0] : null;
  }

  @Query(() => NodeContractConnection)
  async nodeContractsConnection(
    @Args() { where, orderBy, ...pageOptions }: NodeContractConnectionWhereArgs,
    @Info() info: any
  ): Promise<NodeContractConnection> {
    const rawFields = graphqlFields(info, {}, { excludedFields: ['__typename'] });

    let result: any = {
      totalCount: 0,
      edges: [],
      pageInfo: {
        hasNextPage: false,
        hasPreviousPage: false
      }
    };
    // If the related database table does not have any records then an error is thrown to the client
    // by warthog
    try {
      result = await this.service.findConnection<NodeContractWhereInput>(where, orderBy, pageOptions, rawFields);
    } catch (err) {
      console.log(err);
      // TODO: should continue to return this on `Error: Items is empty` or throw the error
      if (!(err.message as string).includes('Items is empty')) throw err;
    }

    return result as Promise<NodeContractConnection>;
  }
}
