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
  PublicIpCreateInput,
  PublicIpCreateManyArgs,
  PublicIpUpdateArgs,
  PublicIpWhereArgs,
  PublicIpWhereInput,
  PublicIpWhereUniqueInput,
  PublicIpOrderByEnum
} from '../../../generated';

import { PublicIp } from './public-ip.model';
import { PublicIpService } from './public-ip.service';

@ObjectType()
export class PublicIpEdge {
  @Field(() => PublicIp, { nullable: false })
  node!: PublicIp;

  @Field(() => String, { nullable: false })
  cursor!: string;
}

@ObjectType()
export class PublicIpConnection {
  @Field(() => Int, { nullable: false })
  totalCount!: number;

  @Field(() => [PublicIpEdge], { nullable: false })
  edges!: PublicIpEdge[];

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
export class PublicIpConnectionWhereArgs extends ConnectionPageInputOptions {
  @Field(() => PublicIpWhereInput, { nullable: true })
  where?: PublicIpWhereInput;

  @Field(() => PublicIpOrderByEnum, { nullable: true })
  orderBy?: [PublicIpOrderByEnum];
}

@Resolver(PublicIp)
export class PublicIpResolver {
  constructor(@Inject('PublicIpService') public readonly service: PublicIpService) {}

  @Query(() => [PublicIp])
  async publicIps(
    @Args() { where, orderBy, limit, offset }: PublicIpWhereArgs,
    @Fields() fields: string[]
  ): Promise<PublicIp[]> {
    return this.service.find<PublicIpWhereInput>(where, orderBy, limit, offset, fields);
  }

  @Query(() => PublicIp, { nullable: true })
  async publicIpByUniqueInput(
    @Arg('where') where: PublicIpWhereUniqueInput,
    @Fields() fields: string[]
  ): Promise<PublicIp | null> {
    const result = await this.service.find(where, undefined, 1, 0, fields);
    return result && result.length >= 1 ? result[0] : null;
  }

  @Query(() => PublicIpConnection)
  async publicIpsConnection(
    @Args() { where, orderBy, ...pageOptions }: PublicIpConnectionWhereArgs,
    @Info() info: any
  ): Promise<PublicIpConnection> {
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
      result = await this.service.findConnection<PublicIpWhereInput>(where, orderBy, pageOptions, rawFields);
    } catch (err) {
      console.log(err);
      // TODO: should continue to return this on `Error: Items is empty` or throw the error
      if (!(err.message as string).includes('Items is empty')) throw err;
    }

    return result as Promise<PublicIpConnection>;
  }
}
