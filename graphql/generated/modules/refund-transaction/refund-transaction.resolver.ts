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
  RefundTransactionCreateInput,
  RefundTransactionCreateManyArgs,
  RefundTransactionUpdateArgs,
  RefundTransactionWhereArgs,
  RefundTransactionWhereInput,
  RefundTransactionWhereUniqueInput,
  RefundTransactionOrderByEnum,
} from '../../warthog';

import { RefundTransaction } from './refund-transaction.model';
import { RefundTransactionService } from './refund-transaction.service';

@ObjectType()
export class RefundTransactionEdge {
  @Field(() => RefundTransaction, { nullable: false })
  node!: RefundTransaction;

  @Field(() => String, { nullable: false })
  cursor!: string;
}

@ObjectType()
export class RefundTransactionConnection {
  @Field(() => Int, { nullable: false })
  totalCount!: number;

  @Field(() => [RefundTransactionEdge], { nullable: false })
  edges!: RefundTransactionEdge[];

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
export class RefundTransactionConnectionWhereArgs extends ConnectionPageInputOptions {
  @Field(() => RefundTransactionWhereInput, { nullable: true })
  where?: RefundTransactionWhereInput;

  @Field(() => RefundTransactionOrderByEnum, { nullable: true })
  orderBy?: [RefundTransactionOrderByEnum];
}

@Resolver(RefundTransaction)
export class RefundTransactionResolver {
  constructor(@Inject('RefundTransactionService') public readonly service: RefundTransactionService) {}

  @Query(() => [RefundTransaction])
  async refundTransactions(
    @Args() { where, orderBy, limit, offset }: RefundTransactionWhereArgs,
    @Fields() fields: string[]
  ): Promise<RefundTransaction[]> {
    return this.service.find<RefundTransactionWhereInput>(where, orderBy, limit, offset, fields);
  }

  @Query(() => RefundTransaction, { nullable: true })
  async refundTransactionByUniqueInput(
    @Arg('where') where: RefundTransactionWhereUniqueInput,
    @Fields() fields: string[]
  ): Promise<RefundTransaction | null> {
    const result = await this.service.find(where, undefined, 1, 0, fields);
    return result && result.length >= 1 ? result[0] : null;
  }

  @Query(() => RefundTransactionConnection)
  async refundTransactionsConnection(
    @Args() { where, orderBy, ...pageOptions }: RefundTransactionConnectionWhereArgs,
    @Info() info: any
  ): Promise<RefundTransactionConnection> {
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
      result = await this.service.findConnection<RefundTransactionWhereInput>(where, orderBy, pageOptions, rawFields);
    } catch (err: any) {
      console.log(err);
      // TODO: should continue to return this on `Error: Items is empty` or throw the error
      if (!(err.message as string).includes('Items is empty')) throw err;
    }

    return result as Promise<RefundTransactionConnection>;
  }
}
