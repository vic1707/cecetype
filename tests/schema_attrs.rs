#![expect(clippy::as_conversions, reason = "test file")]

mod common;

use self::common::*;
use ::schema::{FieldSchema, Schema, StaticSchema, TypeSchema, VariantSchema};

#[rstest::rstest]
#[case::renamed_struct((Renamed, TypeSchema::UnitStruct { name: "TOTO" }))]
#[case::renamed_variant((RenamedVariant::Foo, TypeSchema::Enum { name: "RenamedVariant", variants: &[&VariantSchema::Unit { name: "FOO", discriminant: 0 }] as &[&_] }))]
#[case::renamed_field((RenamedField { foo: 0 }, TypeSchema::Struct { name: "TOTO", fields: &[&FieldSchema { name: "FOO", ty: &TypeSchema::U8 }] as &[&_] }))]
#[case::skipped_variant((SkippedVariant::Foo, TypeSchema::Enum { name: "SkippedVariant", variants: &[&VariantSchema::Unit { name: "Foo", discriminant: 0 }, &VariantSchema::Unit { name: "Baz", discriminant: 2 }] as &[&_] }))]
#[case::skipped_field((SkippedField { foo: 0, bar: 0, baz: 0 }, TypeSchema::Struct { name: "SkippedField", fields: &[&FieldSchema { name: "foo", ty: &TypeSchema::U8 }, &FieldSchema { name: "baz", ty: &TypeSchema::U8 }] as &[&_] }))]
#[case::skipped_tuple_field_newtype((SkippedTupleFieldMeansNewType(0, 1), TypeSchema::NewTypeStruct { name: "SkippedTupleFieldMeansNewType", field: &TypeSchema::U8 }))]
#[case::skipped_enum_tuple_field_newtype((EnumSkippedTupleFieldMeansNewType::Toto(0, 1), TypeSchema::Enum { name: "EnumSkippedTupleFieldMeansNewType", variants: &[&VariantSchema::NewType { name: "Toto", discriminant: 0, field: &TypeSchema::U8 }] as &[&_] }))]
#[case::from_into((FromIntoU8 { inner: 0 }, TypeSchema::U8))]
#[case::transparent((Transparent { foo: 0, bar: 0 }, TypeSchema::U8))]
fn schemas<T: Schema>(#[case] (_ty, expected_schema): (T, StaticSchema)) {
    assert_eq!(T::SCHEMA, &expected_schema);
}
