import { Service, Inject } from 'typedi';
import { Repository } from 'typeorm';
import { InjectRepository } from 'typeorm-typedi-extensions';
import { WhereInput } from 'warthog';
import { WarthogBaseService } from '../../server/WarthogBaseService';

import { Node } from './node.model';

import { NodeWhereArgs, NodeWhereInput } from '../../warthog';

import { Location } from '../location/location.model';
import { LocationService } from '../location/location.service';
import { PublicConfig } from '../public-config/public-config.model';
import { PublicConfigService } from '../public-config/public-config.service';
import { getConnection, getRepository, In, Not } from 'typeorm';
import _ from 'lodash';

@Service('NodeService')
export class NodeService extends WarthogBaseService<Node> {
  @Inject('LocationService')
  public readonly locationService!: LocationService;
  @Inject('PublicConfigService')
  public readonly publicConfigService!: PublicConfigService;

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

  async findWithRelations<W extends WhereInput>(
    _where?: any,
    orderBy?: string | string[],
    limit?: number,
    offset?: number,
    fields?: string[]
  ): Promise<Node[]> {
    const where = <NodeWhereInput>(_where || {});

    // remove relation filters to enable warthog query builders
    const { location } = where;
    delete where.location;

    // remove relation filters to enable warthog query builders
    const { publicConfig } = where;
    delete where.publicConfig;

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

    if (publicConfig) {
      // OTO or MTO
      const publicConfigQuery = this.publicConfigService
        .buildFindQueryWithParams(<any>publicConfig, undefined, undefined, ['id'], 'publicConfig')
        .take(undefined); // remove the default LIMIT

      mainQuery = mainQuery.andWhere(`"node"."public_config_id" IN (${publicConfigQuery.getQuery()})`);

      parameters = { ...parameters, ...publicConfigQuery.getParameters() };
    }

    mainQuery = mainQuery.setParameters(parameters);

    return mainQuery
      .take(limit || 50)
      .skip(offset || 0)
      .getMany();
  }
}
