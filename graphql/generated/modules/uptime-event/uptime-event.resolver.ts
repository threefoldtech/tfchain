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
  UptimeEventCreateInput,
  UptimeEventCreateManyArgs,
  UptimeEventUpdateArgs,
  UptimeEventWhereArgs,
  UptimeEventWhereInput,
  UptimeEventWhereUniqueInput,
  UptimeEventOrderByEnum
} from '../../warthog';

import { UptimeEvent } from './uptime-event.model';
import { UptimeEventService } from './uptime-event.service';

@ObjectType()
export class UptimeEventEdge {
  @Field(() => UptimeEvent, { nullable: false })
  node!: UptimeEvent;

  @Field(() => String, { nullable: false })
  cursor!: string;
}

@ObjectType()
export class UptimeEventConnection {
  @Field(() => Int, { nullable: false })
  totalCount!: number;

  @Field(() => [UptimeEventEdge], { nullable: false })
  edges!: UptimeEventEdge[];

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
export class UptimeEventConnectionWhereArgs extends ConnectionPageInputOptions {
  @Field(() => UptimeEventWhereInput, { nullable: true })
  where?: UptimeEventWhereInput;

  @Field(() => UptimeEventOrderByEnum, { nullable: true })
  orderBy?: [UptimeEventOrderByEnum];
}

@Resolver(UptimeEvent)
export class UptimeEventResolver {
  constructor(@Inject('UptimeEventService') public readonly service: UptimeEventService) {}

  @Query(() => [UptimeEvent])
  async uptimeEvents(
    @Args() { where, orderBy, limit, offset }: UptimeEventWhereArgs,
    @Fields() fields: string[]
  ): Promise<UptimeEvent[]> {
    return this.service.find<UptimeEventWhereInput>(where, orderBy, limit, offset, fields);
  }

  @Query(() => UptimeEvent, { nullable: true })
  async uptimeEventByUniqueInput(
    @Arg('where') where: UptimeEventWhereUniqueInput,
    @Fields() fields: string[]
  ): Promise<UptimeEvent | null> {
    const result = await this.service.find(where, undefined, 1, 0, fields);
    return result && result.length >= 1 ? result[0] : null;
  }

  @Query(() => UptimeEventConnection)
  async uptimeEventsConnection(
    @Args() { where, orderBy, ...pageOptions }: UptimeEventConnectionWhereArgs,
    @Info() info: any
  ): Promise<UptimeEventConnection> {
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
      result = await this.service.findConnection<UptimeEventWhereInput>(where, orderBy, pageOptions, rawFields);
    } catch (err) {
      console.log(err);
      // TODO: should continue to return this on `Error: Items is empty` or throw the error
      if (!(err.message as string).includes('Items is empty')) throw err;
    }

    return result as Promise<UptimeEventConnection>;
  }
}
