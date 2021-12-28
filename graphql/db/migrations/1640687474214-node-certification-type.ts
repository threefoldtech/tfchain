import {MigrationInterface, QueryRunner} from "typeorm";

export class nodeCertificationType1640687474214 implements MigrationInterface {
    name = 'nodeCertificationType1640687474214'

    public async up(queryRunner: QueryRunner): Promise<void> {
        await queryRunner.query(`ALTER TABLE "public"."node" ADD "certification_type" character varying NOT NULL`);
    }

    public async down(queryRunner: QueryRunner): Promise<void> {
        await queryRunner.query(`ALTER TABLE "public"."node" DROP COLUMN "certification_type"`);
    }

}
