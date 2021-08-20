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
  FarmingPolicyCreateInput,
  FarmingPolicyCreateManyArgs,
  FarmingPolicyUpdateArgs,
  FarmingPolicyWhereArgs,
  FarmingPolicyWhereInput,
  FarmingPolicyWhereUniqueInput,
  FarmingPolicyOrderByEnum
} from '../../warthog';

import { FarmingPolicy } from './farming-policy.model';
import { FarmingPolicyService } from './farming-policy.service';

@ObjectType()
export class FarmingPolicyEdge {
  @Field(() => FarmingPolicy, { nullable: false })
  node!: FarmingPolicy;

  @Field(() => String, { nullable: false })
  cursor!: string;
}

@ObjectType()
export class FarmingPolicyConnection {
  @Field(() => Int, { nullable: false })
  totalCount!: number;

  @Field(() => [FarmingPolicyEdge], { nullable: false })
  edges!: FarmingPolicyEdge[];

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
export class FarmingPolicyConnectionWhereArgs extends ConnectionPageInputOptions {
  @Field(() => FarmingPolicyWhereInput, { nullable: true })
  where?: FarmingPolicyWhereInput;

  @Field(() => FarmingPolicyOrderByEnum, { nullable: true })
  orderBy?: [FarmingPolicyOrderByEnum];
}

@Resolver(FarmingPolicy)
export class FarmingPolicyResolver {
  constructor(@Inject('FarmingPolicyService') public readonly service: FarmingPolicyService) {}

  @Query(() => [FarmingPolicy])
  async farmingPolicies(
    @Args() { where, orderBy, limit, offset }: FarmingPolicyWhereArgs,
    @Fields() fields: string[]
  ): Promise<FarmingPolicy[]> {
    return this.service.find<FarmingPolicyWhereInput>(where, orderBy, limit, offset, fields);
  }

  @Query(() => FarmingPolicy, { nullable: true })
  async farmingPolicyByUniqueInput(
    @Arg('where') where: FarmingPolicyWhereUniqueInput,
    @Fields() fields: string[]
  ): Promise<FarmingPolicy | null> {
    const result = await this.service.find(where, undefined, 1, 0, fields);
    return result && result.length >= 1 ? result[0] : null;
  }

  @Query(() => FarmingPolicyConnection)
  async farmingPoliciesConnection(
    @Args() { where, orderBy, ...pageOptions }: FarmingPolicyConnectionWhereArgs,
    @Info() info: any
  ): Promise<FarmingPolicyConnection> {
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
      result = await this.service.findConnection<FarmingPolicyWhereInput>(where, orderBy, pageOptions, rawFields);
    } catch (err) {
      console.log(err);
      // TODO: should continue to return this on `Error: Items is empty` or throw the error
      if (!(err.message as string).includes('Items is empty')) throw err;
    }

    return result as Promise<FarmingPolicyConnection>;
  }
}
