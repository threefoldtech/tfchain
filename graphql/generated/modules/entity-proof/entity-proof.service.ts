import { Service, Inject } from 'typedi';
import { Repository, SelectQueryBuilder } from 'typeorm';
import { InjectRepository } from 'typeorm-typedi-extensions';
import { WhereInput, HydraBaseService } from '@subsquid/warthog';

import { EntityProof } from './entity-proof.model';

import { EntityProofWhereArgs, EntityProofWhereInput } from '../../warthog';

import { Twin } from '../twin/twin.model';
import { TwinService } from '../twin/twin.service';
import { getConnection, getRepository, In, Not } from 'typeorm';
import _ from 'lodash';

@Service('EntityProofService')
export class EntityProofService extends HydraBaseService<EntityProof> {
  @Inject('TwinService')
  public readonly twinRelService!: TwinService;

  constructor(@InjectRepository(EntityProof) protected readonly repository: Repository<EntityProof>) {
    super(EntityProof, repository);
  }

  async find<W extends WhereInput>(
    where?: any,
    orderBy?: string | string[],
    limit?: number,
    offset?: number,
    fields?: string[]
  ): Promise<EntityProof[]> {
    return this.findWithRelations<W>(where, orderBy, limit, offset, fields);
  }

  findWithRelations<W extends WhereInput>(
    _where?: any,
    orderBy?: string | string[],
    limit?: number,
    offset?: number,
    fields?: string[]
  ): Promise<EntityProof[]> {
    return this.buildFindWithRelationsQuery(_where, orderBy, limit, offset, fields).getMany();
  }

  buildFindWithRelationsQuery<W extends WhereInput>(
    _where?: any,
    orderBy?: string | string[],
    limit?: number,
    offset?: number,
    fields?: string[]
  ): SelectQueryBuilder<EntityProof> {
    const where = <EntityProofWhereInput>(_where || {});

    // remove relation filters to enable warthog query builders
    const { twinRel } = where;
    delete where.twinRel;

    let mainQuery = this.buildFindQueryWithParams(<any>where, orderBy, undefined, fields, 'main').take(undefined); // remove LIMIT

    let parameters = mainQuery.getParameters();

    if (twinRel) {
      // OTO or MTO
      const twinRelQuery = this.twinRelService
        .buildFindQueryWithParams(<any>twinRel, undefined, undefined, ['id'], 'twinRel')
        .take(undefined); // remove the default LIMIT

      mainQuery = mainQuery.andWhere(`"entityproof"."twin_rel_id" IN (${twinRelQuery.getQuery()})`);

      parameters = { ...parameters, ...twinRelQuery.getParameters() };
    }

    mainQuery = mainQuery.setParameters(parameters);

    return mainQuery.take(limit || 50).skip(offset || 0);
  }
}
