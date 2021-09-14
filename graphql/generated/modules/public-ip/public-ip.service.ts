import { Service, Inject } from 'typedi';
import { Repository, SelectQueryBuilder } from 'typeorm';
import { InjectRepository } from 'typeorm-typedi-extensions';
import { WhereInput, HydraBaseService } from '@subsquid/warthog';

import { PublicIp } from './public-ip.model';

import { PublicIpWhereArgs, PublicIpWhereInput } from '../../warthog';

import { Farm } from '../farm/farm.model';
import { FarmService } from '../farm/farm.service';
import { getConnection, getRepository, In, Not } from 'typeorm';
import _ from 'lodash';

@Service('PublicIpService')
export class PublicIpService extends HydraBaseService<PublicIp> {
  @Inject('FarmService')
  public readonly farmService!: FarmService;

  constructor(@InjectRepository(PublicIp) protected readonly repository: Repository<PublicIp>) {
    super(PublicIp, repository);
  }

  async find<W extends WhereInput>(
    where?: any,
    orderBy?: string | string[],
    limit?: number,
    offset?: number,
    fields?: string[]
  ): Promise<PublicIp[]> {
    return this.findWithRelations<W>(where, orderBy, limit, offset, fields);
  }

  findWithRelations<W extends WhereInput>(
    _where?: any,
    orderBy?: string | string[],
    limit?: number,
    offset?: number,
    fields?: string[]
  ): Promise<PublicIp[]> {
    return this.buildFindWithRelationsQuery(_where, orderBy, limit, offset, fields).getMany();
  }

  buildFindWithRelationsQuery<W extends WhereInput>(
    _where?: any,
    orderBy?: string | string[],
    limit?: number,
    offset?: number,
    fields?: string[]
  ): SelectQueryBuilder<PublicIp> {
    const where = <PublicIpWhereInput>(_where || {});

    // remove relation filters to enable warthog query builders
    const { farm } = where;
    delete where.farm;

    let mainQuery = this.buildFindQueryWithParams(<any>where, orderBy, undefined, fields, 'main').take(undefined); // remove LIMIT

    let parameters = mainQuery.getParameters();

    if (farm) {
      // OTO or MTO
      const farmQuery = this.farmService
        .buildFindQueryWithParams(<any>farm, undefined, undefined, ['id'], 'farm')
        .take(undefined); // remove the default LIMIT

      mainQuery = mainQuery.andWhere(`"publicip"."farm_id" IN (${farmQuery.getQuery()})`);

      parameters = { ...parameters, ...farmQuery.getParameters() };
    }

    mainQuery = mainQuery.setParameters(parameters);

    return mainQuery.take(limit || 50).skip(offset || 0);
  }
}
