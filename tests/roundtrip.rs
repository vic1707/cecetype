use ::{
    core::{error, fmt},
    serde::Serialize,
};
use schema::*;

#[derive(Serialize, Schema)]
struct MyStruct {
    a: u32,
    b: bool,
}

#[derive(Serialize, Schema)]
struct MyUnitStruct;

#[derive(Serialize, Schema)]
struct MyNewTypeStruct(u8);

#[derive(Serialize, Schema)]
struct MyTupleStruct(u8, ());

#[derive(Serialize, Schema)]
enum MyEnum {
    Unit,
    Tuple(u32, bool),
    Struct { x: u8, y: u8 },
}

#[derive(Serialize, Schema)]
struct Nested {
    inner: MyStruct,
    flag: bool,
}

#[derive(Serialize, Schema)]
enum DeepEnum {
    A(MyStruct),
    B { nested: Nested },
}

#[derive(Serialize, Schema)]
struct Complex {
    tuple: (u32, bool),
    array: [u8; 3],
}

#[derive(Serialize, Schema)]
struct GenericStruct<T>(T);

#[derive(Serialize, Schema)]
enum GenericEnum<T> {
    Toto(T),
}

#[rstest::rstest]
#[case::unit(((), Value::Unit))]
#[case::option_none((Option::<u8>::None, Value::Option(None)))]
#[case::option_some((Some(12u8), Value::Option(Some(Box::new(Value::U8(12))))))]
#[case::result_ok((Result::<u8, &str>::Ok(12), Value::Enum { name: "Result".to_owned(), variant: VariantValue::NewType { name: "Ok".to_owned(), field: Box::new(Value::U8(12)) } }))]
#[case::result_err((Result::<u8, &str>::Err("error"), Value::Enum { name: "Result".to_owned(), variant: VariantValue::NewType { name: "Err".to_owned(), field: Box::new(Value::Str("error".to_owned())) } }))]
#[case::u8_max((u8::MAX, Value::U8(255)))]
#[case::u32((123u32, Value::U32(123)))]
#[case::i64_min((i64::MIN, Value::I64(i64::MIN)))]
#[case::i64_max((i64::MAX, Value::I64(i64::MAX)))]
#[case::f32_inf((12.5f32, Value::F32(12.5)))]
#[case::bool((true, Value::Bool(true)))]
#[case::empty_string(("", Value::Str("".to_owned())))]
#[case::unicode(("é🚀", Value::Str("é🚀".to_owned())))]
#[case::string(("hello", Value::Str("hello".to_owned())))]
#[case::char(('x', Value::Char('x')))]
#[case::nested_tuple(( (((((12u8,),),),),), Value::Tuple(vec![Value::Tuple(vec![Value::Tuple(vec![Value::Tuple(vec![Value::Tuple(vec![Value::U8(12)])])])])]) ))]
#[case::tuple(((1u32, false), Value::Tuple(vec![Value::U32(1), Value::Bool(false)])))]
#[case::empty_slice((&[] as &[u8], Value::Slice(vec![])))]
#[case::slice((&[1u8, 2, 3] as &[u8], Value::Slice(vec![Value::U8(1), Value::U8(2), Value::U8(3)])))]
#[case::empty_array(([] as [u8; 0], Value::Array(vec![])))]
#[case::nested_array(( [[[8]]] as [[[u8; 1]; 1]; 1], Value::Array(vec![Value::Array(vec![Value::Array(vec![Value::U8(8)])])]) ))]
#[case::array(([1u8, 2, 3], Value::Array(vec![Value::U8(1), Value::U8(2), Value::U8(3)])))]
#[case::unit_struct((MyUnitStruct, Value::UnitStruct { name: "MyUnitStruct".to_owned() }))]
#[case::newtype_struct((MyNewTypeStruct(16), Value::NewTypeStruct { name: "MyNewTypeStruct".to_owned(), field: Box::new(Value::U8(16)) }))]
#[case::tuple_struct((MyTupleStruct(16, ()), Value::TupleStruct { name: "MyTupleStruct".to_owned(), fields: vec![Value::U8(16), Value::Unit] }))]
#[case::generic_struct((GenericStruct(16u8), Value::NewTypeStruct { name: "GenericStruct".to_owned(), field: Box::new(Value::U8(16)) }))]
#[case::struct_((MyStruct { a: 42, b: true }, Value::Struct { name: "MyStruct".to_owned(), fields: vec![("a".to_owned(), Value::U32(42)), ("b".to_owned(), Value::Bool(true))] }))]
#[case::enum_unit((MyEnum::Unit, Value::Enum { name: "MyEnum".to_owned(), variant: VariantValue::Unit { name: "Unit".to_owned() } }))]
#[case::enum_tuple((MyEnum::Tuple(10, false), Value::Enum { name: "MyEnum".to_owned(), variant: VariantValue::Tuple { name: "Tuple".to_owned(), fields: vec![Value::U32(10), Value::Bool(false)] } }))]
#[case::enum_struct((MyEnum::Struct { x: 1, y: 2 }, Value::Enum { name: "MyEnum".to_owned(), variant: VariantValue::Struct { name: "Struct".to_owned(), fields: vec![("x".to_owned(), Value::U8(1)), ("y".to_owned(), Value::U8(2))] } }))]
#[case::generic_enum((GenericEnum::Toto(12u8), Value::Enum { name: "GenericEnum".to_owned(), variant: VariantValue::NewType { name: "Toto".to_owned(), field: Box::new(Value::U8(12)) } }))]
#[case::nested((Nested { inner: MyStruct { a: 1, b: false }, flag: true }, Value::Struct { name: "Nested".to_owned(), fields: vec![("inner".to_owned(), Value::Struct { name: "MyStruct".to_owned(), fields: vec![("a".to_owned(), Value::U32(1)), ("b".to_owned(), Value::Bool(false))] }), ("flag".to_owned(), Value::Bool(true))] }))]
#[case::complex((Complex { tuple: (7, true), array: [1, 2, 3] }, Value::Struct { name: "Complex".to_owned(), fields: vec![("tuple".to_owned(), Value::Tuple(vec![Value::U32(7), Value::Bool(true)])), ("array".to_owned(), Value::Array(vec![Value::U8(1),Value::U8(2),Value::U8(3)]))] }))]
#[case::deep_enum((DeepEnum::B { nested: Nested { inner: MyStruct { a: 9, b: true }, flag: false } }, Value::Enum { name: "DeepEnum".to_owned(), variant: VariantValue::Struct { name: "B".to_owned(), fields: vec![("nested".to_owned(), Value::Struct { name: "Nested".to_owned(), fields: vec![("inner".to_owned(), Value::Struct { name: "MyStruct".to_owned(), fields: vec![("a".to_owned(), Value::U32(9)), ("b".to_owned(), Value::Bool(true))] }), ("flag".to_owned(), Value::Bool(false))] })]} }))]
fn roundtrip<R: Roundtrip, D: Serialize + Schema>(
    #[values(Json, Postcard, Yaml)] _protocol: R,
    #[case] (data, expected): (D, OwnedValue),
) {
    match R::roundtrip(&data) {
        Ok(decoded) => assert_eq!(decoded, expected),
        Err(err) => panic!("Couldn't decode payload: {err}"),
    }
}

#[test]
fn struct_missing_field() {
    let json = r#"{ "a": 42 }"#;

    let mut de = serde_json::Deserializer::from_str(json);
    let err = MyStruct::SCHEMA
        .decode_value::<_, Owned>(&mut de)
        .unwrap_err();

    assert_eq!(err.to_string(), "missing field `b` at line 1 column 11");
}

#[test]
fn struct_unknown_field() {
    let json = r#"{ "a": 42, "b": true, "c": 1 }"#;

    let mut de = serde_json::Deserializer::from_str(json);
    let err = MyStruct::SCHEMA
        .decode_value::<_, Owned>(&mut de)
        .unwrap_err();

    assert_eq!(err.to_string(), "unknown field `c` at line 1 column 25");
}

#[test]
fn struct_duplicate_field() {
    let json = r#"{ "a": 1, "a": 2, "b": true }"#;

    let mut de = serde_json::Deserializer::from_str(json);
    let err = MyStruct::SCHEMA
        .decode_value::<_, Owned>(&mut de)
        .unwrap_err();

    assert_eq!(err.to_string(), "duplicate field `a` at line 1 column 13");
}

#[test]
fn tuple_too_short() {
    let json = r#"[1]"#;

    let mut de = serde_json::Deserializer::from_str(json);
    let err = <(u32, bool)>::SCHEMA
        .decode_value::<_, Owned>(&mut de)
        .unwrap_err();

    assert_eq!(
        err.to_string(),
        "invalid length 1, expected tuple at line 1 column 3"
    );
}

#[test]
fn tuple_too_long() {
    let json = r#"[1, true, false]"#;

    let mut de = serde_json::Deserializer::from_str(json);
    let err = <(u32, bool)>::SCHEMA
        .decode_value::<_, Owned>(&mut de)
        .unwrap_err();

    assert_eq!(
        err.to_string(),
        "invalid length 3, expected tuple at line 1 column 16"
    );
}

#[test]
fn tuple_wrong_type() {
    let json = r#"[1, "14"]"#;

    let mut de = serde_json::Deserializer::from_str(json);
    let err = <(u32, bool)>::SCHEMA
        .decode_value::<_, Owned>(&mut de)
        .unwrap_err();

    assert_eq!(
        err.to_string(),
        r#"invalid type: string "14", expected a boolean at line 1 column 8"#
    );
}

#[test]
fn enum_unknown_variant_name() {
    let json = r#""Unknown""#;

    let mut de = serde_json::Deserializer::from_str(json);
    let err = MyEnum::SCHEMA
        .decode_value::<_, Owned>(&mut de)
        .unwrap_err();

    assert_eq!(err.to_string(), "unknown variant: `Unknown`");
}

#[test]
fn enum_unknown_variant_index() {
    let json = r#"{ 99: {} }"#;

    let mut de = serde_json::Deserializer::from_str(json);
    let err = MyEnum::SCHEMA
        .decode_value::<_, Owned>(&mut de)
        .unwrap_err();

    assert_eq!(err.to_string(), "unknown variant: `[id: 99]`");
}

#[test]
fn slice_of_structs() {
    let data = vec![MyStruct { a: 1, b: true }, MyStruct { a: 2, b: false }];

    let json = serde_json::to_string(&data).unwrap();
    let mut de = serde_json::Deserializer::from_str(&json);

    let schema = <[MyStruct]>::SCHEMA;
    let value = schema.decode_value::<_, Owned>(&mut de).unwrap();

    match value {
        Value::Slice(v) => assert_eq!(v.len(), 2),
        _ => panic!("expected slice"),
    }
}

#[test]
fn array_of_enum() {
    let data = [MyEnum::Unit, MyEnum::Unit];

    let result = Json::roundtrip(&data).unwrap();

    match result {
        Value::Array(v) => assert_eq!(v.len(), 2),
        _ => panic!("expected array"),
    }
}

#[test]
fn struct_field_order_irrelevant() {
    let json = r#"{ "b": true, "a": 42 }"#;

    let mut de = serde_json::Deserializer::from_str(json);
    let value = MyStruct::SCHEMA.decode_value::<_, Owned>(&mut de).unwrap();

    match value {
        Value::Struct { fields, .. } => {
            assert_eq!(fields.len(), 2);
        }
        _ => panic!("expected struct"),
    }
}

trait Roundtrip {
    type Error: fmt::Display + error::Error;

    fn roundtrip<T: Serialize + Schema>(value: &'_ T) -> Result<OwnedValue<'_>, Self::Error>;
}

struct Json;

impl Roundtrip for Json {
    type Error = ::serde_json::Error;

    fn roundtrip<T: Serialize + Schema>(value: &'_ T) -> Result<OwnedValue<'_>, Self::Error> {
        let encoded_schema =
            ::serde_json::to_string(T::SCHEMA).expect("Schema serialization failed");
        let decoded_schema = ::serde_json::from_str::<OwnedSchema>(&encoded_schema)
            .expect("Schema deserialization failed");

        let json = ::serde_json::to_string(value).expect("Value serialization failed");
        dbg! { &json };

        let mut de = ::serde_json::Deserializer::from_str(&json);
        decoded_schema.decode_value(&mut de)
    }
}

struct Postcard;

impl Roundtrip for Postcard {
    type Error = ::postcard::Error;

    fn roundtrip<T: Serialize + Schema>(value: &'_ T) -> Result<OwnedValue<'_>, Self::Error> {
        let encoded_schema =
            ::postcard::to_vec::<_, 1024>(T::SCHEMA).expect("Schema serialization failed");
        let decoded_schema = ::postcard::from_bytes::<OwnedSchema>(&encoded_schema)
            .expect("Schema deserialization failed");

        let bytes = ::postcard::to_vec::<_, 1024>(value).expect("Value serialization failed");
        dbg! { &bytes };

        let mut de = ::postcard::Deserializer::from_bytes(&bytes);
        decoded_schema.decode_value(&mut de)
    }
}

struct Yaml;

impl Roundtrip for Yaml {
    type Error = ::yaml_serde::Error;

    fn roundtrip<T: Serialize + Schema>(value: &'_ T) -> Result<OwnedValue<'_>, Self::Error> {
        let encoded_schema =
            ::yaml_serde::to_string(T::SCHEMA).expect("Schema serialization failed");
        let decoded_schema = ::yaml_serde::from_str::<OwnedSchema>(&encoded_schema)
            .expect("Schema deserialization failed");

        let yaml = ::yaml_serde::to_string(value).expect("Value serialization failed");
        dbg! { &yaml };

        let de = ::yaml_serde::Deserializer::from_str(&yaml);
        decoded_schema.decode_value(de)
    }
}
