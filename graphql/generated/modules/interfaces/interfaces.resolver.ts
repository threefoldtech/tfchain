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
  InterfacesCreateInput,
  InterfacesCreateManyArgs,
  InterfacesUpdateArgs,
  InterfacesWhereArgs,
  InterfacesWhereInput,
  InterfacesWhereUniqueInput,
  InterfacesOrderByEnum,
} from '../../warthog';

import { Interfaces } from './interfaces.model';
import { InterfacesService } from './interfaces.service';

import { Node } from '../node/node.model';
import { NodeService } from '../node/node.service';
import { getConnection, getRepository, In, Not } from 'typeorm';
import _ from 'lodash';

@ObjectType()
export class InterfacesEdge {
  @Field(() => Interfaces, { nullable: false })
  node!: Interfaces;

  @Field(() => String, { nullable: false })
  cursor!: string;
}

@ObjectType()
export class InterfacesConnection {
  @Field(() => Int, { nullable: false })
  totalCount!: number;

  @Field(() => [InterfacesEdge], { nullable: false })
  edges!: InterfacesEdge[];

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
export class InterfacesConnectionWhereArgs extends ConnectionPageInputOptions {
  @Field(() => InterfacesWhereInput, { nullable: true })
  where?: InterfacesWhereInput;

  @Field(() => InterfacesOrderByEnum, { nullable: true })
  orderBy?: [InterfacesOrderByEnum];
}

@Resolver(Interfaces)
export class InterfacesResolver {
  constructor(@Inject('InterfacesService') public readonly service: InterfacesService) {}

  @Query(() => [Interfaces])
  async interfaces(
    @Args() { where, orderBy, limit, offset }: InterfacesWhereArgs,
    @Fields() fields: string[]
  ): Promise<Interfaces[]> {
    return this.service.find<InterfacesWhereInput>(where, orderBy, limit, offset, fields);
  }

  @Query(() => Interfaces, { nullable: true })
  async interfacesByUniqueInput(
    @Arg('where') where: InterfacesWhereUniqueInput,
    @Fields() fields: string[]
  ): Promise<Interfaces | null> {
    const result = await this.service.find(where, undefined, 1, 0, fields);
    return result && result.length >= 1 ? result[0] : null;
  }

  @Query(() => InterfacesConnection)
  async interfacesConnection(
    @Args() { where, orderBy, ...pageOptions }: InterfacesConnectionWhereArgs,
    @Info() info: any
  ): Promise<InterfacesConnection> {
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
      result = await this.service.findConnection<InterfacesWhereInput>(where, orderBy, pageOptions, rawFields);
    } catch (err: any) {
      console.log(err);
      // TODO: should continue to return this on `Error: Items is empty` or throw the error
      if (!(err.message as string).includes('Items is empty')) throw err;
    }

    return result as Promise<InterfacesConnection>;
  }

  @FieldResolver(() => Node)
  async node(@Root() r: Interfaces, @Ctx() ctx: BaseContext): Promise<Node | null> {
    return ctx.dataLoader.loaders.Interfaces.node.load(r);
  }
}
