import { Service } from 'typedi';
import { Repository } from 'typeorm';
import { InjectRepository } from 'typeorm-typedi-extensions';
import { BaseService, WhereInput } from 'warthog';

import { ExpiredVestingWithdrawal } from './expired-vesting-withdrawal.model';

@Service('ExpiredVestingWithdrawalService')
export class ExpiredVestingWithdrawalService extends BaseService<ExpiredVestingWithdrawal> {
  constructor(
    @InjectRepository(ExpiredVestingWithdrawal) protected readonly repository: Repository<ExpiredVestingWithdrawal>
  ) {
    super(ExpiredVestingWithdrawal, repository);
  }

  async find<W extends WhereInput>(
    where?: any,
    orderBy?: string,
    limit?: number,
    offset?: number,
    fields?: string[]
  ): Promise<ExpiredVestingWithdrawal[]> {
    let f = fields;
    if (f == undefined) {
      f = [];
    }

    return super.find<W>(where, orderBy, limit, offset, f);
  }
}
