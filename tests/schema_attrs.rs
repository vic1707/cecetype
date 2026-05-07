#![expect(clippy::as_conversions, reason = "test file")]

mod common;

use self::common::*;
use ::{
    cecetype::{Schema, StaticSchema, schema},
    core::marker::PhantomData,
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
#[case::renamed_struct((PhantomData::<Renamed>, schema::Schema::Struct{ name: "TOTO", data: schema::Data::Unit }))]
#[case::renamed_variant((PhantomData::<RenamedVariant>, schema::Schema::Enum { name: "RenamedVariant", variants: &[&(0_u32, "FOO", schema::Data::Unit)] as &[&_] }))]
#[case::renamed_field((PhantomData::<RenamedField>, schema::Schema::Struct{ name: "TOTO", data: schema::Data::Struct { fields: &[&schema::FieldSchema { name: "FOO", ty: &schema::Schema::U8 }] as &[&_] } }))]
#[case::skipped_variant((PhantomData::<SkippedVariant>, schema::Schema::Enum { name: "SkippedVariant", variants: &[&(0_u32, "Foo", schema::Data::Unit), &(2_u32, "Baz", schema::Data::Unit)] as &[&_] }))]
#[case::skipped_field((PhantomData::<SkippedField>, schema::Schema::Struct{ name: "SkippedField", data: schema::Data::Struct { fields: &[&schema::FieldSchema { name: "foo", ty: &schema::Schema::U8 }, &schema::FieldSchema { name: "baz", ty: &schema::Schema::U8 }] as &[&_] } }))]
#[case::skipped_tuple_field_newtype((PhantomData::<SkippedTupleFieldMeansNewType>, schema::Schema::Struct{ name: "SkippedTupleFieldMeansNewType", data: schema::Data::NewType { field: &schema::Schema::U8 } }))]
#[case::skipped_enum_tuple_field_newtype((PhantomData::<EnumSkippedTupleFieldMeansNewType>, schema::Schema::Enum { name: "EnumSkippedTupleFieldMeansNewType", variants: &[&(0_u32, "Toto", schema::Data::NewType { field: &schema::Schema::U8 })] as &[&_] }))]
#[case::from_into((PhantomData::<FromIntoU8>, schema::Schema::U8))]
#[case::transparent((PhantomData::<Transparent>, schema::Schema::U8))]
#[case::schema_attr_item((PhantomData::<SchemaAttrOnItem>, schema::Schema::U8))]
#[case::schema_attr_field((PhantomData::<SchemaAttrOnField>, schema::Schema::Struct{ name: "SchemaAttrOnField", data: schema::Data::Struct { fields: &[&schema::FieldSchema { name: "foo", ty: &schema::Schema::U16 }] as &[&_] } }))]
#[case::schema_attr_variant((PhantomData::<SchemaAttrOnVariant>, schema::Schema::Enum { name: "SchemaAttrOnVariant", variants: &[&(0_u32, "Foo", schema::Data::NewType { field: &schema::Schema::U8 })] as &[&_] }))]
#[case::schema_attr_tuple_variant((PhantomData::<SchemaAttrOnTupleVariant>, schema::Schema::Enum { name: "SchemaAttrOnTupleVariant", variants: &[&(0_u32, "Foo", schema::Data::NewType { field: &schema::Schema::U8 })] as &[&_] }))]
#[case::schema_attr_tuple_variant_field((PhantomData::<SchemaAttrOnTupleVariantField>, schema::Schema::Enum { name: "SchemaAttrOnTupleVariantField", variants: &[&(0_u32, "Foo", schema::Data::Tuple { fields: &[&schema::Schema::U16, &schema::Schema::U8] as &[&_] })] as &[&_] }))]
#[case::schema_attr_bound((PhantomData::<SchemaAttrOnBound::<NotSchema>>, schema::Schema::Struct{ name: "SchemaAttrOnBound", data: schema::Data::NewType { field: &schema::Schema::U8 } }))]
fn schemas<T: Schema>(#[case] (_ty, expected_schema): (PhantomData<T>, StaticSchema)) {
    assert_eq!(T::SCHEMA, &expected_schema);
}
