#![expect(clippy::as_conversions, reason = "test file")]

mod common;

use self::common::*;
use ::{
    core::marker::PhantomData,
    schema::{Data, FieldSchema, Schema, StaticSchema, TypeSchema},
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
    Foo { toto: u16 },
}

#[derive(Serialize, Deserialize, Schema, PartialEq, Eq, Debug)]
enum SchemaAttrOnTupleVariant {
    #[schema(as(u8))]
    Foo(u16, bool),
}

#[derive(Serialize, Deserialize, Schema, PartialEq, Eq, Debug)]
enum SchemaAttrOnTupleVariantField {
    Foo(u16, #[schema(as(u8))] bool),
}

struct NotSchema;

#[derive(Schema)]
#[schema(bounds())]
struct SchemaAttrOnBound<T>(#[schema(as(u8))] T);

#[rstest::rstest]
#[case::renamed_struct((PhantomData::<Renamed>, TypeSchema::Struct{ data: Data::Unit { name: "TOTO" } }))]
#[case::renamed_variant((PhantomData::<RenamedVariant>, TypeSchema::Enum { name: "RenamedVariant", variants: &[&(0_u32, Data::Unit { name: "FOO" })] as &[&_] }))]
#[case::renamed_field((PhantomData::<RenamedField>, TypeSchema::Struct{ data: Data::Struct { name: "TOTO", fields: &[&FieldSchema { name: "FOO", ty: &TypeSchema::U8 }] as &[&_] } }))]
#[case::skipped_variant((PhantomData::<SkippedVariant>, TypeSchema::Enum { name: "SkippedVariant", variants: &[&(0_u32, Data::Unit { name: "Foo" }), &(2_u32, Data::Unit { name: "Baz" })] as &[&_] }))]
#[case::skipped_field((PhantomData::<SkippedField>, TypeSchema::Struct{ data: Data::Struct { name: "SkippedField", fields: &[&FieldSchema { name: "foo", ty: &TypeSchema::U8 }, &FieldSchema { name: "baz", ty: &TypeSchema::U8 }] as &[&_] } }))]
#[case::skipped_tuple_field_newtype((PhantomData::<SkippedTupleFieldMeansNewType>, TypeSchema::Struct{ data: Data::NewType { name: "SkippedTupleFieldMeansNewType", field: &TypeSchema::U8 } }))]
#[case::skipped_enum_tuple_field_newtype((PhantomData::<EnumSkippedTupleFieldMeansNewType>, TypeSchema::Enum { name: "EnumSkippedTupleFieldMeansNewType", variants: &[&(0_u32, Data::NewType { name: "Toto", field: &TypeSchema::U8 })] as &[&_] }))]
#[case::from_into((PhantomData::<FromIntoU8>, TypeSchema::U8))]
#[case::transparent((PhantomData::<Transparent>, TypeSchema::U8))]
#[case::schema_attr_item((PhantomData::<SchemaAttrOnItem>, TypeSchema::U8))]
#[case::schema_attr_field((PhantomData::<SchemaAttrOnField>, TypeSchema::Struct{ data: Data::Struct { name: "SchemaAttrOnField", fields: &[&FieldSchema { name: "foo", ty: &TypeSchema::U16 }] as &[&_] } }))]
#[case::schema_attr_variant((PhantomData::<SchemaAttrOnVariant>, TypeSchema::Enum { name: "SchemaAttrOnVariant", variants: &[&(0_u32, Data::NewType { name: "Foo", field: &TypeSchema::U8 })] as &[&_] }))]
#[case::schema_attr_tuple_variant((PhantomData::<SchemaAttrOnTupleVariant>, TypeSchema::Enum { name: "SchemaAttrOnTupleVariant", variants: &[&(0_u32, Data::NewType { name: "Foo", field: &TypeSchema::U8 })] as &[&_] }))]
#[case::schema_attr_tuple_variant_field((PhantomData::<SchemaAttrOnTupleVariantField>, TypeSchema::Enum { name: "SchemaAttrOnTupleVariantField", variants: &[&(0_u32, Data::Tuple { name: "Foo", fields: &[&TypeSchema::U16, &TypeSchema::U8] as &[&_] })] as &[&_] }))]
#[case::schema_attr_bound((PhantomData::<SchemaAttrOnBound::<NotSchema>>, TypeSchema::Struct{ data: Data::NewType { name: "SchemaAttrOnBound", field: &TypeSchema::U8 } }))]
fn schemas<T: Schema>(#[case] (_ty, expected_schema): (PhantomData<T>, StaticSchema)) {
    assert_eq!(T::SCHEMA, &expected_schema);
}
