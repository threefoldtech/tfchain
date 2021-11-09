import {
  BaseModel,
  BooleanField,
  DateField,
  FloatField,
  IntField,
  NumericField,
  JSONField,
  BytesField,
  EnumField,
  StringField,
  ObjectType,
} from '@subsquid/warthog';
import BN from 'bn.js';
import { InputType, Field } from 'type-graphql';

@InputType('PolicyInput')
@ObjectType()
export class Policy {
  @IntField({
    nullable: true,
  })
  value?: number;

  @StringField({
    nullable: true,
  })
  unit?: string;
}
