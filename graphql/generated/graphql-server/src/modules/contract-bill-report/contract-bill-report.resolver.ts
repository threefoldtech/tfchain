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
  ContractBillReportCreateInput,
  ContractBillReportCreateManyArgs,
  ContractBillReportUpdateArgs,
  ContractBillReportWhereArgs,
  ContractBillReportWhereInput,
  ContractBillReportWhereUniqueInput,
  ContractBillReportOrderByEnum
} from '../../../generated';

import { ContractBillReport } from './contract-bill-report.model';
import { ContractBillReportService } from './contract-bill-report.service';

@ObjectType()
export class ContractBillReportEdge {
  @Field(() => ContractBillReport, { nullable: false })
  node!: ContractBillReport;

  @Field(() => String, { nullable: false })
  cursor!: string;
}

@ObjectType()
export class ContractBillReportConnection {
  @Field(() => Int, { nullable: false })
  totalCount!: number;

  @Field(() => [ContractBillReportEdge], { nullable: false })
  edges!: ContractBillReportEdge[];

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
export class ContractBillReportConnectionWhereArgs extends ConnectionPageInputOptions {
  @Field(() => ContractBillReportWhereInput, { nullable: true })
  where?: ContractBillReportWhereInput;

  @Field(() => ContractBillReportOrderByEnum, { nullable: true })
  orderBy?: [ContractBillReportOrderByEnum];
}

@Resolver(ContractBillReport)
export class ContractBillReportResolver {
  constructor(@Inject('ContractBillReportService') public readonly service: ContractBillReportService) {}

  @Query(() => [ContractBillReport])
  async contractBillReports(
    @Args() { where, orderBy, limit, offset }: ContractBillReportWhereArgs,
    @Fields() fields: string[]
  ): Promise<ContractBillReport[]> {
    return this.service.find<ContractBillReportWhereInput>(where, orderBy, limit, offset, fields);
  }

  @Query(() => ContractBillReport, { nullable: true })
  async contractBillReportByUniqueInput(
    @Arg('where') where: ContractBillReportWhereUniqueInput,
    @Fields() fields: string[]
  ): Promise<ContractBillReport | null> {
    const result = await this.service.find(where, undefined, 1, 0, fields);
    return result && result.length >= 1 ? result[0] : null;
  }

  @Query(() => ContractBillReportConnection)
  async contractBillReportsConnection(
    @Args() { where, orderBy, ...pageOptions }: ContractBillReportConnectionWhereArgs,
    @Info() info: any
  ): Promise<ContractBillReportConnection> {
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
      result = await this.service.findConnection<ContractBillReportWhereInput>(where, orderBy, pageOptions, rawFields);
    } catch (err) {
      console.log(err);
      // TODO: should continue to return this on `Error: Items is empty` or throw the error
      if (!(err.message as string).includes('Items is empty')) throw err;
    }

    return result as Promise<ContractBillReportConnection>;
  }
}
