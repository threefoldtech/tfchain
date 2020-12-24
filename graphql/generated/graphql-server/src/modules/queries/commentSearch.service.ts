import { Service } from 'typedi';
import { Transfer } from '../transfer/transfer.model';

import { InjectRepository } from 'typeorm-typedi-extensions';
import { Repository, getConnection, EntityManager } from 'typeorm';

import { CommentSearchFTSOutput } from './commentSearch.resolver';

interface RawSQLResult {
    origin_table: string,
    id: string,
    rank: number,
    highlight: string
}

@Service('CommentSearchFTSService')
export class CommentSearchFTSService {
    readonly transferRepository: Repository<Transfer>;

    constructor(@InjectRepository(Transfer) transferRepository: Repository<Transfer>
                 ) {
        this.transferRepository = transferRepository;
    }

    async search(text: string, limit:number = 5): Promise<CommentSearchFTSOutput[]> {
        
        return getConnection().transaction<CommentSearchFTSOutput[]>('REPEATABLE READ', async (em: EntityManager) => {
            const query = `
            SELECT origin_table, id, 
                ts_rank(tsv, phraseto_tsquery('english', $1)) as rank,
                ts_headline(document, phraseto_tsquery('english', $1)) as highlight
            FROM comment_search_view
            WHERE phraseto_tsquery('english', $1) @@ tsv
            ORDER BY rank DESC
            LIMIT $2`;

            const results = await em.query(query, [text, limit]) as RawSQLResult[];

            if (results.length == 0) {
                return [];
            }

            const idMap:{ [id:string]: RawSQLResult } = {};
            results.forEach(item => idMap[item.id] = item);
            const ids: string[] = results.map(item => item.id);
            
            const transfers: Transfer[] = await em.createQueryBuilder<Transfer>(Transfer, 'Transfer')
                        .where("id IN (:...ids)", { ids }).getMany();

            const enhancedEntities = [...transfers ].map((e) => {
                return { item: e, 
                    rank: idMap[e.id].rank, 
                    highlight: idMap[e.id].highlight,
                    isTypeOf: idMap[e.id].origin_table } as CommentSearchFTSOutput;
            });

            return enhancedEntities.reduce((accum: CommentSearchFTSOutput[], entity) => {
                if (entity.rank > 0) {
                    accum.push(entity);
                }
                return accum;
            }, []).sort((a,b) => b.rank - a.rank);

        })
    }
}