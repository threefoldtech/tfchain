import { Service } from 'typedi';
import { Repository } from 'typeorm';
import { InjectRepository } from 'typeorm-typedi-extensions';
import { BaseService, WhereInput } from 'warthog';

import { FailedVestingWithdrawal } from './failed-vesting-withdrawal.model';

@Service('FailedVestingWithdrawalService')
export class FailedVestingWithdrawalService extends BaseService<FailedVestingWithdrawal> {
  constructor(
    @InjectRepository(FailedVestingWithdrawal) protected readonly repository: Repository<FailedVestingWithdrawal>
  ) {
    super(FailedVestingWithdrawal, repository);
  }

  async find<W extends WhereInput>(
    where?: any,
    orderBy?: string,
    limit?: number,
    offset?: number,
    fields?: string[]
  ): Promise<FailedVestingWithdrawal[]> {
    let f = fields;
    if (f == undefined) {
      f = [];
    }

    return super.find<W>(where, orderBy, limit, offset, f);
  }
}
