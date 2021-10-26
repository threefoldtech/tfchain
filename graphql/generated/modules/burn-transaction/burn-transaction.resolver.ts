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
  BurnTransactionCreateInput,
  BurnTransactionCreateManyArgs,
  BurnTransactionUpdateArgs,
  BurnTransactionWhereArgs,
  BurnTransactionWhereInput,
  BurnTransactionWhereUniqueInput,
  BurnTransactionOrderByEnum,
} from '../../warthog';

import { BurnTransaction } from './burn-transaction.model';
import { BurnTransactionService } from './burn-transaction.service';

@ObjectType()
export class BurnTransactionEdge {
  @Field(() => BurnTransaction, { nullable: false })
  node!: BurnTransaction;

  @Field(() => String, { nullable: false })
  cursor!: string;
}

@ObjectType()
export class BurnTransactionConnection {
  @Field(() => Int, { nullable: false })
  totalCount!: number;

  @Field(() => [BurnTransactionEdge], { nullable: false })
  edges!: BurnTransactionEdge[];

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
export class BurnTransactionConnectionWhereArgs extends ConnectionPageInputOptions {
  @Field(() => BurnTransactionWhereInput, { nullable: true })
  where?: BurnTransactionWhereInput;

  @Field(() => BurnTransactionOrderByEnum, { nullable: true })
  orderBy?: [BurnTransactionOrderByEnum];
}

@Resolver(BurnTransaction)
export class BurnTransactionResolver {
  constructor(@Inject('BurnTransactionService') public readonly service: BurnTransactionService) {}

  @Query(() => [BurnTransaction])
  async burnTransactions(
    @Args() { where, orderBy, limit, offset }: BurnTransactionWhereArgs,
    @Fields() fields: string[]
  ): Promise<BurnTransaction[]> {
    return this.service.find<BurnTransactionWhereInput>(where, orderBy, limit, offset, fields);
  }

  @Query(() => BurnTransaction, { nullable: true })
  async burnTransactionByUniqueInput(
    @Arg('where') where: BurnTransactionWhereUniqueInput,
    @Fields() fields: string[]
  ): Promise<BurnTransaction | null> {
    const result = await this.service.find(where, undefined, 1, 0, fields);
    return result && result.length >= 1 ? result[0] : null;
  }

  @Query(() => BurnTransactionConnection)
  async burnTransactionsConnection(
    @Args() { where, orderBy, ...pageOptions }: BurnTransactionConnectionWhereArgs,
    @Info() info: any
  ): Promise<BurnTransactionConnection> {
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
      result = await this.service.findConnection<BurnTransactionWhereInput>(where, orderBy, pageOptions, rawFields);
    } catch (err: any) {
      console.log(err);
      // TODO: should continue to return this on `Error: Items is empty` or throw the error
      if (!(err.message as string).includes('Items is empty')) throw err;
    }

    return result as Promise<BurnTransactionConnection>;
  }
}
