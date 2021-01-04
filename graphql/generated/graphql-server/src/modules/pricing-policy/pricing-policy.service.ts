import { Service } from 'typedi';
import { Repository } from 'typeorm';
import { InjectRepository } from 'typeorm-typedi-extensions';
import { BaseService, WhereInput } from 'warthog';

import { PricingPolicy } from './pricing-policy.model';

@Service('PricingPolicyService')
export class PricingPolicyService extends BaseService<PricingPolicy> {
  constructor(@InjectRepository(PricingPolicy) protected readonly repository: Repository<PricingPolicy>) {
    super(PricingPolicy, repository);
  }

  async find<W extends WhereInput>(
    where?: any,
    orderBy?: string,
    limit?: number,
    offset?: number,
    fields?: string[]
  ): Promise<PricingPolicy[]> {
    let f = fields;
    if (f == undefined) {
      f = [];
    }

    return super.find<W>(where, orderBy, limit, offset, f);
  }
}
