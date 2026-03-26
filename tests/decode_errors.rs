#![cfg(test)]
#![expect(clippy::unwrap_used, clippy::panic, reason = "test file")]

mod common;

use self::common::*;
use ::schema::{Schema, Value};

fn assert_json_decode_error<T: Schema + ?Sized>(json: &str, expected: &str) {
    let err = decode_json_value::<T>(json).unwrap_err();

    assert_eq!(err.to_string(), expected);
}

#[test]
fn struct_missing_field() {
    assert_json_decode_error::<BasicStruct>(
        r#"{ "a": 42 }"#,
        "missing field `b` at line 1 column 11",
    );
}

#[test]
fn struct_unknown_field() {
    assert_json_decode_error::<BasicStruct>(
        r#"{ "a": 42, "b": true, "c": 1 }"#,
        "unknown field `c` at line 1 column 25",
    );
}

#[test]
fn struct_duplicate_field() {
    assert_json_decode_error::<BasicStruct>(
        r#"{ "a": 1, "a": 2, "b": true }"#,
        "duplicate field `a` at line 1 column 13",
    );
}

#[test]
fn tuple_too_short() {
    assert_json_decode_error::<(u32, bool)>(
        "[1]",
        "invalid length 1, expected tuple at line 1 column 3",
    );
}

#[test]
fn tuple_too_long() {
    assert_json_decode_error::<(u32, bool)>(
        "[1, true, false]",
        "invalid length 3, expected tuple at line 1 column 16",
    );
}

#[test]
fn tuple_wrong_type() {
    assert_json_decode_error::<(u32, bool)>(
        r#"[1, "14"]"#,
        r#"invalid type: string "14", expected a boolean at line 1 column 8"#,
    );
}

#[test]
fn enum_unknown_variant_name() {
    assert_json_decode_error::<BasicEnum>(r#""Unknown""#, "unknown variant: `Unknown`");
}

#[test]
fn enum_unknown_variant_index() {
    assert_json_decode_error::<BasicEnum>("{ 99: {} }", "unknown variant: `[id: 99]`");
}

#[test]
fn struct_field_order_irrelevant() {
    let Value::Struct { .. } =
        decode_json_value::<BasicStruct>(r#"{ "b": true, "a": 42 }"#).unwrap()
    else {
        panic!("expected struct")
    };
}
