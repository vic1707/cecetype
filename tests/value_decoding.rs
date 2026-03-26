#![expect(
    clippy::unseparated_literal_suffix,
    clippy::as_conversions,
    clippy::unwrap_used,
    clippy::used_underscore_binding,
    clippy::non_ascii_literal,
    reason = "test file"
)]

mod common;

use self::common::*;
use ::{
    schema::{OwnedSchema, OwnedValue, Schema, Value},
    serde::Serialize,
};

#[rstest::rstest]
#[case::unit(((), Value::Unit))]
#[case::option_none((Option::<u8>::None, Value::Option(None)))]
#[case::option_some((Some(12u8), Value::Option(Some(Box::new(Value::U8(12))))))]
#[case::result_ok((Result::<u8, &str>::Ok(12), Value::EnumNewType { name: "Result".to_owned(), discriminant: 0, variant_name: "Ok".to_owned(), field: Box::new(Value::U8(12)) }))]
#[case::result_err((Result::<u8, &str>::Err("error"), Value::EnumNewType { name: "Result".to_owned(), discriminant: 1, variant_name: "Err".to_owned(), field: Box::new(Value::Str("error".to_owned())) }))]
#[case::u8_max((u8::MAX, Value::U8(255)))]
#[case::u32((123u32, Value::U32(123)))]
#[case::i64_min((i64::MIN, Value::I64(i64::MIN)))]
#[case::i64_max((i64::MAX, Value::I64(i64::MAX)))]
#[case::f32_inf((12.5f32, Value::F32(12.5)))]
#[case::bool((true, Value::Bool(true)))]
#[case::empty_string(("", Value::Str(String::new())))]
#[case::unicode(("é🚀", Value::Str("é🚀".to_owned())))]
#[case::string(("hello", Value::Str("hello".to_owned())))]
#[case::char(('x', Value::Char('x')))]
#[case::nested_tuple((((((12u8,),),),), Value::Tuple(vec![Value::Tuple(vec![Value::Tuple(vec![Value::Tuple(vec![Value::U8(12)])])])])))]
#[case::tuple(((1u32, false), Value::Tuple(vec![Value::U32(1), Value::Bool(false)])))]
#[case::empty_slice((&[] as &[u8], Value::Slice(vec![])))]
#[case::slice((&[1u8, 2, 3] as &[u8], Value::Slice(vec![Value::U8(1), Value::U8(2), Value::U8(3)])))]
#[case::empty_array(([] as [u8; 0], Value::Array(vec![])))]
#[case::nested_array(([[[8]]] as [[[u8; 1]; 1]; 1], Value::Array(vec![Value::Array(vec![Value::Array(vec![Value::U8(8)])])] )))]
#[case::array(([1u8, 2, 3], Value::Array(vec![Value::U8(1), Value::U8(2), Value::U8(3)])))]
#[case::unit_struct((UnitStruct, Value::UnitStruct { name: "UnitStruct".to_owned() }))]
#[case::newtype_struct((NewTypeStruct(16), Value::NewTypeStruct { name: "NewTypeStruct".to_owned(), field: Box::new(Value::U8(16)) }))]
#[case::tuple_struct((TupleStruct(16, ()), Value::TupleStruct { name: "TupleStruct".to_owned(), fields: vec![Value::U8(16), Value::Unit] }))]
#[case::generic_struct((GenericStruct(16u8), Value::NewTypeStruct { name: "GenericStruct".to_owned(), field: Box::new(Value::U8(16)) }))]
#[case::struct_((BasicStruct { a: 42, b: true }, Value::Struct { name: "BasicStruct".to_owned(), fields: vec![("a".to_owned(), Value::U32(42)), ("b".to_owned(), Value::Bool(true))] }))]
#[case::enum_unit((BasicEnum::Unit, Value::EnumUnit { name: "BasicEnum".to_owned(), discriminant: 0, variant_name: "Unit".to_owned() }))]
#[case::enum_tuple((BasicEnum::Tuple(10, false), Value::EnumTuple { name: "BasicEnum".to_owned(), discriminant: 1, variant_name: "Tuple".to_owned(), fields: vec![Value::U32(10), Value::Bool(false)] }))]
#[case::enum_struct((BasicEnum::Struct { x: 1, y: 2 }, Value::EnumStruct { name: "BasicEnum".to_owned(), discriminant: 2, variant_name: "Struct".to_owned(), fields: vec![("x".to_owned(), Value::U8(1)), ("y".to_owned(), Value::U8(2))] }))]
#[case::generic_enum((GenericEnum::Toto(12u8), Value::EnumNewType { name: "GenericEnum".to_owned(), discriminant: 0, variant_name: "Toto".to_owned(), field: Box::new(Value::U8(12)) }))]
#[case::nested_struct((NestedStruct { inner: BasicStruct { a: 1, b: false }, tuple: (7, true), array: [1, 2, 3] }, Value::Struct { name: "NestedStruct".to_owned(), fields: vec![("inner".to_owned(), Value::Struct { name: "BasicStruct".to_owned(), fields: vec![("a".to_owned(), Value::U32(1)), ("b".to_owned(), Value::Bool(false))] }), ("tuple".to_owned(), Value::Tuple(vec![Value::U32(7), Value::Bool(true)])), ("array".to_owned(), Value::Array(vec![Value::U8(1), Value::U8(2), Value::U8(3)]))] }))]
#[case::enum_nested((BasicEnum::Nested { payload: NestedStruct { inner: BasicStruct { a: 9, b: true }, tuple: (3, false), array: [4, 5, 6] } }, Value::EnumStruct { name: "BasicEnum".to_owned(), discriminant: 3, variant_name: "Nested".to_owned(), fields: vec![("payload".to_owned(), Value::Struct { name: "NestedStruct".to_owned(), fields: vec![("inner".to_owned(), Value::Struct { name: "BasicStruct".to_owned(), fields: vec![("a".to_owned(), Value::U32(9)), ("b".to_owned(), Value::Bool(true))] }), ("tuple".to_owned(), Value::Tuple(vec![Value::U32(3), Value::Bool(false)])), ("array".to_owned(), Value::Array(vec![Value::U8(4), Value::U8(5), Value::U8(6)]))] })] }))]
#[case::serde_from_into((FromIntoU8 { inner: 0 }, Value::U8(0)))]
#[case::transparent((Transparent { foo: 12, bar: 2 }, Value::U8(12)))]
fn value_decoding<F: protocols::Format, D: Serialize + Schema>(
    #[values(
        protocols::Json,
        protocols::Postcard,
        protocols::Yaml,
        protocols::Ron,
        protocols::SerdeCbor,
        protocols::MessagePack
    )]
    _protocol: F,
    #[case] (data, expected): (D, OwnedValue),
) {
    let encoded_schema = F::encode(D::SCHEMA).unwrap();
    let decoded_schema = F::decode::<OwnedSchema>(&encoded_schema).unwrap();

    assert_eq!(format!("{decoded_schema}"), format!("{}", D::SCHEMA));

    let wire = F::encode(&data).unwrap();
    let value_decoded = F::decode_value::<D>(&wire).unwrap();

    assert_eq!(value_decoded, expected);
}
