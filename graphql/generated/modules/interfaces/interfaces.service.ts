import { Service, Inject } from 'typedi';
import { Repository, SelectQueryBuilder } from 'typeorm';
import { InjectRepository } from 'typeorm-typedi-extensions';
import { WhereInput, HydraBaseService } from '@subsquid/warthog';

import { Interfaces } from './interfaces.model';

import { InterfacesWhereArgs, InterfacesWhereInput } from '../../warthog';

import { Node } from '../node/node.model';
import { NodeService } from '../node/node.service';
import { getConnection, getRepository, In, Not } from 'typeorm';
import _ from 'lodash';

@Service('InterfacesService')
export class InterfacesService extends HydraBaseService<Interfaces> {
  @Inject('NodeService')
  public readonly nodeService!: NodeService;

  constructor(@InjectRepository(Interfaces) protected readonly repository: Repository<Interfaces>) {
    super(Interfaces, repository);
  }

  async find<W extends WhereInput>(
    where?: any,
    orderBy?: string | string[],
    limit?: number,
    offset?: number,
    fields?: string[]
  ): Promise<Interfaces[]> {
    return this.findWithRelations<W>(where, orderBy, limit, offset, fields);
  }

  findWithRelations<W extends WhereInput>(
    _where?: any,
    orderBy?: string | string[],
    limit?: number,
    offset?: number,
    fields?: string[]
  ): Promise<Interfaces[]> {
    return this.buildFindWithRelationsQuery(_where, orderBy, limit, offset, fields).getMany();
  }

  buildFindWithRelationsQuery<W extends WhereInput>(
    _where?: any,
    orderBy?: string | string[],
    limit?: number,
    offset?: number,
    fields?: string[]
  ): SelectQueryBuilder<Interfaces> {
    const where = <InterfacesWhereInput>(_where || {});

    // remove relation filters to enable warthog query builders
    const { node } = where;
    delete where.node;

    let mainQuery = this.buildFindQueryWithParams(<any>where, orderBy, undefined, fields, 'main').take(undefined); // remove LIMIT

    let parameters = mainQuery.getParameters();

    if (node) {
      // OTO or MTO
      const nodeQuery = this.nodeService
        .buildFindQueryWithParams(<any>node, undefined, undefined, ['id'], 'node')
        .take(undefined); // remove the default LIMIT

      mainQuery = mainQuery.andWhere(`"interfaces"."node_id" IN (${nodeQuery.getQuery()})`);

      parameters = { ...parameters, ...nodeQuery.getParameters() };
    }

    mainQuery = mainQuery.setParameters(parameters);

    return mainQuery.take(limit || 50).skip(offset || 0);
  }
}
