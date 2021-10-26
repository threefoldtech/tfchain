import {MigrationInterface, QueryRunner} from "typeorm";

export class burnmodule1635253562146 implements MigrationInterface {
    name = 'burnmodule1635253562146'

    public async up(queryRunner: QueryRunner): Promise<void> {
        await queryRunner.query(`CREATE TABLE "burn_transaction" ("id" character varying NOT NULL, "created_at" TIMESTAMP NOT NULL DEFAULT now(), "created_by_id" character varying NOT NULL, "updated_at" TIMESTAMP DEFAULT now(), "updated_by_id" character varying, "deleted_at" TIMESTAMP, "deleted_by_id" character varying, "version" integer NOT NULL, "block" integer NOT NULL, "amount" numeric NOT NULL, "target" character varying NOT NULL, CONSTRAINT "PK_20ec76c5c56dd6b47dec5f0aaa8" PRIMARY KEY ("id"))`);
        await queryRunner.query(`CREATE TABLE "mint_transaction" ("id" character varying NOT NULL, "created_at" TIMESTAMP NOT NULL DEFAULT now(), "created_by_id" character varying NOT NULL, "updated_at" TIMESTAMP DEFAULT now(), "updated_by_id" character varying, "deleted_at" TIMESTAMP, "deleted_by_id" character varying, "version" integer NOT NULL, "amount" numeric NOT NULL, "target" character varying NOT NULL, "block" integer NOT NULL, CONSTRAINT "PK_19f4328320501dfd14e2bae0855" PRIMARY KEY ("id"))`);
        await queryRunner.query(`CREATE TABLE "refund_transaction" ("id" character varying NOT NULL, "created_at" TIMESTAMP NOT NULL DEFAULT now(), "created_by_id" character varying NOT NULL, "updated_at" TIMESTAMP DEFAULT now(), "updated_by_id" character varying, "deleted_at" TIMESTAMP, "deleted_by_id" character varying, "version" integer NOT NULL, "block" integer NOT NULL, "amount" numeric NOT NULL, "target" character varying NOT NULL, "tx_hash" character varying NOT NULL, CONSTRAINT "PK_74ffc5427c595968dd777f71bf4" PRIMARY KEY ("id"))`);
    }

    public async down(queryRunner: QueryRunner): Promise<void> {
        await queryRunner.query(`DROP TABLE "refund_transaction"`);
        await queryRunner.query(`DROP TABLE "mint_transaction"`);
        await queryRunner.query(`DROP TABLE "burn_transaction"`);
    }

}
