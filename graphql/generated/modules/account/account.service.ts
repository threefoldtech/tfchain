import { Service, Inject } from 'typedi';
import { Repository, SelectQueryBuilder } from 'typeorm';
import { InjectRepository } from 'typeorm-typedi-extensions';
import { WhereInput, HydraBaseService } from '@subsquid/warthog';

import { Account } from './account.model';

import { AccountWhereArgs, AccountWhereInput } from '../../warthog';

import { HistoricalBalance } from '../historical-balance/historical-balance.model';
import { HistoricalBalanceService } from '../historical-balance/historical-balance.service';
import { getConnection, getRepository, In, Not } from 'typeorm';
import _ from 'lodash';

@Service('AccountService')
export class AccountService extends HydraBaseService<Account> {
  @Inject('HistoricalBalanceService')
  public readonly historicalBalancesService!: HistoricalBalanceService;

  constructor(@InjectRepository(Account) protected readonly repository: Repository<Account>) {
    super(Account, repository);
  }

  async find<W extends WhereInput>(
    where?: any,
    orderBy?: string | string[],
    limit?: number,
    offset?: number,
    fields?: string[]
  ): Promise<Account[]> {
    return this.findWithRelations<W>(where, orderBy, limit, offset, fields);
  }

  findWithRelations<W extends WhereInput>(
    _where?: any,
    orderBy?: string | string[],
    limit?: number,
    offset?: number,
    fields?: string[]
  ): Promise<Account[]> {
    return this.buildFindWithRelationsQuery(_where, orderBy, limit, offset, fields).getMany();
  }

  buildFindWithRelationsQuery<W extends WhereInput>(
    _where?: any,
    orderBy?: string | string[],
    limit?: number,
    offset?: number,
    fields?: string[]
  ): SelectQueryBuilder<Account> {
    const where = <AccountWhereInput>(_where || {});

    // remove relation filters to enable warthog query builders

    const { historicalBalances_some, historicalBalances_none, historicalBalances_every } = where;

    if (+!!historicalBalances_some + +!!historicalBalances_none + +!!historicalBalances_every > 1) {
      throw new Error(`A query can have at most one of none, some, every clauses on a relation field`);
    }

    delete where.historicalBalances_some;
    delete where.historicalBalances_none;
    delete where.historicalBalances_every;

    let mainQuery = this.buildFindQueryWithParams(<any>where, orderBy, undefined, fields, 'main').take(undefined); // remove LIMIT

    let parameters = mainQuery.getParameters();

    const historicalBalancesFilter = historicalBalances_some || historicalBalances_none || historicalBalances_every;

    if (historicalBalancesFilter) {
      const historicalBalancesQuery = this.historicalBalancesService
        .buildFindQueryWithParams(<any>historicalBalancesFilter, undefined, undefined, ['id'], 'historicalBalances')
        .take(undefined); //remove the default LIMIT

      parameters = { ...parameters, ...historicalBalancesQuery.getParameters() };

      const subQueryFiltered = this.getQueryBuilder()
        .select([])
        .leftJoin(
          'account.historicalBalances',
          'historicalBalances_filtered',
          `historicalBalances_filtered.id IN (${historicalBalancesQuery.getQuery()})`
        )
        .groupBy('account_id')
        .addSelect('count(historicalBalances_filtered.id)', 'cnt_filtered')
        .addSelect('account.id', 'account_id');

      const subQueryTotal = this.getQueryBuilder()
        .select([])
        .leftJoin('account.historicalBalances', 'historicalBalances_total')
        .groupBy('account_id')
        .addSelect('count(historicalBalances_total.id)', 'cnt_total')
        .addSelect('account.id', 'account_id');

      const subQuery = `
                SELECT
                    f.account_id account_id, f.cnt_filtered cnt_filtered, t.cnt_total cnt_total
                FROM
                    (${subQueryTotal.getQuery()}) t, (${subQueryFiltered.getQuery()}) f
                WHERE
                    t.account_id = f.account_id`;

      if (historicalBalances_none) {
        mainQuery = mainQuery.andWhere(`account.id IN
                (SELECT
                    historicalBalances_subq.account_id
                FROM
                    (${subQuery}) historicalBalances_subq
                WHERE
                    historicalBalances_subq.cnt_filtered = 0
                )`);
      }

      if (historicalBalances_some) {
        mainQuery = mainQuery.andWhere(`account.id IN
                (SELECT
                    historicalBalances_subq.account_id
                FROM
                    (${subQuery}) historicalBalances_subq
                WHERE
                    historicalBalances_subq.cnt_filtered > 0
                )`);
      }

      if (historicalBalances_every) {
        mainQuery = mainQuery.andWhere(`account.id IN
                (SELECT
                    historicalBalances_subq.account_id
                FROM
                    (${subQuery}) historicalBalances_subq
                WHERE
                    historicalBalances_subq.cnt_filtered > 0
                    AND historicalBalances_subq.cnt_filtered = historicalBalances_subq.cnt_total
                )`);
      }
    }

    mainQuery = mainQuery.setParameters(parameters);

    return mainQuery.take(limit || 50).skip(offset || 0);
  }
}
