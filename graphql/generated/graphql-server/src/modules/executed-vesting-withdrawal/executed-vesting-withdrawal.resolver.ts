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
  ExecutedVestingWithdrawalCreateInput,
  ExecutedVestingWithdrawalCreateManyArgs,
  ExecutedVestingWithdrawalUpdateArgs,
  ExecutedVestingWithdrawalWhereArgs,
  ExecutedVestingWithdrawalWhereInput,
  ExecutedVestingWithdrawalWhereUniqueInput,
  ExecutedVestingWithdrawalOrderByEnum
} from '../../../generated';

import { ExecutedVestingWithdrawal } from './executed-vesting-withdrawal.model';
import { ExecutedVestingWithdrawalService } from './executed-vesting-withdrawal.service';

@ObjectType()
export class ExecutedVestingWithdrawalEdge {
  @Field(() => ExecutedVestingWithdrawal, { nullable: false })
  node!: ExecutedVestingWithdrawal;

  @Field(() => String, { nullable: false })
  cursor!: string;
}

@ObjectType()
export class ExecutedVestingWithdrawalConnection {
  @Field(() => Int, { nullable: false })
  totalCount!: number;

  @Field(() => [ExecutedVestingWithdrawalEdge], { nullable: false })
  edges!: ExecutedVestingWithdrawalEdge[];

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
export class ExecutedVestingWithdrawalConnectionWhereArgs extends ConnectionPageInputOptions {
  @Field(() => ExecutedVestingWithdrawalWhereInput, { nullable: true })
  where?: ExecutedVestingWithdrawalWhereInput;

  @Field(() => ExecutedVestingWithdrawalOrderByEnum, { nullable: true })
  orderBy?: ExecutedVestingWithdrawalOrderByEnum;
}

@Resolver(ExecutedVestingWithdrawal)
export class ExecutedVestingWithdrawalResolver {
  constructor(@Inject('ExecutedVestingWithdrawalService') public readonly service: ExecutedVestingWithdrawalService) {}

  @Query(() => [ExecutedVestingWithdrawal])
  async executedVestingWithdrawals(
    @Args() { where, orderBy, limit, offset }: ExecutedVestingWithdrawalWhereArgs,
    @Fields() fields: string[]
  ): Promise<ExecutedVestingWithdrawal[]> {
    return this.service.find<ExecutedVestingWithdrawalWhereInput>(where, orderBy, limit, offset, fields);
  }

  @Query(() => ExecutedVestingWithdrawal, { nullable: true })
  async executedVestingWithdrawal(
    @Arg('where') where: ExecutedVestingWithdrawalWhereUniqueInput,
    @Fields() fields: string[]
  ): Promise<ExecutedVestingWithdrawal | null> {
    const result = await this.service.find(where, undefined, 1, 0, fields);
    return result && result.length >= 1 ? result[0] : null;
  }

  @Query(() => ExecutedVestingWithdrawalConnection)
  async executedVestingWithdrawalsConnection(
    @Args() { where, orderBy, ...pageOptions }: ExecutedVestingWithdrawalConnectionWhereArgs,
    @Info() info: any
  ): Promise<ExecutedVestingWithdrawalConnection> {
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
      result = await this.service.findConnection<ExecutedVestingWithdrawalWhereInput>(
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

    return result as Promise<ExecutedVestingWithdrawalConnection>;
  }
}
