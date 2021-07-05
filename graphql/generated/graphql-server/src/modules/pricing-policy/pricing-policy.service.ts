import { Service, Inject } from 'typedi';
import { Repository } from 'typeorm';
import { InjectRepository } from 'typeorm-typedi-extensions';
import { WhereInput } from 'warthog';
import { WarthogBaseService } from '../../WarthogBaseService';

import { PricingPolicy } from './pricing-policy.model';

import {} from '../variants/variants.model';

import { PricingPolicyWhereArgs, PricingPolicyWhereInput } from '../../../generated';

@Service('PricingPolicyService')
export class PricingPolicyService extends WarthogBaseService<PricingPolicy> {
  constructor(@InjectRepository(PricingPolicy) protected readonly repository: Repository<PricingPolicy>) {
    super(PricingPolicy, repository);
  }

  async find<W extends WhereInput>(
    where?: any,
    orderBy?: string | string[],
    limit?: number,
    offset?: number,
    fields?: string[]
  ): Promise<PricingPolicy[]> {
    let f = fields || [];

    return this.findWithRelations<W>(where, orderBy, limit, offset, f);
  }

  async findWithRelations<W extends WhereInput>(
    _where?: any,
    orderBy?: string | string[],
    limit?: number,
    offset?: number,
    fields?: string[]
  ): Promise<PricingPolicy[]> {
    const where = <PricingPolicyWhereInput>(_where || {});

    let mainQuery = this.buildFindQueryWithParams(<any>where, orderBy, undefined, fields, 'main').take(undefined); // remove LIMIT

    let parameters = mainQuery.getParameters();

    mainQuery = mainQuery.setParameters(parameters);

    return mainQuery
      .take(limit || 50)
      .skip(offset || 0)
      .getMany();
  }
}
