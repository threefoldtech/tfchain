import { Service, Inject } from 'typedi';
import { Repository } from 'typeorm';
import { InjectRepository } from 'typeorm-typedi-extensions';
import { WhereInput } from 'warthog';
import { WarthogBaseService } from '../../WarthogBaseService';

import { ContractBillReport } from './contract-bill-report.model';

import {} from '../variants/variants.model';

import { ContractBillReportWhereArgs, ContractBillReportWhereInput } from '../../../generated';

@Service('ContractBillReportService')
export class ContractBillReportService extends WarthogBaseService<ContractBillReport> {
  constructor(@InjectRepository(ContractBillReport) protected readonly repository: Repository<ContractBillReport>) {
    super(ContractBillReport, repository);
  }

  async find<W extends WhereInput>(
    where?: any,
    orderBy?: string | string[],
    limit?: number,
    offset?: number,
    fields?: string[]
  ): Promise<ContractBillReport[]> {
    let f = fields || [];

    return this.findWithRelations<W>(where, orderBy, limit, offset, f);
  }

  async findWithRelations<W extends WhereInput>(
    _where?: any,
    orderBy?: string | string[],
    limit?: number,
    offset?: number,
    fields?: string[]
  ): Promise<ContractBillReport[]> {
    const where = <ContractBillReportWhereInput>(_where || {});

    let mainQuery = this.buildFindQueryWithParams(<any>where, orderBy, undefined, fields, 'main').take(undefined); // remove LIMIT

    let parameters = mainQuery.getParameters();

    mainQuery = mainQuery.setParameters(parameters);

    return mainQuery
      .take(limit || 50)
      .skip(offset || 0)
      .getMany();
  }
}
