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
  EntityProofCreateInput,
  EntityProofCreateManyArgs,
  EntityProofUpdateArgs,
  EntityProofWhereArgs,
  EntityProofWhereInput,
  EntityProofWhereUniqueInput,
  EntityProofOrderByEnum
} from '../../../generated';

import { EntityProof } from './entity-proof.model';
import { EntityProofService } from './entity-proof.service';

import { Twin } from '../twin/twin.model';
import { getConnection } from 'typeorm';

@ObjectType()
export class EntityProofEdge {
  @Field(() => EntityProof, { nullable: false })
  node!: EntityProof;

  @Field(() => String, { nullable: false })
  cursor!: string;
}

@ObjectType()
export class EntityProofConnection {
  @Field(() => Int, { nullable: false })
  totalCount!: number;

  @Field(() => [EntityProofEdge], { nullable: false })
  edges!: EntityProofEdge[];

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
export class EntityProofConnectionWhereArgs extends ConnectionPageInputOptions {
  @Field(() => EntityProofWhereInput, { nullable: true })
  where?: EntityProofWhereInput;

  @Field(() => EntityProofOrderByEnum, { nullable: true })
  orderBy?: EntityProofOrderByEnum;
}

@Resolver(EntityProof)
export class EntityProofResolver {
  constructor(@Inject('EntityProofService') public readonly service: EntityProofService) {}

  @Query(() => [EntityProof])
  async entityProofs(
    @Args() { where, orderBy, limit, offset }: EntityProofWhereArgs,
    @Fields() fields: string[]
  ): Promise<EntityProof[]> {
    return this.service.find<EntityProofWhereInput>(where, orderBy, limit, offset, fields);
  }

  @Query(() => EntityProof, { nullable: true })
  async entityProof(
    @Arg('where') where: EntityProofWhereUniqueInput,
    @Fields() fields: string[]
  ): Promise<EntityProof | null> {
    const result = await this.service.find(where, undefined, 1, 0, fields);
    return result && result.length >= 1 ? result[0] : null;
  }

  @Query(() => EntityProofConnection)
  async entityProofsConnection(
    @Args() { where, orderBy, ...pageOptions }: EntityProofConnectionWhereArgs,
    @Info() info: any
  ): Promise<EntityProofConnection> {
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
      result = await this.service.findConnection<EntityProofWhereInput>(where, orderBy, pageOptions, rawFields);
    } catch (err) {
      console.log(err);
      // TODO: should continue to return this on `Error: Items is empty` or throw the error
      if (!(err.message as string).includes('Items is empty')) throw err;
    }

    return result as Promise<EntityProofConnection>;
  }

  @FieldResolver(() => Twin)
  async twin(@Root() r: EntityProof): Promise<Twin | null> {
    const result = await getConnection()
      .getRepository(EntityProof)
      .findOne(r.id, { relations: ['twin'] });
    if (result && result.twin !== undefined) {
      return result.twin;
    }
    return null;
  }
}
