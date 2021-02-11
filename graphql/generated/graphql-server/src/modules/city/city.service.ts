import { Service } from 'typedi';
import { Repository } from 'typeorm';
import { InjectRepository } from 'typeorm-typedi-extensions';
import { BaseService, WhereInput } from 'warthog';

import { City } from './city.model';

@Service('CityService')
export class CityService extends BaseService<City> {
  constructor(@InjectRepository(City) protected readonly repository: Repository<City>) {
    super(City, repository);
  }

  async find<W extends WhereInput>(
    where?: any,
    orderBy?: string,
    limit?: number,
    offset?: number,
    fields?: string[]
  ): Promise<City[]> {
    let f = fields;
    if (f == undefined) {
      f = [];
    }

    return super.find<W>(where, orderBy, limit, offset, f);
  }
}
