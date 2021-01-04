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
  Info
} from 'type-graphql';
import graphqlFields from 'graphql-fields';
import { Inject } from 'typedi';
import { Min } from 'class-validator';
import { Fields, StandardDeleteResponse, UserId, PageInfo, RawFields } from 'warthog';

import {
  NodeCreateInput,
  NodeCreateManyArgs,
  NodeUpdateArgs,
  NodeWhereArgs,
  NodeWhereInput,
  NodeWhereUniqueInput,
  NodeOrderByEnum
} from '../../../generated';

import { Node } from './node.model';
import { NodeService } from './node.service';

import { Resources } from '../resources/resources.model';
import { Location } from '../location/location.model';
import { getConnection } from 'typeorm';

@ObjectType()
export class NodeEdge {
  @Field(() => Node, { nullable: false })
  node!: Node;

  @Field(() => String, { nullable: false })
  cursor!: string;
}

@ObjectType()
export class NodeConnection {
  @Field(() => Int, { nullable: false })
  totalCount!: number;

  @Field(() => [NodeEdge], { nullable: false })
  edges!: NodeEdge[];

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
export class NodeConnectionWhereArgs extends ConnectionPageInputOptions {
  @Field(() => NodeWhereInput, { nullable: true })
  where?: NodeWhereInput;

  @Field(() => NodeOrderByEnum, { nullable: true })
  orderBy?: NodeOrderByEnum;
}

@Resolver(Node)
export class NodeResolver {
  constructor(@Inject('NodeService') public readonly service: NodeService) {}

  @Query(() => [Node])
  async nodes(@Args() { where, orderBy, limit, offset }: NodeWhereArgs, @Fields() fields: string[]): Promise<Node[]> {
    return this.service.find<NodeWhereInput>(where, orderBy, limit, offset, fields);
  }

  @Query(() => Node, { nullable: true })
  async node(@Arg('where') where: NodeWhereUniqueInput, @Fields() fields: string[]): Promise<Node | null> {
    const result = await this.service.find(where, undefined, 1, 0, fields);
    return result && result.length >= 1 ? result[0] : null;
  }

  @Query(() => NodeConnection)
  async nodesConnection(
    @Args() { where, orderBy, ...pageOptions }: NodeConnectionWhereArgs,
    @Info() info: any
  ): Promise<NodeConnection> {
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
      result = await this.service.findConnection<NodeWhereInput>(where, orderBy, pageOptions, rawFields);
    } catch (err) {
      console.log(err);
      // TODO: should continue to return this on `Error: Items is empty` or throw the error
      if (!(err.message as string).includes('Items is empty')) throw err;
    }

    return result as Promise<NodeConnection>;
  }

  @FieldResolver(() => Resources)
  async resources(@Root() r: Node): Promise<Resources | null> {
    const result = await getConnection()
      .getRepository(Node)
      .findOne(r.id, { relations: ['resources'] });
    if (result && result.resources !== undefined) {
      return result.resources;
    }
    return null;
  }
  @FieldResolver(() => Location)
  async location(@Root() r: Node): Promise<Location | null> {
    const result = await getConnection()
      .getRepository(Node)
      .findOne(r.id, { relations: ['location'] });
    if (result && result.location !== undefined) {
      return result.location;
    }
    return null;
  }
}
