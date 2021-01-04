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
  ResourcesCreateInput,
  ResourcesCreateManyArgs,
  ResourcesUpdateArgs,
  ResourcesWhereArgs,
  ResourcesWhereInput,
  ResourcesWhereUniqueInput,
  ResourcesOrderByEnum
} from '../../../generated';

import { Resources } from './resources.model';
import { ResourcesService } from './resources.service';

import { Node } from '../node/node.model';
import { getConnection } from 'typeorm';

@ObjectType()
export class ResourcesEdge {
  @Field(() => Resources, { nullable: false })
  node!: Resources;

  @Field(() => String, { nullable: false })
  cursor!: string;
}

@ObjectType()
export class ResourcesConnection {
  @Field(() => Int, { nullable: false })
  totalCount!: number;

  @Field(() => [ResourcesEdge], { nullable: false })
  edges!: ResourcesEdge[];

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
export class ResourcesConnectionWhereArgs extends ConnectionPageInputOptions {
  @Field(() => ResourcesWhereInput, { nullable: true })
  where?: ResourcesWhereInput;

  @Field(() => ResourcesOrderByEnum, { nullable: true })
  orderBy?: ResourcesOrderByEnum;
}

@Resolver(Resources)
export class ResourcesResolver {
  constructor(@Inject('ResourcesService') public readonly service: ResourcesService) {}

  @Query(() => [Resources])
  async resources(
    @Args() { where, orderBy, limit, offset }: ResourcesWhereArgs,
    @Fields() fields: string[]
  ): Promise<Resources[]> {
    return this.service.find<ResourcesWhereInput>(where, orderBy, limit, offset, fields);
  }

  @Query(() => Resources, { nullable: true })
  async resources(
    @Arg('where') where: ResourcesWhereUniqueInput,
    @Fields() fields: string[]
  ): Promise<Resources | null> {
    const result = await this.service.find(where, undefined, 1, 0, fields);
    return result && result.length >= 1 ? result[0] : null;
  }

  @Query(() => ResourcesConnection)
  async resourcesConnection(
    @Args() { where, orderBy, ...pageOptions }: ResourcesConnectionWhereArgs,
    @Info() info: any
  ): Promise<ResourcesConnection> {
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
      result = await this.service.findConnection<ResourcesWhereInput>(where, orderBy, pageOptions, rawFields);
    } catch (err) {
      console.log(err);
      // TODO: should continue to return this on `Error: Items is empty` or throw the error
      if (!(err.message as string).includes('Items is empty')) throw err;
    }

    return result as Promise<ResourcesConnection>;
  }

  @FieldResolver(() => Node)
  async noderesources(@Root() r: Resources): Promise<Node[] | null> {
    const result = await getConnection()
      .getRepository(Resources)
      .findOne(r.id, { relations: ['noderesources'] });
    if (result && result.noderesources !== undefined) {
      return result.noderesources;
    }
    return null;
  }
}
