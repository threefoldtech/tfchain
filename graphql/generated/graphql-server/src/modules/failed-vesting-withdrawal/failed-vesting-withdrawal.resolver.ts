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
  FailedVestingWithdrawalCreateInput,
  FailedVestingWithdrawalCreateManyArgs,
  FailedVestingWithdrawalUpdateArgs,
  FailedVestingWithdrawalWhereArgs,
  FailedVestingWithdrawalWhereInput,
  FailedVestingWithdrawalWhereUniqueInput,
  FailedVestingWithdrawalOrderByEnum
} from '../../../generated';

import { FailedVestingWithdrawal } from './failed-vesting-withdrawal.model';
import { FailedVestingWithdrawalService } from './failed-vesting-withdrawal.service';

@ObjectType()
export class FailedVestingWithdrawalEdge {
  @Field(() => FailedVestingWithdrawal, { nullable: false })
  node!: FailedVestingWithdrawal;

  @Field(() => String, { nullable: false })
  cursor!: string;
}

@ObjectType()
export class FailedVestingWithdrawalConnection {
  @Field(() => Int, { nullable: false })
  totalCount!: number;

  @Field(() => [FailedVestingWithdrawalEdge], { nullable: false })
  edges!: FailedVestingWithdrawalEdge[];

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
export class FailedVestingWithdrawalConnectionWhereArgs extends ConnectionPageInputOptions {
  @Field(() => FailedVestingWithdrawalWhereInput, { nullable: true })
  where?: FailedVestingWithdrawalWhereInput;

  @Field(() => FailedVestingWithdrawalOrderByEnum, { nullable: true })
  orderBy?: FailedVestingWithdrawalOrderByEnum;
}

@Resolver(FailedVestingWithdrawal)
export class FailedVestingWithdrawalResolver {
  constructor(@Inject('FailedVestingWithdrawalService') public readonly service: FailedVestingWithdrawalService) {}

  @Query(() => [FailedVestingWithdrawal])
  async failedVestingWithdrawals(
    @Args() { where, orderBy, limit, offset }: FailedVestingWithdrawalWhereArgs,
    @Fields() fields: string[]
  ): Promise<FailedVestingWithdrawal[]> {
    return this.service.find<FailedVestingWithdrawalWhereInput>(where, orderBy, limit, offset, fields);
  }

  @Query(() => FailedVestingWithdrawal, { nullable: true })
  async failedVestingWithdrawal(
    @Arg('where') where: FailedVestingWithdrawalWhereUniqueInput,
    @Fields() fields: string[]
  ): Promise<FailedVestingWithdrawal | null> {
    const result = await this.service.find(where, undefined, 1, 0, fields);
    return result && result.length >= 1 ? result[0] : null;
  }

  @Query(() => FailedVestingWithdrawalConnection)
  async failedVestingWithdrawalsConnection(
    @Args() { where, orderBy, ...pageOptions }: FailedVestingWithdrawalConnectionWhereArgs,
    @Info() info: any
  ): Promise<FailedVestingWithdrawalConnection> {
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
      result = await this.service.findConnection<FailedVestingWithdrawalWhereInput>(
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

    return result as Promise<FailedVestingWithdrawalConnection>;
  }
}
