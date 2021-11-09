import { Service, Inject } from 'typedi';
import { Repository, SelectQueryBuilder } from 'typeorm';
import { InjectRepository } from 'typeorm-typedi-extensions';
import { WhereInput, HydraBaseService } from '@subsquid/warthog';

import { Node } from './node.model';

import { NodeWhereArgs, NodeWhereInput } from '../../warthog';

import { Location } from '../location/location.model';
import { LocationService } from '../location/location.service';
import { Interfaces } from '../interfaces/interfaces.model';
import { InterfacesService } from '../interfaces/interfaces.service';
import { getConnection, getRepository, In, Not } from 'typeorm';
import _ from 'lodash';

@Service('NodeService')
export class NodeService extends HydraBaseService<Node> {
  @Inject('LocationService')
  public readonly locationService!: LocationService;
  @Inject('InterfacesService')
  public readonly interfacesService!: InterfacesService;

  constructor(@InjectRepository(Node) protected readonly repository: Repository<Node>) {
    super(Node, repository);
  }

  async find<W extends WhereInput>(
    where?: any,
    orderBy?: string | string[],
    limit?: number,
    offset?: number,
    fields?: string[]
  ): Promise<Node[]> {
    return this.findWithRelations<W>(where, orderBy, limit, offset, fields);
  }

  findWithRelations<W extends WhereInput>(
    _where?: any,
    orderBy?: string | string[],
    limit?: number,
    offset?: number,
    fields?: string[]
  ): Promise<Node[]> {
    return this.buildFindWithRelationsQuery(_where, orderBy, limit, offset, fields).getMany();
  }

  buildFindWithRelationsQuery<W extends WhereInput>(
    _where?: any,
    orderBy?: string | string[],
    limit?: number,
    offset?: number,
    fields?: string[]
  ): SelectQueryBuilder<Node> {
    const where = <NodeWhereInput>(_where || {});

    // remove relation filters to enable warthog query builders
    const { location } = where;
    delete where.location;

    // remove relation filters to enable warthog query builders

    const { interfaces_some, interfaces_none, interfaces_every } = where;

    if (+!!interfaces_some + +!!interfaces_none + +!!interfaces_every > 1) {
      throw new Error(`A query can have at most one of none, some, every clauses on a relation field`);
    }

    delete where.interfaces_some;
    delete where.interfaces_none;
    delete where.interfaces_every;

    let mainQuery = this.buildFindQueryWithParams(<any>where, orderBy, undefined, fields, 'main').take(undefined); // remove LIMIT

    let parameters = mainQuery.getParameters();

    if (location) {
      // OTO or MTO
      const locationQuery = this.locationService
        .buildFindQueryWithParams(<any>location, undefined, undefined, ['id'], 'location')
        .take(undefined); // remove the default LIMIT

      mainQuery = mainQuery.andWhere(`"node"."location_id" IN (${locationQuery.getQuery()})`);

      parameters = { ...parameters, ...locationQuery.getParameters() };
    }

    const interfacesFilter = interfaces_some || interfaces_none || interfaces_every;

    if (interfacesFilter) {
      const interfacesQuery = this.interfacesService
        .buildFindQueryWithParams(<any>interfacesFilter, undefined, undefined, ['id'], 'interfaces')
        .take(undefined); //remove the default LIMIT

      parameters = { ...parameters, ...interfacesQuery.getParameters() };

      const subQueryFiltered = this.getQueryBuilder()
        .select([])
        .leftJoin('node.interfaces', 'interfaces_filtered', `interfaces_filtered.id IN (${interfacesQuery.getQuery()})`)
        .groupBy('node_id')
        .addSelect('count(interfaces_filtered.id)', 'cnt_filtered')
        .addSelect('node.id', 'node_id');

      const subQueryTotal = this.getQueryBuilder()
        .select([])
        .leftJoin('node.interfaces', 'interfaces_total')
        .groupBy('node_id')
        .addSelect('count(interfaces_total.id)', 'cnt_total')
        .addSelect('node.id', 'node_id');

      const subQuery = `
                SELECT
                    f.node_id node_id, f.cnt_filtered cnt_filtered, t.cnt_total cnt_total
                FROM
                    (${subQueryTotal.getQuery()}) t, (${subQueryFiltered.getQuery()}) f
                WHERE
                    t.node_id = f.node_id`;

      if (interfaces_none) {
        mainQuery = mainQuery.andWhere(`node.id IN
                (SELECT
                    interfaces_subq.node_id
                FROM
                    (${subQuery}) interfaces_subq
                WHERE
                    interfaces_subq.cnt_filtered = 0
                )`);
      }

      if (interfaces_some) {
        mainQuery = mainQuery.andWhere(`node.id IN
                (SELECT
                    interfaces_subq.node_id
                FROM
                    (${subQuery}) interfaces_subq
                WHERE
                    interfaces_subq.cnt_filtered > 0
                )`);
      }

      if (interfaces_every) {
        mainQuery = mainQuery.andWhere(`node.id IN
                (SELECT
                    interfaces_subq.node_id
                FROM
                    (${subQuery}) interfaces_subq
                WHERE
                    interfaces_subq.cnt_filtered > 0
                    AND interfaces_subq.cnt_filtered = interfaces_subq.cnt_total
                )`);
      }
    }

    mainQuery = mainQuery.setParameters(parameters);

    return mainQuery.take(limit || 50).skip(offset || 0);
  }
}
