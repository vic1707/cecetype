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
    core::fmt,
    serde::{de::DeserializeOwned, Serialize},
};

#[rstest::rstest]
#[case::unit(())]
#[case::option_none(Option::<u8>::None)]
#[case::option_some(Some(12u8))]
#[case::result_ok(Result::<u8, String>::Ok(12))]
#[case::result_err(Result::<u8, String>::Err("error".to_owned()))]
#[case::u8_max(u8::MAX)]
#[case::u32(123u32)]
#[case::i64_min(i64::MIN)]
#[case::i64_max(i64::MAX)]
#[case::f32_inf(12.5f32)]
#[case::bool(true)]
#[case::empty_string(String::new())]
#[case::unicode("é🚀".to_owned())]
#[case::string("hello".to_owned())]
#[case::char('x')]
#[case::nested_tuple((((((12u8,),),),),))]
#[case::tuple((1u32, false))]
#[case::empty_array([] as [u8; 0])]
#[case::nested_array([[[8]]] as [[[u8; 1]; 1]; 1])]
#[case::array([1u8, 2, 3])]
#[case::unit_struct(UnitStruct)]
#[case::newtype_struct(NewTypeStruct(16))]
#[case::tuple_struct(TupleStruct(16, ()))]
#[case::generic_struct(GenericStruct(16u8))]
#[case::struct_(BasicStruct { a: 42, b: true })]
#[case::enum_unit(BasicEnum::Unit)]
#[case::enum_tuple(BasicEnum::Tuple(10, false))]
#[case::enum_struct(BasicEnum::Struct { x: 1, y: 2 })]
#[case::generic_enum(GenericEnum::Toto(12u8))]
#[case::nested_struct(NestedStruct { inner: BasicStruct { a: 1, b: false }, tuple: (7, true), array: [1, 2, 3] })]
#[case::enum_nested(BasicEnum::Nested { payload: NestedStruct { inner: BasicStruct { a: 9, b: true }, tuple: (3, false), array: [4, 5, 6] } })]
#[case::serde_from_into(FromIntoU8 { inner: 0 })]
#[case::transparent(Transparent { foo: 12, bar: 0 })]
fn roundtrip<
    F: protocols::Format,
    D: Serialize + ::schema::Schema + DeserializeOwned + PartialEq + fmt::Debug,
>(
    #[values(
        protocols::Json,
        protocols::Postcard,
        protocols::Yaml,
        protocols::Ron,
        protocols::SerdeCbor,
        protocols::MessagePack
    )]
    _protocol: F,
    #[case] data: D,
) {
    let wire = F::encode(&data).unwrap();
    let value_decoded = F::decode_value::<D>(&wire).unwrap();

    let wire2 = F::encode(&value_decoded).unwrap();
    let decoded_data = F::decode::<D>(&wire2).unwrap();

    assert_eq!(wire, wire2);
    assert_eq!(decoded_data, data);
}
