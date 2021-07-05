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
  CountryCreateInput,
  CountryCreateManyArgs,
  CountryUpdateArgs,
  CountryWhereArgs,
  CountryWhereInput,
  CountryWhereUniqueInput,
  CountryOrderByEnum
} from '../../../generated';

import { Country } from './country.model';
import { CountryService } from './country.service';

@ObjectType()
export class CountryEdge {
  @Field(() => Country, { nullable: false })
  node!: Country;

  @Field(() => String, { nullable: false })
  cursor!: string;
}

@ObjectType()
export class CountryConnection {
  @Field(() => Int, { nullable: false })
  totalCount!: number;

  @Field(() => [CountryEdge], { nullable: false })
  edges!: CountryEdge[];

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
export class CountryConnectionWhereArgs extends ConnectionPageInputOptions {
  @Field(() => CountryWhereInput, { nullable: true })
  where?: CountryWhereInput;

  @Field(() => CountryOrderByEnum, { nullable: true })
  orderBy?: [CountryOrderByEnum];
}

@Resolver(Country)
export class CountryResolver {
  constructor(@Inject('CountryService') public readonly service: CountryService) {}

  @Query(() => [Country])
  async countries(
    @Args() { where, orderBy, limit, offset }: CountryWhereArgs,
    @Fields() fields: string[]
  ): Promise<Country[]> {
    return this.service.find<CountryWhereInput>(where, orderBy, limit, offset, fields);
  }

  @Query(() => Country, { nullable: true })
  async countryByUniqueInput(
    @Arg('where') where: CountryWhereUniqueInput,
    @Fields() fields: string[]
  ): Promise<Country | null> {
    const result = await this.service.find(where, undefined, 1, 0, fields);
    return result && result.length >= 1 ? result[0] : null;
  }

  @Query(() => CountryConnection)
  async countriesConnection(
    @Args() { where, orderBy, ...pageOptions }: CountryConnectionWhereArgs,
    @Info() info: any
  ): Promise<CountryConnection> {
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
      result = await this.service.findConnection<CountryWhereInput>(where, orderBy, pageOptions, rawFields);
    } catch (err) {
      console.log(err);
      // TODO: should continue to return this on `Error: Items is empty` or throw the error
      if (!(err.message as string).includes('Items is empty')) throw err;
    }

    return result as Promise<CountryConnection>;
  }
}
