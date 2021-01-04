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
  LocationCreateInput,
  LocationCreateManyArgs,
  LocationUpdateArgs,
  LocationWhereArgs,
  LocationWhereInput,
  LocationWhereUniqueInput,
  LocationOrderByEnum
} from '../../../generated';

import { Location } from './location.model';
import { LocationService } from './location.service';

import { Node } from '../node/node.model';
import { getConnection } from 'typeorm';

@ObjectType()
export class LocationEdge {
  @Field(() => Location, { nullable: false })
  node!: Location;

  @Field(() => String, { nullable: false })
  cursor!: string;
}

@ObjectType()
export class LocationConnection {
  @Field(() => Int, { nullable: false })
  totalCount!: number;

  @Field(() => [LocationEdge], { nullable: false })
  edges!: LocationEdge[];

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
export class LocationConnectionWhereArgs extends ConnectionPageInputOptions {
  @Field(() => LocationWhereInput, { nullable: true })
  where?: LocationWhereInput;

  @Field(() => LocationOrderByEnum, { nullable: true })
  orderBy?: LocationOrderByEnum;
}

@Resolver(Location)
export class LocationResolver {
  constructor(@Inject('LocationService') public readonly service: LocationService) {}

  @Query(() => [Location])
  async locations(
    @Args() { where, orderBy, limit, offset }: LocationWhereArgs,
    @Fields() fields: string[]
  ): Promise<Location[]> {
    return this.service.find<LocationWhereInput>(where, orderBy, limit, offset, fields);
  }

  @Query(() => Location, { nullable: true })
  async location(@Arg('where') where: LocationWhereUniqueInput, @Fields() fields: string[]): Promise<Location | null> {
    const result = await this.service.find(where, undefined, 1, 0, fields);
    return result && result.length >= 1 ? result[0] : null;
  }

  @Query(() => LocationConnection)
  async locationsConnection(
    @Args() { where, orderBy, ...pageOptions }: LocationConnectionWhereArgs,
    @Info() info: any
  ): Promise<LocationConnection> {
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
      result = await this.service.findConnection<LocationWhereInput>(where, orderBy, pageOptions, rawFields);
    } catch (err) {
      console.log(err);
      // TODO: should continue to return this on `Error: Items is empty` or throw the error
      if (!(err.message as string).includes('Items is empty')) throw err;
    }

    return result as Promise<LocationConnection>;
  }

  @FieldResolver(() => Node)
  async nodelocation(@Root() r: Location): Promise<Node[] | null> {
    const result = await getConnection()
      .getRepository(Location)
      .findOne(r.id, { relations: ['nodelocation'] });
    if (result && result.nodelocation !== undefined) {
      return result.nodelocation;
    }
    return null;
  }
}
