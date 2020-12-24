import { ObjectType, Field, Float, Int, Arg, Query, Resolver, createUnionType } from 'type-graphql';
import { Inject } from 'typedi';
import { Transfer } from '../transfer/transfer.model';
import { CommentSearchFTSService } from './commentSearch.service';

@ObjectType()
export class CommentSearchFTSOutput {
    @Field(type => CommentSearchSearchItem)
    item!: typeof CommentSearchSearchItem

    @Field(type => Float)
    rank!: number

    @Field(type => String)
    isTypeOf!: string

    @Field(type => String)
    highlight!: string
}

export const CommentSearchSearchItem = createUnionType({
    name: "CommentSearchSearchResult",
    types: () => [
        Transfer,
    ],
});


@Resolver()
export default class CommentSearchFTSResolver {

    constructor(@Inject('CommentSearchFTSService') readonly service: CommentSearchFTSService) {}

    @Query(() => [CommentSearchFTSOutput])
    async commentSearch(
        @Arg('text') query: string, 
        @Arg('limit', () => Int, { defaultValue: 5 }) limit: number):Promise<Array<CommentSearchFTSOutput>>{
        
        return this.service.search(query, limit);
    }

}