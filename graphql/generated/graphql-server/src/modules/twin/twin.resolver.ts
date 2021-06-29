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
  TwinCreateInput,
  TwinCreateManyArgs,
  TwinUpdateArgs,
  TwinWhereArgs,
  TwinWhereInput,
  TwinWhereUniqueInput,
  TwinOrderByEnum
} from '../../../generated';

import { Twin } from './twin.model';
import { TwinService } from './twin.service';

import { EntityProof } from '../entity-proof/entity-proof.model';
import { EntityProofService } from '../entity-proof/entity-proof.service';
import { getConnection, getRepository, In, Not } from 'typeorm';
import _ from 'lodash';

@ObjectType()
export class TwinEdge {
  @Field(() => Twin, { nullable: false })
  node!: Twin;

  @Field(() => String, { nullable: false })
  cursor!: string;
}

@ObjectType()
export class TwinConnection {
  @Field(() => Int, { nullable: false })
  totalCount!: number;

  @Field(() => [TwinEdge], { nullable: false })
  edges!: TwinEdge[];

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
export class TwinConnectionWhereArgs extends ConnectionPageInputOptions {
  @Field(() => TwinWhereInput, { nullable: true })
  where?: TwinWhereInput;

  @Field(() => TwinOrderByEnum, { nullable: true })
  orderBy?: [TwinOrderByEnum];
}

@Resolver(Twin)
export class TwinResolver {
  constructor(@Inject('TwinService') public readonly service: TwinService) {}

  @Query(() => [Twin])
  async twins(@Args() { where, orderBy, limit, offset }: TwinWhereArgs, @Fields() fields: string[]): Promise<Twin[]> {
    return this.service.find<TwinWhereInput>(where, orderBy, limit, offset, fields);
  }

  @Query(() => Twin, { nullable: true })
  async twinByUniqueInput(@Arg('where') where: TwinWhereUniqueInput, @Fields() fields: string[]): Promise<Twin | null> {
    const result = await this.service.find(where, undefined, 1, 0, fields);
    return result && result.length >= 1 ? result[0] : null;
  }

  @Query(() => TwinConnection)
  async twinsConnection(
    @Args() { where, orderBy, ...pageOptions }: TwinConnectionWhereArgs,
    @Info() info: any
  ): Promise<TwinConnection> {
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
      result = await this.service.findConnection<TwinWhereInput>(where, orderBy, pageOptions, rawFields);
    } catch (err) {
      console.log(err);
      // TODO: should continue to return this on `Error: Items is empty` or throw the error
      if (!(err.message as string).includes('Items is empty')) throw err;
    }

    return result as Promise<TwinConnection>;
  }

  @FieldResolver(() => EntityProof)
  async entityprooftwinRel(@Root() r: Twin, @Ctx() ctx: BaseContext): Promise<EntityProof[] | null> {
    return ctx.dataLoader.loaders.Twin.entityprooftwinRel.load(r);
  }
}
