#![expect(clippy::as_conversions, reason = "test file")]

mod common;

use self::common::*;
use ::{
    schema::{FieldSchema, Schema, StaticSchema, TypeSchema, VariantSchema},
    serde::{Deserialize, Serialize},
};

#[derive(Serialize, Deserialize, Schema, PartialEq, Eq, Debug)]
#[schema(as(u8))]
struct SchemaAttrOnItem {
    foo_bar: u16,
}

#[derive(Serialize, Deserialize, Schema, PartialEq, Eq, Debug)]
struct SchemaAttrOnField {
    #[schema(as(u16))]
    foo: u8,
}

#[derive(Serialize, Deserialize, Schema, PartialEq, Eq, Debug)]
enum SchemaAttrOnVariant {
    #[schema(as(u8))]
    Foo { foo: u16 },
}

#[derive(Serialize, Deserialize, Schema, PartialEq, Eq, Debug)]
enum SchemaAttrOnTupleVariant {
    #[schema(as(u8))]
    Foo(u16, bool),
}

struct NotSchema;

#[derive(Schema)]
#[schema(bounds())]
struct SchemaAttrOnBound<T>(#[schema(as(u8))] T);

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
#[case::schema_attr_item((SchemaAttrOnItem { foo_bar: 0 }, TypeSchema::U8))]
#[case::schema_attr_field((SchemaAttrOnField { foo: 0 }, TypeSchema::Struct { name: "SchemaAttrOnField", fields: &[&FieldSchema { name: "foo", ty: &TypeSchema::U16 }] as &[&_] }))]
#[case::schema_attr_variant((SchemaAttrOnVariant::Foo { foo: 0 }, TypeSchema::Enum { name: "SchemaAttrOnVariant", variants: &[&VariantSchema::NewType { name: "Foo", discriminant: 0, field: &TypeSchema::U8 }] as &[&_] }))]
#[case::schema_attr_tuple_variant((SchemaAttrOnTupleVariant::Foo(0, false), TypeSchema::Enum { name: "SchemaAttrOnTupleVariant", variants: &[&VariantSchema::NewType { name: "Foo", discriminant: 0, field: &TypeSchema::U8 }] as &[&_] }))]
#[case::schema_attr_bound((SchemaAttrOnBound(NotSchema), TypeSchema::NewTypeStruct { name: "SchemaAttrOnBound", field: &TypeSchema::U8 }))]
fn schemas<T: Schema>(#[case] (_ty, expected_schema): (T, StaticSchema)) {
    assert_eq!(T::SCHEMA, &expected_schema);
}
