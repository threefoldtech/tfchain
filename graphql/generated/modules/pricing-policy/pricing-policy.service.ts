import { Service, Inject } from 'typedi';
import { Repository, SelectQueryBuilder } from 'typeorm';
import { InjectRepository } from 'typeorm-typedi-extensions';
import { WhereInput, HydraBaseService } from '@subsquid/warthog';

import { PricingPolicy } from './pricing-policy.model';

import { PricingPolicyWhereArgs, PricingPolicyWhereInput } from '../../warthog';

import { Policy } from '../policy/policy.model';
import { PolicyService } from '../policy/policy.service';
import { getConnection, getRepository, In, Not } from 'typeorm';
import _ from 'lodash';

@Service('PricingPolicyService')
export class PricingPolicyService extends HydraBaseService<PricingPolicy> {
  @Inject('PolicyService')
  public readonly suService!: PolicyService;
  @Inject('PolicyService')
  public readonly cuService!: PolicyService;
  @Inject('PolicyService')
  public readonly nuService!: PolicyService;
  @Inject('PolicyService')
  public readonly ipuService!: PolicyService;

  constructor(@InjectRepository(PricingPolicy) protected readonly repository: Repository<PricingPolicy>) {
    super(PricingPolicy, repository);
  }

  async find<W extends WhereInput>(
    where?: any,
    orderBy?: string | string[],
    limit?: number,
    offset?: number,
    fields?: string[]
  ): Promise<PricingPolicy[]> {
    return this.findWithRelations<W>(where, orderBy, limit, offset, fields);
  }

  findWithRelations<W extends WhereInput>(
    _where?: any,
    orderBy?: string | string[],
    limit?: number,
    offset?: number,
    fields?: string[]
  ): Promise<PricingPolicy[]> {
    return this.buildFindWithRelationsQuery(_where, orderBy, limit, offset, fields).getMany();
  }

  buildFindWithRelationsQuery<W extends WhereInput>(
    _where?: any,
    orderBy?: string | string[],
    limit?: number,
    offset?: number,
    fields?: string[]
  ): SelectQueryBuilder<PricingPolicy> {
    const where = <PricingPolicyWhereInput>(_where || {});

    // remove relation filters to enable warthog query builders
    const { su } = where;
    delete where.su;

    // remove relation filters to enable warthog query builders
    const { cu } = where;
    delete where.cu;

    // remove relation filters to enable warthog query builders
    const { nu } = where;
    delete where.nu;

    // remove relation filters to enable warthog query builders
    const { ipu } = where;
    delete where.ipu;

    let mainQuery = this.buildFindQueryWithParams(<any>where, orderBy, undefined, fields, 'main').take(undefined); // remove LIMIT

    let parameters = mainQuery.getParameters();

    if (su) {
      // OTO or MTO
      const suQuery = this.suService
        .buildFindQueryWithParams(<any>su, undefined, undefined, ['id'], 'su')
        .take(undefined); // remove the default LIMIT

      mainQuery = mainQuery.andWhere(`"pricingpolicy"."su_id" IN (${suQuery.getQuery()})`);

      parameters = { ...parameters, ...suQuery.getParameters() };
    }

    if (cu) {
      // OTO or MTO
      const cuQuery = this.cuService
        .buildFindQueryWithParams(<any>cu, undefined, undefined, ['id'], 'cu')
        .take(undefined); // remove the default LIMIT

      mainQuery = mainQuery.andWhere(`"pricingpolicy"."cu_id" IN (${cuQuery.getQuery()})`);

      parameters = { ...parameters, ...cuQuery.getParameters() };
    }

    if (nu) {
      // OTO or MTO
      const nuQuery = this.nuService
        .buildFindQueryWithParams(<any>nu, undefined, undefined, ['id'], 'nu')
        .take(undefined); // remove the default LIMIT

      mainQuery = mainQuery.andWhere(`"pricingpolicy"."nu_id" IN (${nuQuery.getQuery()})`);

      parameters = { ...parameters, ...nuQuery.getParameters() };
    }

    if (ipu) {
      // OTO or MTO
      const ipuQuery = this.ipuService
        .buildFindQueryWithParams(<any>ipu, undefined, undefined, ['id'], 'ipu')
        .take(undefined); // remove the default LIMIT

      mainQuery = mainQuery.andWhere(`"pricingpolicy"."ipu_id" IN (${ipuQuery.getQuery()})`);

      parameters = { ...parameters, ...ipuQuery.getParameters() };
    }

    mainQuery = mainQuery.setParameters(parameters);

    return mainQuery.take(limit || 50).skip(offset || 0);
  }
}
