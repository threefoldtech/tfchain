import { Service, Inject } from 'typedi';
import { Repository } from 'typeorm';
import { InjectRepository } from 'typeorm-typedi-extensions';
import { WhereInput } from 'warthog';
import { WarthogBaseService } from '../../WarthogBaseService';

import { Entity } from './entity.model';

import {} from '../variants/variants.model';

import { EntityWhereArgs, EntityWhereInput } from '../../../generated';

@Service('EntityService')
export class EntityService extends WarthogBaseService<Entity> {
  constructor(@InjectRepository(Entity) protected readonly repository: Repository<Entity>) {
    super(Entity, repository);
  }

  async find<W extends WhereInput>(
    where?: any,
    orderBy?: string | string[],
    limit?: number,
    offset?: number,
    fields?: string[]
  ): Promise<Entity[]> {
    let f = fields || [];

    return this.findWithRelations<W>(where, orderBy, limit, offset, f);
  }

  async findWithRelations<W extends WhereInput>(
    _where?: any,
    orderBy?: string | string[],
    limit?: number,
    offset?: number,
    fields?: string[]
  ): Promise<Entity[]> {
    const where = <EntityWhereInput>(_where || {});

    let mainQuery = this.buildFindQueryWithParams(<any>where, orderBy, undefined, fields, 'main').take(undefined); // remove LIMIT

    let parameters = mainQuery.getParameters();

    mainQuery = mainQuery.setParameters(parameters);

    return mainQuery
      .take(limit || 50)
      .skip(offset || 0)
      .getMany();
  }
}
