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
  ConsumptionCreateInput,
  ConsumptionCreateManyArgs,
  ConsumptionUpdateArgs,
  ConsumptionWhereArgs,
  ConsumptionWhereInput,
  ConsumptionWhereUniqueInput,
  ConsumptionOrderByEnum,
} from '../../warthog';

import { Consumption } from './consumption.model';
import { ConsumptionService } from './consumption.service';

@ObjectType()
export class ConsumptionEdge {
  @Field(() => Consumption, { nullable: false })
  node!: Consumption;

  @Field(() => String, { nullable: false })
  cursor!: string;
}

@ObjectType()
export class ConsumptionConnection {
  @Field(() => Int, { nullable: false })
  totalCount!: number;

  @Field(() => [ConsumptionEdge], { nullable: false })
  edges!: ConsumptionEdge[];

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
export class ConsumptionConnectionWhereArgs extends ConnectionPageInputOptions {
  @Field(() => ConsumptionWhereInput, { nullable: true })
  where?: ConsumptionWhereInput;

  @Field(() => ConsumptionOrderByEnum, { nullable: true })
  orderBy?: [ConsumptionOrderByEnum];
}

@Resolver(Consumption)
export class ConsumptionResolver {
  constructor(@Inject('ConsumptionService') public readonly service: ConsumptionService) {}

  @Query(() => [Consumption])
  async consumptions(
    @Args() { where, orderBy, limit, offset }: ConsumptionWhereArgs,
    @Fields() fields: string[]
  ): Promise<Consumption[]> {
    return this.service.find<ConsumptionWhereInput>(where, orderBy, limit, offset, fields);
  }

  @Query(() => Consumption, { nullable: true })
  async consumptionByUniqueInput(
    @Arg('where') where: ConsumptionWhereUniqueInput,
    @Fields() fields: string[]
  ): Promise<Consumption | null> {
    const result = await this.service.find(where, undefined, 1, 0, fields);
    return result && result.length >= 1 ? result[0] : null;
  }

  @Query(() => ConsumptionConnection)
  async consumptionsConnection(
    @Args() { where, orderBy, ...pageOptions }: ConsumptionConnectionWhereArgs,
    @Info() info: any
  ): Promise<ConsumptionConnection> {
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
      result = await this.service.findConnection<ConsumptionWhereInput>(where, orderBy, pageOptions, rawFields);
    } catch (err: any) {
      console.log(err);
      // TODO: should continue to return this on `Error: Items is empty` or throw the error
      if (!(err.message as string).includes('Items is empty')) throw err;
    }

    return result as Promise<ConsumptionConnection>;
  }
}
