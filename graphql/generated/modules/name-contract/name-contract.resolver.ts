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
  NameContractCreateInput,
  NameContractCreateManyArgs,
  NameContractUpdateArgs,
  NameContractWhereArgs,
  NameContractWhereInput,
  NameContractWhereUniqueInput,
  NameContractOrderByEnum,
} from '../../warthog';

import { NameContract } from './name-contract.model';
import { NameContractService } from './name-contract.service';

@ObjectType()
export class NameContractEdge {
  @Field(() => NameContract, { nullable: false })
  node!: NameContract;

  @Field(() => String, { nullable: false })
  cursor!: string;
}

@ObjectType()
export class NameContractConnection {
  @Field(() => Int, { nullable: false })
  totalCount!: number;

  @Field(() => [NameContractEdge], { nullable: false })
  edges!: NameContractEdge[];

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
export class NameContractConnectionWhereArgs extends ConnectionPageInputOptions {
  @Field(() => NameContractWhereInput, { nullable: true })
  where?: NameContractWhereInput;

  @Field(() => NameContractOrderByEnum, { nullable: true })
  orderBy?: [NameContractOrderByEnum];
}

@Resolver(NameContract)
export class NameContractResolver {
  constructor(@Inject('NameContractService') public readonly service: NameContractService) {}

  @Query(() => [NameContract])
  async nameContracts(
    @Args() { where, orderBy, limit, offset }: NameContractWhereArgs,
    @Fields() fields: string[]
  ): Promise<NameContract[]> {
    return this.service.find<NameContractWhereInput>(where, orderBy, limit, offset, fields);
  }

  @Query(() => NameContract, { nullable: true })
  async nameContractByUniqueInput(
    @Arg('where') where: NameContractWhereUniqueInput,
    @Fields() fields: string[]
  ): Promise<NameContract | null> {
    const result = await this.service.find(where, undefined, 1, 0, fields);
    return result && result.length >= 1 ? result[0] : null;
  }

  @Query(() => NameContractConnection)
  async nameContractsConnection(
    @Args() { where, orderBy, ...pageOptions }: NameContractConnectionWhereArgs,
    @Info() info: any
  ): Promise<NameContractConnection> {
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
      result = await this.service.findConnection<NameContractWhereInput>(where, orderBy, pageOptions, rawFields);
    } catch (err: any) {
      console.log(err);
      // TODO: should continue to return this on `Error: Items is empty` or throw the error
      if (!(err.message as string).includes('Items is empty')) throw err;
    }

    return result as Promise<NameContractConnection>;
  }
}
