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
  CertificationCodesCreateInput,
  CertificationCodesCreateManyArgs,
  CertificationCodesUpdateArgs,
  CertificationCodesWhereArgs,
  CertificationCodesWhereInput,
  CertificationCodesWhereUniqueInput,
  CertificationCodesOrderByEnum
} from '../../../generated';

import { CertificationCodes } from './certification-codes.model';
import { CertificationCodesService } from './certification-codes.service';

@ObjectType()
export class CertificationCodesEdge {
  @Field(() => CertificationCodes, { nullable: false })
  node!: CertificationCodes;

  @Field(() => String, { nullable: false })
  cursor!: string;
}

@ObjectType()
export class CertificationCodesConnection {
  @Field(() => Int, { nullable: false })
  totalCount!: number;

  @Field(() => [CertificationCodesEdge], { nullable: false })
  edges!: CertificationCodesEdge[];

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
export class CertificationCodesConnectionWhereArgs extends ConnectionPageInputOptions {
  @Field(() => CertificationCodesWhereInput, { nullable: true })
  where?: CertificationCodesWhereInput;

  @Field(() => CertificationCodesOrderByEnum, { nullable: true })
  orderBy?: CertificationCodesOrderByEnum;
}

@Resolver(CertificationCodes)
export class CertificationCodesResolver {
  constructor(@Inject('CertificationCodesService') public readonly service: CertificationCodesService) {}

  @Query(() => [CertificationCodes])
  async certificationCodes(
    @Args() { where, orderBy, limit, offset }: CertificationCodesWhereArgs,
    @Fields() fields: string[]
  ): Promise<CertificationCodes[]> {
    return this.service.find<CertificationCodesWhereInput>(where, orderBy, limit, offset, fields);
  }

  @Query(() => CertificationCodes, { nullable: true })
  async certificationCodes(
    @Arg('where') where: CertificationCodesWhereUniqueInput,
    @Fields() fields: string[]
  ): Promise<CertificationCodes | null> {
    const result = await this.service.find(where, undefined, 1, 0, fields);
    return result && result.length >= 1 ? result[0] : null;
  }

  @Query(() => CertificationCodesConnection)
  async certificationCodesConnection(
    @Args() { where, orderBy, ...pageOptions }: CertificationCodesConnectionWhereArgs,
    @Info() info: any
  ): Promise<CertificationCodesConnection> {
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
      result = await this.service.findConnection<CertificationCodesWhereInput>(where, orderBy, pageOptions, rawFields);
    } catch (err) {
      console.log(err);
      // TODO: should continue to return this on `Error: Items is empty` or throw the error
      if (!(err.message as string).includes('Items is empty')) throw err;
    }

    return result as Promise<CertificationCodesConnection>;
  }
}
