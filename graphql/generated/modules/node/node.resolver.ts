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
  Ctx,
} from 'type-graphql';
import graphqlFields from 'graphql-fields';
import { Inject } from 'typedi';
import { Min } from 'class-validator';
import {
  Fields,
  StandardDeleteResponse,
  UserId,
  PageInfo,
  RawFields,
  NestedFields,
  BaseContext,
} from '@subsquid/warthog';

import {
  NodeCreateInput,
  NodeCreateManyArgs,
  NodeUpdateArgs,
  NodeWhereArgs,
  NodeWhereInput,
  NodeWhereUniqueInput,
  NodeOrderByEnum,
} from '../../warthog';

import { Node } from './node.model';
import { NodeService } from './node.service';

import { Location } from '../location/location.model';
import { LocationService } from '../location/location.service';
import { PublicConfig } from '../public-config/public-config.model';
import { PublicConfigService } from '../public-config/public-config.service';
import { Interfaces } from '../interfaces/interfaces.model';
import { InterfacesService } from '../interfaces/interfaces.service';
import { getConnection, getRepository, In, Not } from 'typeorm';
import _ from 'lodash';

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
  orderBy?: [NodeOrderByEnum];
}

@Resolver(Node)
export class NodeResolver {
  constructor(@Inject('NodeService') public readonly service: NodeService) {}

  @Query(() => [Node])
  async nodes(@Args() { where, orderBy, limit, offset }: NodeWhereArgs, @Fields() fields: string[]): Promise<Node[]> {
    return this.service.find<NodeWhereInput>(where, orderBy, limit, offset, fields);
  }

  @Query(() => Node, { nullable: true })
  async nodeByUniqueInput(@Arg('where') where: NodeWhereUniqueInput, @Fields() fields: string[]): Promise<Node | null> {
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
        hasPreviousPage: false,
      },
    };
    // If the related database table does not have any records then an error is thrown to the client
    // by warthog
    try {
      result = await this.service.findConnection<NodeWhereInput>(where, orderBy, pageOptions, rawFields);
    } catch (err: any) {
      console.log(err);
      // TODO: should continue to return this on `Error: Items is empty` or throw the error
      if (!(err.message as string).includes('Items is empty')) throw err;
    }

    return result as Promise<NodeConnection>;
  }

  @FieldResolver(() => Location)
  async location(@Root() r: Node, @Ctx() ctx: BaseContext): Promise<Location | null> {
    return ctx.dataLoader.loaders.Node.location.load(r);
  }

  @FieldResolver(() => PublicConfig)
  async publicConfig(@Root() r: Node, @Ctx() ctx: BaseContext): Promise<PublicConfig | null> {
    return ctx.dataLoader.loaders.Node.publicConfig.load(r);
  }

  @FieldResolver(() => Interfaces)
  async interfaces(@Root() r: Node, @Ctx() ctx: BaseContext): Promise<Interfaces[] | null> {
    return ctx.dataLoader.loaders.Node.interfaces.load(r);
  }
}
