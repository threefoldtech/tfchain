import { Service, Inject } from 'typedi';
import { Repository, SelectQueryBuilder } from 'typeorm';
import { InjectRepository } from 'typeorm-typedi-extensions';
import { WhereInput, HydraBaseService } from '@subsquid/warthog';

import { PublicConfig } from './public-config.model';

import { PublicConfigWhereArgs, PublicConfigWhereInput } from '../../warthog';

import { Node } from '../node/node.model';
import { NodeService } from '../node/node.service';
import { getConnection, getRepository, In, Not } from 'typeorm';
import _ from 'lodash';

@Service('PublicConfigService')
export class PublicConfigService extends HydraBaseService<PublicConfig> {
  @Inject('NodeService')
  public readonly nodepublicConfigService!: NodeService;

  constructor(@InjectRepository(PublicConfig) protected readonly repository: Repository<PublicConfig>) {
    super(PublicConfig, repository);
  }

  async find<W extends WhereInput>(
    where?: any,
    orderBy?: string | string[],
    limit?: number,
    offset?: number,
    fields?: string[]
  ): Promise<PublicConfig[]> {
    return this.findWithRelations<W>(where, orderBy, limit, offset, fields);
  }

  findWithRelations<W extends WhereInput>(
    _where?: any,
    orderBy?: string | string[],
    limit?: number,
    offset?: number,
    fields?: string[]
  ): Promise<PublicConfig[]> {
    return this.buildFindWithRelationsQuery(_where, orderBy, limit, offset, fields).getMany();
  }

  buildFindWithRelationsQuery<W extends WhereInput>(
    _where?: any,
    orderBy?: string | string[],
    limit?: number,
    offset?: number,
    fields?: string[]
  ): SelectQueryBuilder<PublicConfig> {
    const where = <PublicConfigWhereInput>(_where || {});

    // remove relation filters to enable warthog query builders

    const { nodepublicConfig_some, nodepublicConfig_none, nodepublicConfig_every } = where;

    if (+!!nodepublicConfig_some + +!!nodepublicConfig_none + +!!nodepublicConfig_every > 1) {
      throw new Error(`A query can have at most one of none, some, every clauses on a relation field`);
    }

    delete where.nodepublicConfig_some;
    delete where.nodepublicConfig_none;
    delete where.nodepublicConfig_every;

    let mainQuery = this.buildFindQueryWithParams(<any>where, orderBy, undefined, fields, 'main').take(undefined); // remove LIMIT

    let parameters = mainQuery.getParameters();

    const nodepublicConfigFilter = nodepublicConfig_some || nodepublicConfig_none || nodepublicConfig_every;

    if (nodepublicConfigFilter) {
      const nodepublicConfigQuery = this.nodepublicConfigService
        .buildFindQueryWithParams(<any>nodepublicConfigFilter, undefined, undefined, ['id'], 'nodepublicConfig')
        .take(undefined); //remove the default LIMIT

      parameters = { ...parameters, ...nodepublicConfigQuery.getParameters() };

      const subQueryFiltered = this.getQueryBuilder()
        .select([])
        .leftJoin(
          'publicconfig.nodepublicConfig',
          'nodepublicConfig_filtered',
          `nodepublicConfig_filtered.id IN (${nodepublicConfigQuery.getQuery()})`
        )
        .groupBy('publicconfig_id')
        .addSelect('count(nodepublicConfig_filtered.id)', 'cnt_filtered')
        .addSelect('publicconfig.id', 'publicconfig_id');

      const subQueryTotal = this.getQueryBuilder()
        .select([])
        .leftJoin('publicconfig.nodepublicConfig', 'nodepublicConfig_total')
        .groupBy('publicconfig_id')
        .addSelect('count(nodepublicConfig_total.id)', 'cnt_total')
        .addSelect('publicconfig.id', 'publicconfig_id');

      const subQuery = `
                SELECT
                    f.publicconfig_id publicconfig_id, f.cnt_filtered cnt_filtered, t.cnt_total cnt_total
                FROM
                    (${subQueryTotal.getQuery()}) t, (${subQueryFiltered.getQuery()}) f
                WHERE
                    t.publicconfig_id = f.publicconfig_id`;

      if (nodepublicConfig_none) {
        mainQuery = mainQuery.andWhere(`publicconfig.id IN
                (SELECT
                    nodepublicConfig_subq.publicconfig_id
                FROM
                    (${subQuery}) nodepublicConfig_subq
                WHERE
                    nodepublicConfig_subq.cnt_filtered = 0
                )`);
      }

      if (nodepublicConfig_some) {
        mainQuery = mainQuery.andWhere(`publicconfig.id IN
                (SELECT
                    nodepublicConfig_subq.publicconfig_id
                FROM
                    (${subQuery}) nodepublicConfig_subq
                WHERE
                    nodepublicConfig_subq.cnt_filtered > 0
                )`);
      }

      if (nodepublicConfig_every) {
        mainQuery = mainQuery.andWhere(`publicconfig.id IN
                (SELECT
                    nodepublicConfig_subq.publicconfig_id
                FROM
                    (${subQuery}) nodepublicConfig_subq
                WHERE
                    nodepublicConfig_subq.cnt_filtered > 0
                    AND nodepublicConfig_subq.cnt_filtered = nodepublicConfig_subq.cnt_total
                )`);
      }
    }

    mainQuery = mainQuery.setParameters(parameters);

    return mainQuery.take(limit || 50).skip(offset || 0);
  }
}
