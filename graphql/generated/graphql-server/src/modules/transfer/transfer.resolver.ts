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
  TransferCreateInput,
  TransferCreateManyArgs,
  TransferUpdateArgs,
  TransferWhereArgs,
  TransferWhereInput,
  TransferWhereUniqueInput,
  TransferOrderByEnum
} from '../../../generated';

import { Transfer } from './transfer.model';
import { TransferService } from './transfer.service';

@ObjectType()
export class TransferEdge {
  @Field(() => Transfer, { nullable: false })
  node!: Transfer;

  @Field(() => String, { nullable: false })
  cursor!: string;
}

@ObjectType()
export class TransferConnection {
  @Field(() => Int, { nullable: false })
  totalCount!: number;

  @Field(() => [TransferEdge], { nullable: false })
  edges!: TransferEdge[];

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
export class TransferConnectionWhereArgs extends ConnectionPageInputOptions {
  @Field(() => TransferWhereInput, { nullable: true })
  where?: TransferWhereInput;

  @Field(() => TransferOrderByEnum, { nullable: true })
  orderBy?: TransferOrderByEnum;
}

@Resolver(Transfer)
export class TransferResolver {
  constructor(@Inject('TransferService') public readonly service: TransferService) {}

  @Query(() => [Transfer])
  async transfers(
    @Args() { where, orderBy, limit, offset }: TransferWhereArgs,
    @Fields() fields: string[]
  ): Promise<Transfer[]> {
    return this.service.find<TransferWhereInput>(where, orderBy, limit, offset, fields);
  }

  @Query(() => Transfer, { nullable: true })
  async transfer(@Arg('where') where: TransferWhereUniqueInput, @Fields() fields: string[]): Promise<Transfer | null> {
    const result = await this.service.find(where, undefined, 1, 0, fields);
    return result && result.length >= 1 ? result[0] : null;
  }

  @Query(() => TransferConnection)
  async transfersConnection(
    @Args() { where, orderBy, ...pageOptions }: TransferConnectionWhereArgs,
    @Info() info: any
  ): Promise<TransferConnection> {
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
      result = await this.service.findConnection<TransferWhereInput>(where, orderBy, pageOptions, rawFields);
    } catch (err) {
      console.log(err);
      // TODO: should continue to return this on `Error: Items is empty` or throw the error
      if (!(err.message as string).includes('Items is empty')) throw err;
    }

    return result as Promise<TransferConnection>;
  }
}
