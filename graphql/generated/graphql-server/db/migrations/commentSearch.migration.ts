import { MigrationInterface, QueryRunner } from "typeorm";

export class CommentSearchMigration1616661593347 implements MigrationInterface {
    name = 'commentSearchMigration1616661593347'

    public async up(queryRunner: QueryRunner): Promise<void> {
        // TODO: escape 
        await queryRunner.query(`
            ALTER TABLE transfer 
            ADD COLUMN comment_search_tsv tsvector 
            GENERATED ALWAYS AS (  
                    setweight(to_tsvector('english', coalesce("comment", '')), 'A') 
                ) 
            STORED;
        `);
        await queryRunner.query(`
            ALTER TABLE transfer 
            ADD COLUMN comment_search_doc text 
            GENERATED ALWAYS AS (  
                    coalesce("comment", '') 
                ) 
            STORED;
        `);
        await queryRunner.query(`CREATE INDEX comment_search_transfer_idx ON transfer USING GIN (comment_search_tsv)`);

        await queryRunner.query(`
            CREATE VIEW comment_search_view AS
            SELECT 
                text 'transfer' AS origin_table, id, comment_search_tsv AS tsv, comment_search_doc AS document 
            FROM
                transfer
        `);

    }

    public async down(queryRunner: QueryRunner): Promise<void> {
        await queryRunner.query(`DROP VIEW comment_search_view`);
        await queryRunner.query(`DROP INDEX comment_search_transfer_idx`);
        await queryRunner.query(`ALTER TABLE transfer DROP COLUMN comment_search_tsv`);
        await queryRunner.query(`ALTER TABLE transfer DROP COLUMN comment_search_doc`);
    }


}
