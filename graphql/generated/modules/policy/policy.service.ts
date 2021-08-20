import { Service, Inject } from 'typedi';
import { Repository } from 'typeorm';
import { InjectRepository } from 'typeorm-typedi-extensions';
import { WhereInput } from 'warthog';
import { WarthogBaseService } from '../../server/WarthogBaseService';

import { Policy } from './policy.model';

 

import { PolicyWhereArgs, PolicyWhereInput } from '../../warthog';

import { PricingPolicy } from '../pricing-policy/pricing-policy.model';
import { PricingPolicyService } from '../pricing-policy/pricing-policy.service';
import { getConnection, getRepository, In, Not } from 'typeorm';
import _ from 'lodash';

@Service('PolicyService')
export class PolicyService extends WarthogBaseService<Policy> {
  @Inject('PricingPolicyService')
  public readonly pricingpolicysuService!: PricingPolicyService;
  @Inject('PricingPolicyService')
  public readonly pricingpolicycuService!: PricingPolicyService;
  @Inject('PricingPolicyService')
  public readonly pricingpolicynuService!: PricingPolicyService;
  @Inject('PricingPolicyService')
  public readonly pricingpolicyipuService!: PricingPolicyService;

  constructor(@InjectRepository(Policy) protected readonly repository: Repository<Policy>) {
    super(Policy, repository);
  }

  async find<W extends WhereInput>(
    where?: any,
    orderBy?: string | string[],
    limit?: number,
    offset?: number,
    fields?: string[]
  ): Promise<Policy[]> {
    return this.findWithRelations<W>(where, orderBy, limit, offset, fields);
  }

  async findWithRelations<W extends WhereInput>(
    _where?: any,
    orderBy?: string | string[],
    limit?: number,
    offset?: number,
    fields?: string[]
  ): Promise<Policy[]> {
    const where = <PolicyWhereInput>(_where || {});

    // remove relation filters to enable warthog query builders

    const { pricingpolicysu_some, pricingpolicysu_none, pricingpolicysu_every } = where;

    if (+!!pricingpolicysu_some + +!!pricingpolicysu_none + +!!pricingpolicysu_every > 1) {
      throw new Error(`A query can have at most one of none, some, every clauses on a relation field`);
    }

    delete where.pricingpolicysu_some;
    delete where.pricingpolicysu_none;
    delete where.pricingpolicysu_every;
    // remove relation filters to enable warthog query builders

    const { pricingpolicycu_some, pricingpolicycu_none, pricingpolicycu_every } = where;

    if (+!!pricingpolicycu_some + +!!pricingpolicycu_none + +!!pricingpolicycu_every > 1) {
      throw new Error(`A query can have at most one of none, some, every clauses on a relation field`);
    }

    delete where.pricingpolicycu_some;
    delete where.pricingpolicycu_none;
    delete where.pricingpolicycu_every;
    // remove relation filters to enable warthog query builders

    const { pricingpolicynu_some, pricingpolicynu_none, pricingpolicynu_every } = where;

    if (+!!pricingpolicynu_some + +!!pricingpolicynu_none + +!!pricingpolicynu_every > 1) {
      throw new Error(`A query can have at most one of none, some, every clauses on a relation field`);
    }

    delete where.pricingpolicynu_some;
    delete where.pricingpolicynu_none;
    delete where.pricingpolicynu_every;
    // remove relation filters to enable warthog query builders

    const { pricingpolicyipu_some, pricingpolicyipu_none, pricingpolicyipu_every } = where;

    if (+!!pricingpolicyipu_some + +!!pricingpolicyipu_none + +!!pricingpolicyipu_every > 1) {
      throw new Error(`A query can have at most one of none, some, every clauses on a relation field`);
    }

    delete where.pricingpolicyipu_some;
    delete where.pricingpolicyipu_none;
    delete where.pricingpolicyipu_every;

    let mainQuery = this.buildFindQueryWithParams(<any>where, orderBy, undefined, fields, 'main').take(undefined); // remove LIMIT

    let parameters = mainQuery.getParameters();

    const pricingpolicysuFilter = pricingpolicysu_some || pricingpolicysu_none || pricingpolicysu_every;

    if (pricingpolicysuFilter) {
      const pricingpolicysuQuery = this.pricingpolicysuService
        .buildFindQueryWithParams(<any>pricingpolicysuFilter, undefined, undefined, ['id'], 'pricingpolicysu')
        .take(undefined); //remove the default LIMIT

      parameters = { ...parameters, ...pricingpolicysuQuery.getParameters() };

      const subQueryFiltered = this.getQueryBuilder()
        .select([])
        .leftJoin(
          'policy.pricingpolicysu',
          'pricingpolicysu_filtered',
          `pricingpolicysu_filtered.id IN (${pricingpolicysuQuery.getQuery()})`
        )
        .groupBy('policy_id')
        .addSelect('count(pricingpolicysu_filtered.id)', 'cnt_filtered')
        .addSelect('policy.id', 'policy_id');

      const subQueryTotal = this.getQueryBuilder()
        .select([])
        .leftJoin('policy.pricingpolicysu', 'pricingpolicysu_total')
        .groupBy('policy_id')
        .addSelect('count(pricingpolicysu_total.id)', 'cnt_total')
        .addSelect('policy.id', 'policy_id');

      const subQuery = `
                SELECT
                    f.policy_id policy_id, f.cnt_filtered cnt_filtered, t.cnt_total cnt_total
                FROM
                    (${subQueryTotal.getQuery()}) t, (${subQueryFiltered.getQuery()}) f
                WHERE
                    t.policy_id = f.policy_id`;

      if (pricingpolicysu_none) {
        mainQuery = mainQuery.andWhere(`policy.id IN
                (SELECT
                    pricingpolicysu_subq.policy_id
                FROM
                    (${subQuery}) pricingpolicysu_subq
                WHERE
                    pricingpolicysu_subq.cnt_filtered = 0
                )`);
      }

      if (pricingpolicysu_some) {
        mainQuery = mainQuery.andWhere(`policy.id IN
                (SELECT
                    pricingpolicysu_subq.policy_id
                FROM
                    (${subQuery}) pricingpolicysu_subq
                WHERE
                    pricingpolicysu_subq.cnt_filtered > 0
                )`);
      }

      if (pricingpolicysu_every) {
        mainQuery = mainQuery.andWhere(`policy.id IN
                (SELECT
                    pricingpolicysu_subq.policy_id
                FROM
                    (${subQuery}) pricingpolicysu_subq
                WHERE
                    pricingpolicysu_subq.cnt_filtered > 0
                    AND pricingpolicysu_subq.cnt_filtered = pricingpolicysu_subq.cnt_total
                )`);
      }
    }

    const pricingpolicycuFilter = pricingpolicycu_some || pricingpolicycu_none || pricingpolicycu_every;

    if (pricingpolicycuFilter) {
      const pricingpolicycuQuery = this.pricingpolicycuService
        .buildFindQueryWithParams(<any>pricingpolicycuFilter, undefined, undefined, ['id'], 'pricingpolicycu')
        .take(undefined); //remove the default LIMIT

      parameters = { ...parameters, ...pricingpolicycuQuery.getParameters() };

      const subQueryFiltered = this.getQueryBuilder()
        .select([])
        .leftJoin(
          'policy.pricingpolicycu',
          'pricingpolicycu_filtered',
          `pricingpolicycu_filtered.id IN (${pricingpolicycuQuery.getQuery()})`
        )
        .groupBy('policy_id')
        .addSelect('count(pricingpolicycu_filtered.id)', 'cnt_filtered')
        .addSelect('policy.id', 'policy_id');

      const subQueryTotal = this.getQueryBuilder()
        .select([])
        .leftJoin('policy.pricingpolicycu', 'pricingpolicycu_total')
        .groupBy('policy_id')
        .addSelect('count(pricingpolicycu_total.id)', 'cnt_total')
        .addSelect('policy.id', 'policy_id');

      const subQuery = `
                SELECT
                    f.policy_id policy_id, f.cnt_filtered cnt_filtered, t.cnt_total cnt_total
                FROM
                    (${subQueryTotal.getQuery()}) t, (${subQueryFiltered.getQuery()}) f
                WHERE
                    t.policy_id = f.policy_id`;

      if (pricingpolicycu_none) {
        mainQuery = mainQuery.andWhere(`policy.id IN
                (SELECT
                    pricingpolicycu_subq.policy_id
                FROM
                    (${subQuery}) pricingpolicycu_subq
                WHERE
                    pricingpolicycu_subq.cnt_filtered = 0
                )`);
      }

      if (pricingpolicycu_some) {
        mainQuery = mainQuery.andWhere(`policy.id IN
                (SELECT
                    pricingpolicycu_subq.policy_id
                FROM
                    (${subQuery}) pricingpolicycu_subq
                WHERE
                    pricingpolicycu_subq.cnt_filtered > 0
                )`);
      }

      if (pricingpolicycu_every) {
        mainQuery = mainQuery.andWhere(`policy.id IN
                (SELECT
                    pricingpolicycu_subq.policy_id
                FROM
                    (${subQuery}) pricingpolicycu_subq
                WHERE
                    pricingpolicycu_subq.cnt_filtered > 0
                    AND pricingpolicycu_subq.cnt_filtered = pricingpolicycu_subq.cnt_total
                )`);
      }
    }

    const pricingpolicynuFilter = pricingpolicynu_some || pricingpolicynu_none || pricingpolicynu_every;

    if (pricingpolicynuFilter) {
      const pricingpolicynuQuery = this.pricingpolicynuService
        .buildFindQueryWithParams(<any>pricingpolicynuFilter, undefined, undefined, ['id'], 'pricingpolicynu')
        .take(undefined); //remove the default LIMIT

      parameters = { ...parameters, ...pricingpolicynuQuery.getParameters() };

      const subQueryFiltered = this.getQueryBuilder()
        .select([])
        .leftJoin(
          'policy.pricingpolicynu',
          'pricingpolicynu_filtered',
          `pricingpolicynu_filtered.id IN (${pricingpolicynuQuery.getQuery()})`
        )
        .groupBy('policy_id')
        .addSelect('count(pricingpolicynu_filtered.id)', 'cnt_filtered')
        .addSelect('policy.id', 'policy_id');

      const subQueryTotal = this.getQueryBuilder()
        .select([])
        .leftJoin('policy.pricingpolicynu', 'pricingpolicynu_total')
        .groupBy('policy_id')
        .addSelect('count(pricingpolicynu_total.id)', 'cnt_total')
        .addSelect('policy.id', 'policy_id');

      const subQuery = `
                SELECT
                    f.policy_id policy_id, f.cnt_filtered cnt_filtered, t.cnt_total cnt_total
                FROM
                    (${subQueryTotal.getQuery()}) t, (${subQueryFiltered.getQuery()}) f
                WHERE
                    t.policy_id = f.policy_id`;

      if (pricingpolicynu_none) {
        mainQuery = mainQuery.andWhere(`policy.id IN
                (SELECT
                    pricingpolicynu_subq.policy_id
                FROM
                    (${subQuery}) pricingpolicynu_subq
                WHERE
                    pricingpolicynu_subq.cnt_filtered = 0
                )`);
      }

      if (pricingpolicynu_some) {
        mainQuery = mainQuery.andWhere(`policy.id IN
                (SELECT
                    pricingpolicynu_subq.policy_id
                FROM
                    (${subQuery}) pricingpolicynu_subq
                WHERE
                    pricingpolicynu_subq.cnt_filtered > 0
                )`);
      }

      if (pricingpolicynu_every) {
        mainQuery = mainQuery.andWhere(`policy.id IN
                (SELECT
                    pricingpolicynu_subq.policy_id
                FROM
                    (${subQuery}) pricingpolicynu_subq
                WHERE
                    pricingpolicynu_subq.cnt_filtered > 0
                    AND pricingpolicynu_subq.cnt_filtered = pricingpolicynu_subq.cnt_total
                )`);
      }
    }

    const pricingpolicyipuFilter = pricingpolicyipu_some || pricingpolicyipu_none || pricingpolicyipu_every;

    if (pricingpolicyipuFilter) {
      const pricingpolicyipuQuery = this.pricingpolicyipuService
        .buildFindQueryWithParams(<any>pricingpolicyipuFilter, undefined, undefined, ['id'], 'pricingpolicyipu')
        .take(undefined); //remove the default LIMIT

      parameters = { ...parameters, ...pricingpolicyipuQuery.getParameters() };

      const subQueryFiltered = this.getQueryBuilder()
        .select([])
        .leftJoin(
          'policy.pricingpolicyipu',
          'pricingpolicyipu_filtered',
          `pricingpolicyipu_filtered.id IN (${pricingpolicyipuQuery.getQuery()})`
        )
        .groupBy('policy_id')
        .addSelect('count(pricingpolicyipu_filtered.id)', 'cnt_filtered')
        .addSelect('policy.id', 'policy_id');

      const subQueryTotal = this.getQueryBuilder()
        .select([])
        .leftJoin('policy.pricingpolicyipu', 'pricingpolicyipu_total')
        .groupBy('policy_id')
        .addSelect('count(pricingpolicyipu_total.id)', 'cnt_total')
        .addSelect('policy.id', 'policy_id');

      const subQuery = `
                SELECT
                    f.policy_id policy_id, f.cnt_filtered cnt_filtered, t.cnt_total cnt_total
                FROM
                    (${subQueryTotal.getQuery()}) t, (${subQueryFiltered.getQuery()}) f
                WHERE
                    t.policy_id = f.policy_id`;

      if (pricingpolicyipu_none) {
        mainQuery = mainQuery.andWhere(`policy.id IN
                (SELECT
                    pricingpolicyipu_subq.policy_id
                FROM
                    (${subQuery}) pricingpolicyipu_subq
                WHERE
                    pricingpolicyipu_subq.cnt_filtered = 0
                )`);
      }

      if (pricingpolicyipu_some) {
        mainQuery = mainQuery.andWhere(`policy.id IN
                (SELECT
                    pricingpolicyipu_subq.policy_id
                FROM
                    (${subQuery}) pricingpolicyipu_subq
                WHERE
                    pricingpolicyipu_subq.cnt_filtered > 0
                )`);
      }

      if (pricingpolicyipu_every) {
        mainQuery = mainQuery.andWhere(`policy.id IN
                (SELECT
                    pricingpolicyipu_subq.policy_id
                FROM
                    (${subQuery}) pricingpolicyipu_subq
                WHERE
                    pricingpolicyipu_subq.cnt_filtered > 0
                    AND pricingpolicyipu_subq.cnt_filtered = pricingpolicyipu_subq.cnt_total
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
