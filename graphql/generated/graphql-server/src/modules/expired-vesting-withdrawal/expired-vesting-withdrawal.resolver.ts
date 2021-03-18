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
  ExpiredVestingWithdrawalCreateInput,
  ExpiredVestingWithdrawalCreateManyArgs,
  ExpiredVestingWithdrawalUpdateArgs,
  ExpiredVestingWithdrawalWhereArgs,
  ExpiredVestingWithdrawalWhereInput,
  ExpiredVestingWithdrawalWhereUniqueInput,
  ExpiredVestingWithdrawalOrderByEnum
} from '../../../generated';

import { ExpiredVestingWithdrawal } from './expired-vesting-withdrawal.model';
import { ExpiredVestingWithdrawalService } from './expired-vesting-withdrawal.service';

@ObjectType()
export class ExpiredVestingWithdrawalEdge {
  @Field(() => ExpiredVestingWithdrawal, { nullable: false })
  node!: ExpiredVestingWithdrawal;

  @Field(() => String, { nullable: false })
  cursor!: string;
}

@ObjectType()
export class ExpiredVestingWithdrawalConnection {
  @Field(() => Int, { nullable: false })
  totalCount!: number;

  @Field(() => [ExpiredVestingWithdrawalEdge], { nullable: false })
  edges!: ExpiredVestingWithdrawalEdge[];

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
export class ExpiredVestingWithdrawalConnectionWhereArgs extends ConnectionPageInputOptions {
  @Field(() => ExpiredVestingWithdrawalWhereInput, { nullable: true })
  where?: ExpiredVestingWithdrawalWhereInput;

  @Field(() => ExpiredVestingWithdrawalOrderByEnum, { nullable: true })
  orderBy?: ExpiredVestingWithdrawalOrderByEnum;
}

@Resolver(ExpiredVestingWithdrawal)
export class ExpiredVestingWithdrawalResolver {
  constructor(@Inject('ExpiredVestingWithdrawalService') public readonly service: ExpiredVestingWithdrawalService) {}

  @Query(() => [ExpiredVestingWithdrawal])
  async expiredVestingWithdrawals(
    @Args() { where, orderBy, limit, offset }: ExpiredVestingWithdrawalWhereArgs,
    @Fields() fields: string[]
  ): Promise<ExpiredVestingWithdrawal[]> {
    return this.service.find<ExpiredVestingWithdrawalWhereInput>(where, orderBy, limit, offset, fields);
  }

  @Query(() => ExpiredVestingWithdrawal, { nullable: true })
  async expiredVestingWithdrawal(
    @Arg('where') where: ExpiredVestingWithdrawalWhereUniqueInput,
    @Fields() fields: string[]
  ): Promise<ExpiredVestingWithdrawal | null> {
    const result = await this.service.find(where, undefined, 1, 0, fields);
    return result && result.length >= 1 ? result[0] : null;
  }

  @Query(() => ExpiredVestingWithdrawalConnection)
  async expiredVestingWithdrawalsConnection(
    @Args() { where, orderBy, ...pageOptions }: ExpiredVestingWithdrawalConnectionWhereArgs,
    @Info() info: any
  ): Promise<ExpiredVestingWithdrawalConnection> {
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
      result = await this.service.findConnection<ExpiredVestingWithdrawalWhereInput>(
        where,
        orderBy,
        pageOptions,
        rawFields
      );
    } catch (err) {
      console.log(err);
      // TODO: should continue to return this on `Error: Items is empty` or throw the error
      if (!(err.message as string).includes('Items is empty')) throw err;
    }

    return result as Promise<ExpiredVestingWithdrawalConnection>;
  }
}
