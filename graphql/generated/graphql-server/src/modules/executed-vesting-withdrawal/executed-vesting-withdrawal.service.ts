import { Service } from 'typedi';
import { Repository } from 'typeorm';
import { InjectRepository } from 'typeorm-typedi-extensions';
import { BaseService, WhereInput } from 'warthog';

import { ExecutedVestingWithdrawal } from './executed-vesting-withdrawal.model';

@Service('ExecutedVestingWithdrawalService')
export class ExecutedVestingWithdrawalService extends BaseService<ExecutedVestingWithdrawal> {
  constructor(
    @InjectRepository(ExecutedVestingWithdrawal) protected readonly repository: Repository<ExecutedVestingWithdrawal>
  ) {
    super(ExecutedVestingWithdrawal, repository);
  }

  async find<W extends WhereInput>(
    where?: any,
    orderBy?: string,
    limit?: number,
    offset?: number,
    fields?: string[]
  ): Promise<ExecutedVestingWithdrawal[]> {
    let f = fields;
    if (f == undefined) {
      f = [];
    }

    return super.find<W>(where, orderBy, limit, offset, f);
  }
}
