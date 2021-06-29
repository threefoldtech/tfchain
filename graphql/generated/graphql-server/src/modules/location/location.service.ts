import { Service, Inject } from 'typedi';
import { Repository } from 'typeorm';
import { InjectRepository } from 'typeorm-typedi-extensions';
import { WhereInput } from 'warthog';
import { WarthogBaseService } from '../../WarthogBaseService';

import { Location } from './location.model';

import {} from '../variants/variants.model';

import { LocationWhereArgs, LocationWhereInput } from '../../../generated';

import { Node } from '../node/node.model';
import { NodeService } from '../node/node.service';
import { getConnection, getRepository, In, Not } from 'typeorm';
import _ from 'lodash';

@Service('LocationService')
export class LocationService extends WarthogBaseService<Location> {
  @Inject('NodeService')
  public readonly nodelocationService!: NodeService;

  constructor(@InjectRepository(Location) protected readonly repository: Repository<Location>) {
    super(Location, repository);
  }

  async find<W extends WhereInput>(
    where?: any,
    orderBy?: string | string[],
    limit?: number,
    offset?: number,
    fields?: string[]
  ): Promise<Location[]> {
    let f = fields || [];

    return this.findWithRelations<W>(where, orderBy, limit, offset, f);
  }

  async findWithRelations<W extends WhereInput>(
    _where?: any,
    orderBy?: string | string[],
    limit?: number,
    offset?: number,
    fields?: string[]
  ): Promise<Location[]> {
    const where = <LocationWhereInput>(_where || {});

    // remove relation filters to enable warthog query builders

    const { nodelocation_some, nodelocation_none, nodelocation_every } = where;

    if (+!!nodelocation_some + +!!nodelocation_none + +!!nodelocation_every > 1) {
      throw new Error(`A query can have at most one of none, some, every clauses on a relation field`);
    }

    delete where.nodelocation_some;
    delete where.nodelocation_none;
    delete where.nodelocation_every;

    let mainQuery = this.buildFindQueryWithParams(<any>where, orderBy, undefined, fields, 'main').take(undefined); // remove LIMIT

    let parameters = mainQuery.getParameters();

    const nodelocationFilter = nodelocation_some || nodelocation_none || nodelocation_every;

    if (nodelocationFilter) {
      const nodelocationQuery = this.nodelocationService
        .buildFindQueryWithParams(<any>nodelocationFilter, undefined, undefined, ['id'], 'nodelocation')
        .take(undefined); //remove the default LIMIT

      parameters = { ...parameters, ...nodelocationQuery.getParameters() };

      const subQueryFiltered = this.getQueryBuilder()
        .select([])
        .leftJoin(
          'location.nodelocation',
          'nodelocation_filtered',
          `nodelocation_filtered.id IN (${nodelocationQuery.getQuery()})`
        )
        .groupBy('location_id')
        .addSelect('count(nodelocation_filtered.id)', 'cnt_filtered')
        .addSelect('location.id', 'location_id');

      const subQueryTotal = this.getQueryBuilder()
        .select([])
        .leftJoin('location.nodelocation', 'nodelocation_total')
        .groupBy('location_id')
        .addSelect('count(nodelocation_total.id)', 'cnt_total')
        .addSelect('location.id', 'location_id');

      const subQuery = `
                SELECT 
                    f.location_id location_id, f.cnt_filtered cnt_filtered, t.cnt_total cnt_total 
                FROM 
                    (${subQueryTotal.getQuery()}) t, (${subQueryFiltered.getQuery()}) f 
                WHERE 
                    t.location_id = f.location_id`;

      if (nodelocation_none) {
        mainQuery = mainQuery.andWhere(`location.id IN 
                (SELECT 
                    nodelocation_subq.location_id
                FROM 
                    (${subQuery}) nodelocation_subq 
                WHERE 
                    nodelocation_subq.cnt_filtered = 0
                )`);
      }

      if (nodelocation_some) {
        mainQuery = mainQuery.andWhere(`location.id IN 
                (SELECT 
                    nodelocation_subq.location_id
                FROM 
                    (${subQuery}) nodelocation_subq 
                WHERE 
                    nodelocation_subq.cnt_filtered > 0
                )`);
      }

      if (nodelocation_every) {
        mainQuery = mainQuery.andWhere(`location.id IN 
                (SELECT 
                    nodelocation_subq.location_id
                FROM 
                    (${subQuery}) nodelocation_subq 
                WHERE 
                    nodelocation_subq.cnt_filtered > 0 
                    AND nodelocation_subq.cnt_filtered = nodelocation_subq.cnt_total
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
