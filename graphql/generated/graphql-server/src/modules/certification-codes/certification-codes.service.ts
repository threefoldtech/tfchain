import { Service } from 'typedi';
import { Repository } from 'typeorm';
import { InjectRepository } from 'typeorm-typedi-extensions';
import { BaseService, WhereInput } from 'warthog';

import { CertificationCodes } from './certification-codes.model';

@Service('CertificationCodesService')
export class CertificationCodesService extends BaseService<CertificationCodes> {
  constructor(@InjectRepository(CertificationCodes) protected readonly repository: Repository<CertificationCodes>) {
    super(CertificationCodes, repository);
  }

  async find<W extends WhereInput>(
    where?: any,
    orderBy?: string,
    limit?: number,
    offset?: number,
    fields?: string[]
  ): Promise<CertificationCodes[]> {
    let f = fields;
    if (f == undefined) {
      f = [];
    }

    return super.find<W>(where, orderBy, limit, offset, f);
  }
}
