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
  PolicyCreateInput,
  PolicyCreateManyArgs,
  PolicyUpdateArgs,
  PolicyWhereArgs,
  PolicyWhereInput,
  PolicyWhereUniqueInput,
  PolicyOrderByEnum,
} from '../../warthog';

import { Policy } from './policy.model';
import { PolicyService } from './policy.service';

import { PricingPolicy } from '../pricing-policy/pricing-policy.model';
import { PricingPolicyService } from '../pricing-policy/pricing-policy.service';
import { getConnection, getRepository, In, Not } from 'typeorm';
import _ from 'lodash';

@ObjectType()
export class PolicyEdge {
  @Field(() => Policy, { nullable: false })
  node!: Policy;

  @Field(() => String, { nullable: false })
  cursor!: string;
}

@ObjectType()
export class PolicyConnection {
  @Field(() => Int, { nullable: false })
  totalCount!: number;

  @Field(() => [PolicyEdge], { nullable: false })
  edges!: PolicyEdge[];

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
export class PolicyConnectionWhereArgs extends ConnectionPageInputOptions {
  @Field(() => PolicyWhereInput, { nullable: true })
  where?: PolicyWhereInput;

  @Field(() => PolicyOrderByEnum, { nullable: true })
  orderBy?: [PolicyOrderByEnum];
}

@Resolver(Policy)
export class PolicyResolver {
  constructor(@Inject('PolicyService') public readonly service: PolicyService) {}

  @Query(() => [Policy])
  async policies(
    @Args() { where, orderBy, limit, offset }: PolicyWhereArgs,
    @Fields() fields: string[]
  ): Promise<Policy[]> {
    return this.service.find<PolicyWhereInput>(where, orderBy, limit, offset, fields);
  }

  @Query(() => Policy, { nullable: true })
  async policyByUniqueInput(
    @Arg('where') where: PolicyWhereUniqueInput,
    @Fields() fields: string[]
  ): Promise<Policy | null> {
    const result = await this.service.find(where, undefined, 1, 0, fields);
    return result && result.length >= 1 ? result[0] : null;
  }

  @Query(() => PolicyConnection)
  async policiesConnection(
    @Args() { where, orderBy, ...pageOptions }: PolicyConnectionWhereArgs,
    @Info() info: any
  ): Promise<PolicyConnection> {
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
      result = await this.service.findConnection<PolicyWhereInput>(where, orderBy, pageOptions, rawFields);
    } catch (err: any) {
      console.log(err);
      // TODO: should continue to return this on `Error: Items is empty` or throw the error
      if (!(err.message as string).includes('Items is empty')) throw err;
    }

    return result as Promise<PolicyConnection>;
  }

  @FieldResolver(() => PricingPolicy)
  async pricingpolicysu(@Root() r: Policy, @Ctx() ctx: BaseContext): Promise<PricingPolicy[] | null> {
    return ctx.dataLoader.loaders.Policy.pricingpolicysu.load(r);
  }

  @FieldResolver(() => PricingPolicy)
  async pricingpolicycu(@Root() r: Policy, @Ctx() ctx: BaseContext): Promise<PricingPolicy[] | null> {
    return ctx.dataLoader.loaders.Policy.pricingpolicycu.load(r);
  }

  @FieldResolver(() => PricingPolicy)
  async pricingpolicynu(@Root() r: Policy, @Ctx() ctx: BaseContext): Promise<PricingPolicy[] | null> {
    return ctx.dataLoader.loaders.Policy.pricingpolicynu.load(r);
  }

  @FieldResolver(() => PricingPolicy)
  async pricingpolicyipu(@Root() r: Policy, @Ctx() ctx: BaseContext): Promise<PricingPolicy[] | null> {
    return ctx.dataLoader.loaders.Policy.pricingpolicyipu.load(r);
  }
}
