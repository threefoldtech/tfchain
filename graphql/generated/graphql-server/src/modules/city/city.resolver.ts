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
  CityCreateInput,
  CityCreateManyArgs,
  CityUpdateArgs,
  CityWhereArgs,
  CityWhereInput,
  CityWhereUniqueInput,
  CityOrderByEnum
} from '../../../generated';

import { City } from './city.model';
import { CityService } from './city.service';

@ObjectType()
export class CityEdge {
  @Field(() => City, { nullable: false })
  node!: City;

  @Field(() => String, { nullable: false })
  cursor!: string;
}

@ObjectType()
export class CityConnection {
  @Field(() => Int, { nullable: false })
  totalCount!: number;

  @Field(() => [CityEdge], { nullable: false })
  edges!: CityEdge[];

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
export class CityConnectionWhereArgs extends ConnectionPageInputOptions {
  @Field(() => CityWhereInput, { nullable: true })
  where?: CityWhereInput;

  @Field(() => CityOrderByEnum, { nullable: true })
  orderBy?: [CityOrderByEnum];
}

@Resolver(City)
export class CityResolver {
  constructor(@Inject('CityService') public readonly service: CityService) {}

  @Query(() => [City])
  async cities(@Args() { where, orderBy, limit, offset }: CityWhereArgs, @Fields() fields: string[]): Promise<City[]> {
    return this.service.find<CityWhereInput>(where, orderBy, limit, offset, fields);
  }

  @Query(() => City, { nullable: true })
  async cityByUniqueInput(@Arg('where') where: CityWhereUniqueInput, @Fields() fields: string[]): Promise<City | null> {
    const result = await this.service.find(where, undefined, 1, 0, fields);
    return result && result.length >= 1 ? result[0] : null;
  }

  @Query(() => CityConnection)
  async citiesConnection(
    @Args() { where, orderBy, ...pageOptions }: CityConnectionWhereArgs,
    @Info() info: any
  ): Promise<CityConnection> {
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
      result = await this.service.findConnection<CityWhereInput>(where, orderBy, pageOptions, rawFields);
    } catch (err) {
      console.log(err);
      // TODO: should continue to return this on `Error: Items is empty` or throw the error
      if (!(err.message as string).includes('Items is empty')) throw err;
    }

    return result as Promise<CityConnection>;
  }
}
