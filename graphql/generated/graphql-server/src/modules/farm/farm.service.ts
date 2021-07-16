import { Service, Inject } from 'typedi';
import { Repository } from 'typeorm';
import { InjectRepository } from 'typeorm-typedi-extensions';
import { WhereInput } from 'warthog';
import { WarthogBaseService } from '../../WarthogBaseService';

import { Farm } from './farm.model';

import {} from '../variants/variants.model';

import { FarmWhereArgs, FarmWhereInput } from '../../../generated';

import { PublicIp } from '../public-ip/public-ip.model';
import { PublicIpService } from '../public-ip/public-ip.service';
import { getConnection, getRepository, In, Not } from 'typeorm';
import _ from 'lodash';

@Service('FarmService')
export class FarmService extends WarthogBaseService<Farm> {
  @Inject('PublicIpService')
  public readonly publicIPsService!: PublicIpService;

  constructor(@InjectRepository(Farm) protected readonly repository: Repository<Farm>) {
    super(Farm, repository);
  }

  async find<W extends WhereInput>(
    where?: any,
    orderBy?: string | string[],
    limit?: number,
    offset?: number,
    fields?: string[]
  ): Promise<Farm[]> {
    let f = fields || [];

    return this.findWithRelations<W>(where, orderBy, limit, offset, f);
  }

  async findWithRelations<W extends WhereInput>(
    _where?: any,
    orderBy?: string | string[],
    limit?: number,
    offset?: number,
    fields?: string[]
  ): Promise<Farm[]> {
    const where = <FarmWhereInput>(_where || {});

    // remove relation filters to enable warthog query builders

    const { publicIPs_some, publicIPs_none, publicIPs_every } = where;

    if (+!!publicIPs_some + +!!publicIPs_none + +!!publicIPs_every > 1) {
      throw new Error(`A query can have at most one of none, some, every clauses on a relation field`);
    }

    delete where.publicIPs_some;
    delete where.publicIPs_none;
    delete where.publicIPs_every;

    let mainQuery = this.buildFindQueryWithParams(<any>where, orderBy, undefined, fields, 'main').take(undefined); // remove LIMIT

    let parameters = mainQuery.getParameters();

    const publicIPsFilter = publicIPs_some || publicIPs_none || publicIPs_every;

    if (publicIPsFilter) {
      const publicIPsQuery = this.publicIPsService
        .buildFindQueryWithParams(<any>publicIPsFilter, undefined, undefined, ['id'], 'publicIPs')
        .take(undefined); //remove the default LIMIT

      parameters = { ...parameters, ...publicIPsQuery.getParameters() };

      const subQueryFiltered = this.getQueryBuilder()
        .select([])
        .leftJoin('farm.publicIPs', 'publicIPs_filtered', `publicIPs_filtered.id IN (${publicIPsQuery.getQuery()})`)
        .groupBy('farm_id')
        .addSelect('count(publicIPs_filtered.id)', 'cnt_filtered')
        .addSelect('farm.id', 'farm_id');

      const subQueryTotal = this.getQueryBuilder()
        .select([])
        .leftJoin('farm.publicIPs', 'publicIPs_total')
        .groupBy('farm_id')
        .addSelect('count(publicIPs_total.id)', 'cnt_total')
        .addSelect('farm.id', 'farm_id');

      const subQuery = `
                SELECT 
                    f.farm_id farm_id, f.cnt_filtered cnt_filtered, t.cnt_total cnt_total 
                FROM 
                    (${subQueryTotal.getQuery()}) t, (${subQueryFiltered.getQuery()}) f 
                WHERE 
                    t.farm_id = f.farm_id`;

      if (publicIPs_none) {
        mainQuery = mainQuery.andWhere(`farm.id IN 
                (SELECT 
                    publicIPs_subq.farm_id
                FROM 
                    (${subQuery}) publicIPs_subq 
                WHERE 
                    publicIPs_subq.cnt_filtered = 0
                )`);
      }

      if (publicIPs_some) {
        mainQuery = mainQuery.andWhere(`farm.id IN 
                (SELECT 
                    publicIPs_subq.farm_id
                FROM 
                    (${subQuery}) publicIPs_subq 
                WHERE 
                    publicIPs_subq.cnt_filtered > 0
                )`);
      }

      if (publicIPs_every) {
        mainQuery = mainQuery.andWhere(`farm.id IN 
                (SELECT 
                    publicIPs_subq.farm_id
                FROM 
                    (${subQuery}) publicIPs_subq 
                WHERE 
                    publicIPs_subq.cnt_filtered > 0 
                    AND publicIPs_subq.cnt_filtered = publicIPs_subq.cnt_total
                )`);
      }
    }

    mainQuery = mainQuery.setParameters(parameters);

    return mainQuery
      .take(limit || 50)
      .skip(offset || 0)
      .getMany();
  }
}
