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
  BlockTimestampCreateInput,
  BlockTimestampCreateManyArgs,
  BlockTimestampUpdateArgs,
  BlockTimestampWhereArgs,
  BlockTimestampWhereInput,
  BlockTimestampWhereUniqueInput,
  BlockTimestampOrderByEnum
} from '../../../generated';

import { BlockTimestamp } from './block-timestamp.model';
import { BlockTimestampService } from './block-timestamp.service';

@ObjectType()
export class BlockTimestampEdge {
  @Field(() => BlockTimestamp, { nullable: false })
  node!: BlockTimestamp;

  @Field(() => String, { nullable: false })
  cursor!: string;
}

@ObjectType()
export class BlockTimestampConnection {
  @Field(() => Int, { nullable: false })
  totalCount!: number;

  @Field(() => [BlockTimestampEdge], { nullable: false })
  edges!: BlockTimestampEdge[];

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
export class BlockTimestampConnectionWhereArgs extends ConnectionPageInputOptions {
  @Field(() => BlockTimestampWhereInput, { nullable: true })
  where?: BlockTimestampWhereInput;

  @Field(() => BlockTimestampOrderByEnum, { nullable: true })
  orderBy?: [BlockTimestampOrderByEnum];
}

@Resolver(BlockTimestamp)
export class BlockTimestampResolver {
  constructor(@Inject('BlockTimestampService') public readonly service: BlockTimestampService) {}

  @Query(() => [BlockTimestamp])
  async blockTimestamps(
    @Args() { where, orderBy, limit, offset }: BlockTimestampWhereArgs,
    @Fields() fields: string[]
  ): Promise<BlockTimestamp[]> {
    return this.service.find<BlockTimestampWhereInput>(where, orderBy, limit, offset, fields);
  }

  @Query(() => BlockTimestamp, { nullable: true })
  async blockTimestampByUniqueInput(
    @Arg('where') where: BlockTimestampWhereUniqueInput,
    @Fields() fields: string[]
  ): Promise<BlockTimestamp | null> {
    const result = await this.service.find(where, undefined, 1, 0, fields);
    return result && result.length >= 1 ? result[0] : null;
  }

  @Query(() => BlockTimestampConnection)
  async blockTimestampsConnection(
    @Args() { where, orderBy, ...pageOptions }: BlockTimestampConnectionWhereArgs,
    @Info() info: any
  ): Promise<BlockTimestampConnection> {
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
      result = await this.service.findConnection<BlockTimestampWhereInput>(where, orderBy, pageOptions, rawFields);
    } catch (err) {
      console.log(err);
      // TODO: should continue to return this on `Error: Items is empty` or throw the error
      if (!(err.message as string).includes('Items is empty')) throw err;
    }

    return result as Promise<BlockTimestampConnection>;
  }
}
