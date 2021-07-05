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
  PublicConfigCreateInput,
  PublicConfigCreateManyArgs,
  PublicConfigUpdateArgs,
  PublicConfigWhereArgs,
  PublicConfigWhereInput,
  PublicConfigWhereUniqueInput,
  PublicConfigOrderByEnum
} from '../../../generated';

import { PublicConfig } from './public-config.model';
import { PublicConfigService } from './public-config.service';

import { Node } from '../node/node.model';
import { NodeService } from '../node/node.service';
import { getConnection, getRepository, In, Not } from 'typeorm';
import _ from 'lodash';

@ObjectType()
export class PublicConfigEdge {
  @Field(() => PublicConfig, { nullable: false })
  node!: PublicConfig;

  @Field(() => String, { nullable: false })
  cursor!: string;
}

@ObjectType()
export class PublicConfigConnection {
  @Field(() => Int, { nullable: false })
  totalCount!: number;

  @Field(() => [PublicConfigEdge], { nullable: false })
  edges!: PublicConfigEdge[];

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
export class PublicConfigConnectionWhereArgs extends ConnectionPageInputOptions {
  @Field(() => PublicConfigWhereInput, { nullable: true })
  where?: PublicConfigWhereInput;

  @Field(() => PublicConfigOrderByEnum, { nullable: true })
  orderBy?: [PublicConfigOrderByEnum];
}

@Resolver(PublicConfig)
export class PublicConfigResolver {
  constructor(@Inject('PublicConfigService') public readonly service: PublicConfigService) {}

  @Query(() => [PublicConfig])
  async publicConfigs(
    @Args() { where, orderBy, limit, offset }: PublicConfigWhereArgs,
    @Fields() fields: string[]
  ): Promise<PublicConfig[]> {
    return this.service.find<PublicConfigWhereInput>(where, orderBy, limit, offset, fields);
  }

  @Query(() => PublicConfig, { nullable: true })
  async publicConfigByUniqueInput(
    @Arg('where') where: PublicConfigWhereUniqueInput,
    @Fields() fields: string[]
  ): Promise<PublicConfig | null> {
    const result = await this.service.find(where, undefined, 1, 0, fields);
    return result && result.length >= 1 ? result[0] : null;
  }

  @Query(() => PublicConfigConnection)
  async publicConfigsConnection(
    @Args() { where, orderBy, ...pageOptions }: PublicConfigConnectionWhereArgs,
    @Info() info: any
  ): Promise<PublicConfigConnection> {
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
      result = await this.service.findConnection<PublicConfigWhereInput>(where, orderBy, pageOptions, rawFields);
    } catch (err) {
      console.log(err);
      // TODO: should continue to return this on `Error: Items is empty` or throw the error
      if (!(err.message as string).includes('Items is empty')) throw err;
    }

    return result as Promise<PublicConfigConnection>;
  }

  @FieldResolver(() => Node)
  async nodepublicConfig(@Root() r: PublicConfig, @Ctx() ctx: BaseContext): Promise<Node[] | null> {
    return ctx.dataLoader.loaders.PublicConfig.nodepublicConfig.load(r);
  }
}
