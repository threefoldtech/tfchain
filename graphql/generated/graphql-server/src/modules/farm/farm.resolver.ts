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
  FarmCreateInput,
  FarmCreateManyArgs,
  FarmUpdateArgs,
  FarmWhereArgs,
  FarmWhereInput,
  FarmWhereUniqueInput,
  FarmOrderByEnum
} from '../../../generated';

import { Farm } from './farm.model';
import { FarmService } from './farm.service';

@ObjectType()
export class FarmEdge {
  @Field(() => Farm, { nullable: false })
  node!: Farm;

  @Field(() => String, { nullable: false })
  cursor!: string;
}

@ObjectType()
export class FarmConnection {
  @Field(() => Int, { nullable: false })
  totalCount!: number;

  @Field(() => [FarmEdge], { nullable: false })
  edges!: FarmEdge[];

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
export class FarmConnectionWhereArgs extends ConnectionPageInputOptions {
  @Field(() => FarmWhereInput, { nullable: true })
  where?: FarmWhereInput;

  @Field(() => FarmOrderByEnum, { nullable: true })
  orderBy?: [FarmOrderByEnum];
}

@Resolver(Farm)
export class FarmResolver {
  constructor(@Inject('FarmService') public readonly service: FarmService) {}

  @Query(() => [Farm])
  async farms(@Args() { where, orderBy, limit, offset }: FarmWhereArgs, @Fields() fields: string[]): Promise<Farm[]> {
    return this.service.find<FarmWhereInput>(where, orderBy, limit, offset, fields);
  }

  @Query(() => Farm, { nullable: true })
  async farmByUniqueInput(@Arg('where') where: FarmWhereUniqueInput, @Fields() fields: string[]): Promise<Farm | null> {
    const result = await this.service.find(where, undefined, 1, 0, fields);
    return result && result.length >= 1 ? result[0] : null;
  }

  @Query(() => FarmConnection)
  async farmsConnection(
    @Args() { where, orderBy, ...pageOptions }: FarmConnectionWhereArgs,
    @Info() info: any
  ): Promise<FarmConnection> {
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
      result = await this.service.findConnection<FarmWhereInput>(where, orderBy, pageOptions, rawFields);
    } catch (err) {
      console.log(err);
      // TODO: should continue to return this on `Error: Items is empty` or throw the error
      if (!(err.message as string).includes('Items is empty')) throw err;
    }

    return result as Promise<FarmConnection>;
  }
}
