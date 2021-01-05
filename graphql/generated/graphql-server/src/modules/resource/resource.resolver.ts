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
  ResourceCreateInput,
  ResourceCreateManyArgs,
  ResourceUpdateArgs,
  ResourceWhereArgs,
  ResourceWhereInput,
  ResourceWhereUniqueInput,
  ResourceOrderByEnum
} from '../../../generated';

import { Resource } from './resource.model';
import { ResourceService } from './resource.service';

import { Node } from '../node/node.model';
import { getConnection } from 'typeorm';

@ObjectType()
export class ResourceEdge {
  @Field(() => Resource, { nullable: false })
  node!: Resource;

  @Field(() => String, { nullable: false })
  cursor!: string;
}

@ObjectType()
export class ResourceConnection {
  @Field(() => Int, { nullable: false })
  totalCount!: number;

  @Field(() => [ResourceEdge], { nullable: false })
  edges!: ResourceEdge[];

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
export class ResourceConnectionWhereArgs extends ConnectionPageInputOptions {
  @Field(() => ResourceWhereInput, { nullable: true })
  where?: ResourceWhereInput;

  @Field(() => ResourceOrderByEnum, { nullable: true })
  orderBy?: ResourceOrderByEnum;
}

@Resolver(Resource)
export class ResourceResolver {
  constructor(@Inject('ResourceService') public readonly service: ResourceService) {}

  @Query(() => [Resource])
  async resources(
    @Args() { where, orderBy, limit, offset }: ResourceWhereArgs,
    @Fields() fields: string[]
  ): Promise<Resource[]> {
    return this.service.find<ResourceWhereInput>(where, orderBy, limit, offset, fields);
  }

  @Query(() => Resource, { nullable: true })
  async resource(@Arg('where') where: ResourceWhereUniqueInput, @Fields() fields: string[]): Promise<Resource | null> {
    const result = await this.service.find(where, undefined, 1, 0, fields);
    return result && result.length >= 1 ? result[0] : null;
  }

  @Query(() => ResourceConnection)
  async resourcesConnection(
    @Args() { where, orderBy, ...pageOptions }: ResourceConnectionWhereArgs,
    @Info() info: any
  ): Promise<ResourceConnection> {
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
      result = await this.service.findConnection<ResourceWhereInput>(where, orderBy, pageOptions, rawFields);
    } catch (err) {
      console.log(err);
      // TODO: should continue to return this on `Error: Items is empty` or throw the error
      if (!(err.message as string).includes('Items is empty')) throw err;
    }

    return result as Promise<ResourceConnection>;
  }

  @FieldResolver(() => Node)
  async noderesources(@Root() r: Resource): Promise<Node[] | null> {
    const result = await getConnection()
      .getRepository(Resource)
      .findOne(r.id, { relations: ['noderesources'] });
    if (result && result.noderesources !== undefined) {
      return result.noderesources;
    }
    return null;
  }
}
