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
  PricingPolicyCreateInput,
  PricingPolicyCreateManyArgs,
  PricingPolicyUpdateArgs,
  PricingPolicyWhereArgs,
  PricingPolicyWhereInput,
  PricingPolicyWhereUniqueInput,
  PricingPolicyOrderByEnum
} from '../../warthog';

import { PricingPolicy } from './pricing-policy.model';
import { PricingPolicyService } from './pricing-policy.service';

import { Policy } from '../policy/policy.model';
import { PolicyService } from '../policy/policy.service';
import { getConnection, getRepository, In, Not } from 'typeorm';
import _ from 'lodash';

@ObjectType()
export class PricingPolicyEdge {
  @Field(() => PricingPolicy, { nullable: false })
  node!: PricingPolicy;

  @Field(() => String, { nullable: false })
  cursor!: string;
}

@ObjectType()
export class PricingPolicyConnection {
  @Field(() => Int, { nullable: false })
  totalCount!: number;

  @Field(() => [PricingPolicyEdge], { nullable: false })
  edges!: PricingPolicyEdge[];

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
export class PricingPolicyConnectionWhereArgs extends ConnectionPageInputOptions {
  @Field(() => PricingPolicyWhereInput, { nullable: true })
  where?: PricingPolicyWhereInput;

  @Field(() => PricingPolicyOrderByEnum, { nullable: true })
  orderBy?: [PricingPolicyOrderByEnum];
}

@Resolver(PricingPolicy)
export class PricingPolicyResolver {
  constructor(@Inject('PricingPolicyService') public readonly service: PricingPolicyService) {}

  @Query(() => [PricingPolicy])
  async pricingPolicies(
    @Args() { where, orderBy, limit, offset }: PricingPolicyWhereArgs,
    @Fields() fields: string[]
  ): Promise<PricingPolicy[]> {
    return this.service.find<PricingPolicyWhereInput>(where, orderBy, limit, offset, fields);
  }

  @Query(() => PricingPolicy, { nullable: true })
  async pricingPolicyByUniqueInput(
    @Arg('where') where: PricingPolicyWhereUniqueInput,
    @Fields() fields: string[]
  ): Promise<PricingPolicy | null> {
    const result = await this.service.find(where, undefined, 1, 0, fields);
    return result && result.length >= 1 ? result[0] : null;
  }

  @Query(() => PricingPolicyConnection)
  async pricingPoliciesConnection(
    @Args() { where, orderBy, ...pageOptions }: PricingPolicyConnectionWhereArgs,
    @Info() info: any
  ): Promise<PricingPolicyConnection> {
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
      result = await this.service.findConnection<PricingPolicyWhereInput>(where, orderBy, pageOptions, rawFields);
    } catch (err) {
      console.log(err);
      // TODO: should continue to return this on `Error: Items is empty` or throw the error
      if (!(err.message as string).includes('Items is empty')) throw err;
    }

    return result as Promise<PricingPolicyConnection>;
  }

  @FieldResolver(() => Policy)
  async su(@Root() r: PricingPolicy, @Ctx() ctx: BaseContext): Promise<Policy | null> {
    return ctx.dataLoader.loaders.PricingPolicy.su.load(r);
  }

  @FieldResolver(() => Policy)
  async cu(@Root() r: PricingPolicy, @Ctx() ctx: BaseContext): Promise<Policy | null> {
    return ctx.dataLoader.loaders.PricingPolicy.cu.load(r);
  }

  @FieldResolver(() => Policy)
  async nu(@Root() r: PricingPolicy, @Ctx() ctx: BaseContext): Promise<Policy | null> {
    return ctx.dataLoader.loaders.PricingPolicy.nu.load(r);
  }

  @FieldResolver(() => Policy)
  async ipu(@Root() r: PricingPolicy, @Ctx() ctx: BaseContext): Promise<Policy | null> {
    return ctx.dataLoader.loaders.PricingPolicy.ipu.load(r);
  }
}
