#![expect(
    clippy::unseparated_literal_suffix,
    clippy::as_conversions,
    clippy::unwrap_used,
    clippy::used_underscore_binding,
    clippy::non_ascii_literal,
    reason = "test file"
)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod common;

use self::common::*;
#[cfg(feature = "alloc")]
use ::alloc::collections::{BTreeSet, BinaryHeap, LinkedList, VecDeque};
#[cfg(feature = "std")]
use ::std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::{Mutex, RwLock},
};
use ::{
    core::{
        cell::{Cell, RefCell},
        marker::PhantomData,
        num::{NonZeroI64, NonZeroU32, Saturating, Wrapping},
        time::Duration,
    },
    schema::{OwnedSchema, OwnedValue, Schema, Value, ValueData},
    serde::Serialize,
};

#[rstest::rstest]
#[case::unit(((), Value::Unit))]
#[case::option_none((Option::<u8>::None, Value::Option(None)))]
#[case::option_some((Some(12u8), Value::Option(Some(Box::new(Value::U8(12))))))]
#[case::result_ok((Result::<u8, &str>::Ok(12), Value::Enum { name: "Result".to_owned(), discriminant: 0, data: ValueData::NewType { name: "Ok".to_owned(), field: Box::new(Value::U8(12)) } }))]
#[case::result_err((Result::<u8, &str>::Err("error"), Value::Enum { name: "Result".to_owned(), discriminant: 1, data: ValueData::NewType { name: "Err".to_owned(), field: Box::new(Value::Str("error".to_owned())) } }))]
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
#[case::unit_struct((UnitStruct, Value::Struct{ data: ValueData::Unit { name: "UnitStruct".to_owned() } }))]
#[case::newtype_struct((NewTypeStruct(16), Value::Struct{ data: ValueData::NewType { name: "NewTypeStruct".to_owned(), field: Box::new(Value::U8(16)) } }))]
#[case::tuple_struct((TupleStruct(16, ()), Value::Struct{ data: ValueData::Tuple { name: "TupleStruct".to_owned(), fields: vec![Value::U8(16), Value::Unit] } }))]
#[case::generic_struct((GenericStruct(16u8), Value::Struct{ data: ValueData::NewType { name: "GenericStruct".to_owned(), field: Box::new(Value::U8(16)) } }))]
#[case::struct_((BasicStruct { a: 42, b: true }, Value::Struct{ data: ValueData::Struct { name: "BasicStruct".to_owned(), fields: vec![("a".to_owned(), Value::U32(42)), ("b".to_owned(), Value::Bool(true))] } }))]
#[case::enum_unit((BasicEnum::Unit, Value::Enum { name: "BasicEnum".to_owned(), discriminant: 0, data: ValueData::Unit { name: "Unit".to_owned() } }))]
#[case::enum_tuple((BasicEnum::Tuple(10, false), Value::Enum { name: "BasicEnum".to_owned(), discriminant: 1, data: ValueData::Tuple { name: "Tuple".to_owned(), fields: vec![Value::U32(10), Value::Bool(false)] } }))]
#[case::enum_struct((BasicEnum::Struct { x: 1, y: 2 }, Value::Enum { name: "BasicEnum".to_owned(), discriminant: 2, data: ValueData::Struct { name: "Struct".to_owned(), fields: vec![("x".to_owned(), Value::U8(1)), ("y".to_owned(), Value::U8(2))] } }))]
#[case::generic_enum((GenericEnum::Toto(12u8), Value::Enum { name: "GenericEnum".to_owned(), discriminant: 0, data: ValueData::NewType { name: "Toto".to_owned(), field: Box::new(Value::U8(12)) } }))]
#[case::nested_struct((NestedStruct { inner: BasicStruct { a: 1, b: false }, tuple: (7, true), array: [1, 2, 3] }, Value::Struct{ data: ValueData::Struct { name: "NestedStruct".to_owned(), fields: vec![("inner".to_owned(), Value::Struct{ data: ValueData::Struct { name: "BasicStruct".to_owned(), fields: vec![("a".to_owned(), Value::U32(1)), ("b".to_owned(), Value::Bool(false))] } }), ("tuple".to_owned(), Value::Tuple(vec![Value::U32(7), Value::Bool(true)])), ("array".to_owned(), Value::Array(vec![Value::U8(1), Value::U8(2), Value::U8(3)]))] } }))]
#[case::enum_nested((BasicEnum::Nested { payload: NestedStruct { inner: BasicStruct { a: 9, b: true }, tuple: (3, false), array: [4, 5, 6] } }, Value::Enum { name: "BasicEnum".to_owned(), discriminant: 3, data: ValueData::Struct { name: "Nested".to_owned(), fields: vec![("payload".to_owned(), Value::Struct{ data: ValueData::Struct { name: "NestedStruct".to_owned(), fields: vec![("inner".to_owned(), Value::Struct{ data: ValueData::Struct { name: "BasicStruct".to_owned(), fields: vec![("a".to_owned(), Value::U32(9)), ("b".to_owned(), Value::Bool(true))] } }), ("tuple".to_owned(), Value::Tuple(vec![Value::U32(3), Value::Bool(false)])), ("array".to_owned(), Value::Array(vec![Value::U8(4), Value::U8(5), Value::U8(6)]))] } })] } }))]
#[case::serde_from_into((FromIntoU8 { inner: 0 }, Value::U8(0)))]
#[case::transparent((Transparent { foo: 12, bar: 2 }, Value::U8(12)))]
#[case::nonzero_u32((NonZeroU32::new(42).unwrap(), Value::U32(42)))]
#[case::nonzero_i64((NonZeroI64::new(-100).unwrap(), Value::I64(-100)))]
#[case::wrapping_u32((Wrapping(255_u32), Value::U32(255)))]
#[case::saturating_i16((Saturating(-32000_i16), Value::I16(-32000)))]
#[case::cell_u8((Cell::new(7_u8), Value::U8(7)))]
#[case::refcell_bool((RefCell::new(true), Value::Bool(true)))]
#[case::duration((Duration::new(120, 500_000_000), Value::Struct{ data: ValueData::Struct { name: "Duration".to_owned(), fields: vec![("secs".to_owned(), Value::U64(120)), ("nanos".to_owned(), Value::U32(500_000_000))] } }))]
#[case::phantomdata((PhantomData::<()>, Value::Struct { data: ValueData::Unit { name: "PhantomData".to_owned() } }))]
#[cfg_attr(feature = "alloc", case::btreeset((BTreeSet::from([1_u32, 2, 3]), Value::Slice(vec![Value::U32(1), Value::U32(2), Value::U32(3)]))))]
#[cfg_attr(feature = "alloc", case::vecdeque((VecDeque::from([10_i32, 20_i32, 30_i32]), Value::Slice(vec![Value::I32(10), Value::I32(20), Value::I32(30)]))))]
#[cfg_attr(feature = "alloc", case::linkedlist(([1_u8, 2, 3].into_iter().collect::<LinkedList<_>>(), Value::Slice(vec![Value::U8(1), Value::U8(2), Value::U8(3)]))))]
#[cfg_attr(feature = "alloc", case::binaryheap((BinaryHeap::from(vec![42_u32]), Value::Slice(vec![Value::U32(42)]))))]
#[cfg_attr(feature = "std", case::pathbuf((PathBuf::from("/usr/local/bin"), Value::Str("/usr/local/bin".to_owned()))))]
#[cfg_attr(feature = "std", case::hashmap((HashMap::from([("key".to_owned(), 42_u32)]), Value::Map(vec![(Value::Str("key".to_owned()), Value::U32(42))]))))]
#[cfg_attr(feature = "std", case::hashset((HashSet::from([99_u32]), Value::Slice(vec![Value::U32(99)]))))]
#[cfg_attr(feature = "std", case::mutex((Mutex::new("hello".to_owned()), Value::Str("hello".to_owned()))))]
#[cfg_attr(feature = "std", case::rwlock((RwLock::new("hello".to_owned()), Value::Str("hello".to_owned()))))]
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
