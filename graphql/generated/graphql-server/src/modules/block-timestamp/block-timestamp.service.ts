import { Service, Inject } from 'typedi';
import { Repository } from 'typeorm';
import { InjectRepository } from 'typeorm-typedi-extensions';
import { WhereInput } from 'warthog';
import { WarthogBaseService } from '../../WarthogBaseService';

import { BlockTimestamp } from './block-timestamp.model';

import {} from '../variants/variants.model';

import { BlockTimestampWhereArgs, BlockTimestampWhereInput } from '../../../generated';

@Service('BlockTimestampService')
export class BlockTimestampService extends WarthogBaseService<BlockTimestamp> {
  constructor(@InjectRepository(BlockTimestamp) protected readonly repository: Repository<BlockTimestamp>) {
    super(BlockTimestamp, repository);
  }

  async find<W extends WhereInput>(
    where?: any,
    orderBy?: string | string[],
    limit?: number,
    offset?: number,
    fields?: string[]
  ): Promise<BlockTimestamp[]> {
    let f = fields || [];

    return this.findWithRelations<W>(where, orderBy, limit, offset, f);
  }

  async findWithRelations<W extends WhereInput>(
    _where?: any,
    orderBy?: string | string[],
    limit?: number,
    offset?: number,
    fields?: string[]
  ): Promise<BlockTimestamp[]> {
    const where = <BlockTimestampWhereInput>(_where || {});

    let mainQuery = this.buildFindQueryWithParams(<any>where, orderBy, undefined, fields, 'main').take(undefined); // remove LIMIT

    let parameters = mainQuery.getParameters();

    mainQuery = mainQuery.setParameters(parameters);

    return mainQuery
      .take(limit || 50)
      .skip(offset || 0)
      .getMany();
  }
}
