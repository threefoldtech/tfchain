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
  MintTransactionCreateInput,
  MintTransactionCreateManyArgs,
  MintTransactionUpdateArgs,
  MintTransactionWhereArgs,
  MintTransactionWhereInput,
  MintTransactionWhereUniqueInput,
  MintTransactionOrderByEnum,
} from '../../warthog';

import { MintTransaction } from './mint-transaction.model';
import { MintTransactionService } from './mint-transaction.service';

@ObjectType()
export class MintTransactionEdge {
  @Field(() => MintTransaction, { nullable: false })
  node!: MintTransaction;

  @Field(() => String, { nullable: false })
  cursor!: string;
}

@ObjectType()
export class MintTransactionConnection {
  @Field(() => Int, { nullable: false })
  totalCount!: number;

  @Field(() => [MintTransactionEdge], { nullable: false })
  edges!: MintTransactionEdge[];

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
export class MintTransactionConnectionWhereArgs extends ConnectionPageInputOptions {
  @Field(() => MintTransactionWhereInput, { nullable: true })
  where?: MintTransactionWhereInput;

  @Field(() => MintTransactionOrderByEnum, { nullable: true })
  orderBy?: [MintTransactionOrderByEnum];
}

@Resolver(MintTransaction)
export class MintTransactionResolver {
  constructor(@Inject('MintTransactionService') public readonly service: MintTransactionService) {}

  @Query(() => [MintTransaction])
  async mintTransactions(
    @Args() { where, orderBy, limit, offset }: MintTransactionWhereArgs,
    @Fields() fields: string[]
  ): Promise<MintTransaction[]> {
    return this.service.find<MintTransactionWhereInput>(where, orderBy, limit, offset, fields);
  }

  @Query(() => MintTransaction, { nullable: true })
  async mintTransactionByUniqueInput(
    @Arg('where') where: MintTransactionWhereUniqueInput,
    @Fields() fields: string[]
  ): Promise<MintTransaction | null> {
    const result = await this.service.find(where, undefined, 1, 0, fields);
    return result && result.length >= 1 ? result[0] : null;
  }

  @Query(() => MintTransactionConnection)
  async mintTransactionsConnection(
    @Args() { where, orderBy, ...pageOptions }: MintTransactionConnectionWhereArgs,
    @Info() info: any
  ): Promise<MintTransactionConnection> {
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
      result = await this.service.findConnection<MintTransactionWhereInput>(where, orderBy, pageOptions, rawFields);
    } catch (err: any) {
      console.log(err);
      // TODO: should continue to return this on `Error: Items is empty` or throw the error
      if (!(err.message as string).includes('Items is empty')) throw err;
    }

    return result as Promise<MintTransactionConnection>;
  }
}
