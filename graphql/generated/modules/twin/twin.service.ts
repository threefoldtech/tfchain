import { Service, Inject } from 'typedi';
import { Repository, SelectQueryBuilder } from 'typeorm';
import { InjectRepository } from 'typeorm-typedi-extensions';
import { WhereInput, HydraBaseService } from '@subsquid/warthog';

import { Twin } from './twin.model';

import { TwinWhereArgs, TwinWhereInput } from '../../warthog';

import { EntityProof } from '../entity-proof/entity-proof.model';
import { EntityProofService } from '../entity-proof/entity-proof.service';
import { getConnection, getRepository, In, Not } from 'typeorm';
import _ from 'lodash';

@Service('TwinService')
export class TwinService extends HydraBaseService<Twin> {
  @Inject('EntityProofService')
  public readonly entityprooftwinRelService!: EntityProofService;

  constructor(@InjectRepository(Twin) protected readonly repository: Repository<Twin>) {
    super(Twin, repository);
  }

  async find<W extends WhereInput>(
    where?: any,
    orderBy?: string | string[],
    limit?: number,
    offset?: number,
    fields?: string[]
  ): Promise<Twin[]> {
    return this.findWithRelations<W>(where, orderBy, limit, offset, fields);
  }

  findWithRelations<W extends WhereInput>(
    _where?: any,
    orderBy?: string | string[],
    limit?: number,
    offset?: number,
    fields?: string[]
  ): Promise<Twin[]> {
    return this.buildFindWithRelationsQuery(_where, orderBy, limit, offset, fields).getMany();
  }

  buildFindWithRelationsQuery<W extends WhereInput>(
    _where?: any,
    orderBy?: string | string[],
    limit?: number,
    offset?: number,
    fields?: string[]
  ): SelectQueryBuilder<Twin> {
    const where = <TwinWhereInput>(_where || {});

    // remove relation filters to enable warthog query builders

    const { entityprooftwinRel_some, entityprooftwinRel_none, entityprooftwinRel_every } = where;

    if (+!!entityprooftwinRel_some + +!!entityprooftwinRel_none + +!!entityprooftwinRel_every > 1) {
      throw new Error(`A query can have at most one of none, some, every clauses on a relation field`);
    }

    delete where.entityprooftwinRel_some;
    delete where.entityprooftwinRel_none;
    delete where.entityprooftwinRel_every;

    let mainQuery = this.buildFindQueryWithParams(<any>where, orderBy, undefined, fields, 'main').take(undefined); // remove LIMIT

    let parameters = mainQuery.getParameters();

    const entityprooftwinRelFilter = entityprooftwinRel_some || entityprooftwinRel_none || entityprooftwinRel_every;

    if (entityprooftwinRelFilter) {
      const entityprooftwinRelQuery = this.entityprooftwinRelService
        .buildFindQueryWithParams(<any>entityprooftwinRelFilter, undefined, undefined, ['id'], 'entityprooftwinRel')
        .take(undefined); //remove the default LIMIT

      parameters = { ...parameters, ...entityprooftwinRelQuery.getParameters() };

      const subQueryFiltered = this.getQueryBuilder()
        .select([])
        .leftJoin(
          'twin.entityprooftwinRel',
          'entityprooftwinRel_filtered',
          `entityprooftwinRel_filtered.id IN (${entityprooftwinRelQuery.getQuery()})`
        )
        .groupBy('twin_id')
        .addSelect('count(entityprooftwinRel_filtered.id)', 'cnt_filtered')
        .addSelect('twin.id', 'twin_id');

      const subQueryTotal = this.getQueryBuilder()
        .select([])
        .leftJoin('twin.entityprooftwinRel', 'entityprooftwinRel_total')
        .groupBy('twin_id')
        .addSelect('count(entityprooftwinRel_total.id)', 'cnt_total')
        .addSelect('twin.id', 'twin_id');

      const subQuery = `
                SELECT
                    f.twin_id twin_id, f.cnt_filtered cnt_filtered, t.cnt_total cnt_total
                FROM
                    (${subQueryTotal.getQuery()}) t, (${subQueryFiltered.getQuery()}) f
                WHERE
                    t.twin_id = f.twin_id`;

      if (entityprooftwinRel_none) {
        mainQuery = mainQuery.andWhere(`twin.id IN
                (SELECT
                    entityprooftwinRel_subq.twin_id
                FROM
                    (${subQuery}) entityprooftwinRel_subq
                WHERE
                    entityprooftwinRel_subq.cnt_filtered = 0
                )`);
      }

      if (entityprooftwinRel_some) {
        mainQuery = mainQuery.andWhere(`twin.id IN
                (SELECT
                    entityprooftwinRel_subq.twin_id
                FROM
                    (${subQuery}) entityprooftwinRel_subq
                WHERE
                    entityprooftwinRel_subq.cnt_filtered > 0
                )`);
      }

      if (entityprooftwinRel_every) {
        mainQuery = mainQuery.andWhere(`twin.id IN
                (SELECT
                    entityprooftwinRel_subq.twin_id
                FROM
                    (${subQuery}) entityprooftwinRel_subq
                WHERE
                    entityprooftwinRel_subq.cnt_filtered > 0
                    AND entityprooftwinRel_subq.cnt_filtered = entityprooftwinRel_subq.cnt_total
                )`);
      }
    }

    mainQuery = mainQuery.setParameters(parameters);

    return mainQuery.take(limit || 50).skip(offset || 0);
  }
}
