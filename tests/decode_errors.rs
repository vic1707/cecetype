#![cfg(test)]
#![expect(
    clippy::unwrap_used,
    clippy::used_underscore_binding,
    reason = "test file"
)]

mod common;

use self::common::*;
use ::cecetype::{Schema, flavors::Owned, value::Value};
use ::core::marker::PhantomData;

#[rstest::rstest]
#[case::missing_field(PhantomData::<BasicStruct>, r#"{ "a": 42 }"#, "missing field `b` in struct `BasicStruct` at line 1 column 11")]
#[case::unknown_field(PhantomData::<BasicStruct>, r#"{ "a": 42, "b": true, "c": 1 }"#, "unknown field `c` in struct `BasicStruct` at line 1 column 25")]
#[case::duplicate_field(PhantomData::<BasicStruct>, r#"{ "a": 1, "a": 2, "b": true }"#, "duplicate field `a` in struct `BasicStruct` at line 1 column 13")]
#[case::tuple_too_short(PhantomData::<(u32, bool)>, "[1]", "invalid length 1, expected tuple at line 1 column 3")]
#[case::tuple_too_long(PhantomData::<(u32, bool)>, "[1, true, false]", "invalid length 3, expected tuple at line 1 column 16")]
#[case::tuple_wrong_type(PhantomData::<(u32, bool)>, r#"[1, "14"]"#, r#"invalid type: string "14", expected a boolean at line 1 column 8"#)]
#[case::unknown_variant(PhantomData::<BasicEnum>, r#""Unknown""#, "unknown variant: `Unknown`")]
fn json_decode_errors<T: Schema + ?Sized>(
    #[case] _type: PhantomData<T>,
    #[case] input: &str,
    #[case] expected_msg: &str,
) {
    let err = T::SCHEMA
        .decode_value::<_, Owned>(&mut ::serde_json::Deserializer::from_str(input))
        .unwrap_err();

    assert_eq!(err.to_string(), expected_msg);
}

#[test]
fn struct_field_order_irrelevant() {
    let result = BasicStruct::SCHEMA
        .decode_value::<_, Owned>(&mut ::serde_json::Deserializer::from_str(
            r#"{ "b": true, "a": 42 }"#,
        ))
        .unwrap();

    assert!(matches!(result, Value::Struct { .. }));
}
