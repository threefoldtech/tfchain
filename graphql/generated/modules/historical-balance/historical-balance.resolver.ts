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
  Ctx,
} from 'type-graphql';
import graphqlFields from 'graphql-fields';
import { Inject } from 'typedi';
import { Min } from 'class-validator';
import {
  Fields,
  StandardDeleteResponse,
  UserId,
  PageInfo,
  RawFields,
  NestedFields,
  BaseContext,
} from '@subsquid/warthog';

import {
  HistoricalBalanceCreateInput,
  HistoricalBalanceCreateManyArgs,
  HistoricalBalanceUpdateArgs,
  HistoricalBalanceWhereArgs,
  HistoricalBalanceWhereInput,
  HistoricalBalanceWhereUniqueInput,
  HistoricalBalanceOrderByEnum,
} from '../../warthog';

import { HistoricalBalance } from './historical-balance.model';
import { HistoricalBalanceService } from './historical-balance.service';

import { Account } from '../account/account.model';
import { AccountService } from '../account/account.service';
import { getConnection, getRepository, In, Not } from 'typeorm';
import _ from 'lodash';

@ObjectType()
export class HistoricalBalanceEdge {
  @Field(() => HistoricalBalance, { nullable: false })
  node!: HistoricalBalance;

  @Field(() => String, { nullable: false })
  cursor!: string;
}

@ObjectType()
export class HistoricalBalanceConnection {
  @Field(() => Int, { nullable: false })
  totalCount!: number;

  @Field(() => [HistoricalBalanceEdge], { nullable: false })
  edges!: HistoricalBalanceEdge[];

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
export class HistoricalBalanceConnectionWhereArgs extends ConnectionPageInputOptions {
  @Field(() => HistoricalBalanceWhereInput, { nullable: true })
  where?: HistoricalBalanceWhereInput;

  @Field(() => HistoricalBalanceOrderByEnum, { nullable: true })
  orderBy?: [HistoricalBalanceOrderByEnum];
}

@Resolver(HistoricalBalance)
export class HistoricalBalanceResolver {
  constructor(@Inject('HistoricalBalanceService') public readonly service: HistoricalBalanceService) {}

  @Query(() => [HistoricalBalance])
  async historicalBalances(
    @Args() { where, orderBy, limit, offset }: HistoricalBalanceWhereArgs,
    @Fields() fields: string[]
  ): Promise<HistoricalBalance[]> {
    return this.service.find<HistoricalBalanceWhereInput>(where, orderBy, limit, offset, fields);
  }

  @Query(() => HistoricalBalance, { nullable: true })
  async historicalBalanceByUniqueInput(
    @Arg('where') where: HistoricalBalanceWhereUniqueInput,
    @Fields() fields: string[]
  ): Promise<HistoricalBalance | null> {
    const result = await this.service.find(where, undefined, 1, 0, fields);
    return result && result.length >= 1 ? result[0] : null;
  }

  @Query(() => HistoricalBalanceConnection)
  async historicalBalancesConnection(
    @Args() { where, orderBy, ...pageOptions }: HistoricalBalanceConnectionWhereArgs,
    @Info() info: any
  ): Promise<HistoricalBalanceConnection> {
    const rawFields = graphqlFields(info, {}, { excludedFields: ['__typename'] });

    let result: any = {
      totalCount: 0,
      edges: [],
      pageInfo: {
        hasNextPage: false,
        hasPreviousPage: false,
      },
    };
    // If the related database table does not have any records then an error is thrown to the client
    // by warthog
    try {
      result = await this.service.findConnection<HistoricalBalanceWhereInput>(where, orderBy, pageOptions, rawFields);
    } catch (err: any) {
      console.log(err);
      // TODO: should continue to return this on `Error: Items is empty` or throw the error
      if (!(err.message as string).includes('Items is empty')) throw err;
    }

    return result as Promise<HistoricalBalanceConnection>;
  }

  @FieldResolver(() => Account)
  async account(@Root() r: HistoricalBalance, @Ctx() ctx: BaseContext): Promise<Account | null> {
    return ctx.dataLoader.loaders.HistoricalBalance.account.load(r);
  }
}
