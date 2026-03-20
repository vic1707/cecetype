// TODO: Add tests with options and results
use schema::*;
use ::{
    core::{error, fmt},
    serde::{Deserialize, Serialize},
};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct MyStruct {
    a: u32,
    b: bool,
}

impl Schema for MyStruct {
    const SCHEMA: &'static StaticSchema = &TypeSchema::Struct {
        name: "MyStruct",
        fields: &[
            &FieldSchema {
                name: "a",
                ty: &TypeSchema::U32,
            },
            &FieldSchema {
                name: "b",
                ty: &TypeSchema::Bool,
            },
        ] as &[&_],
    };
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct MyUnitStruct;

impl Schema for MyUnitStruct {
    const SCHEMA: &'static StaticSchema = &TypeSchema::UnitStruct {
        name: "MyUnitStruct",
    };
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct MyNewTypeStruct(u8);

impl Schema for MyNewTypeStruct {
    const SCHEMA: &'static StaticSchema = &TypeSchema::NewTypeStruct {
        name: "MyNewTypeStruct",
        field: &TypeSchema::U8,
    };
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct MyTupleStruct(u8, ());

impl Schema for MyTupleStruct {
    const SCHEMA: &'static StaticSchema = &TypeSchema::TupleStruct {
        name: "MyTupleStruct",
        fields: &[&TypeSchema::U8, &TypeSchema::Unit],
    };
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
enum MyEnum {
    Unit,
    Tuple(u32, bool),
    Struct { x: u8, y: u8 },
}

impl Schema for MyEnum {
    const SCHEMA: &'static StaticSchema = &TypeSchema::Enum {
        name: "MyEnum",
        variants: &[
            &VariantSchema::Unit {
                name: "Unit",
                discriminant: 0,
            },
            &VariantSchema::Tuple {
                name: "Tuple",
                discriminant: 1,
                fields: &[&TypeSchema::U32, &TypeSchema::Bool] as &[&_],
            },
            &VariantSchema::Struct {
                name: "Struct",
                discriminant: 2,
                fields: &[
                    &FieldSchema {
                        name: "x",
                        ty: &TypeSchema::U8,
                    },
                    &FieldSchema {
                        name: "y",
                        ty: &TypeSchema::U8,
                    },
                ] as &[&_],
            },
        ] as &[&_],
    };
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Nested {
    inner: MyStruct,
    flag: bool,
}

impl Schema for Nested {
    const SCHEMA: &'static StaticSchema = &TypeSchema::Struct {
        name: "Nested",
        fields: &[
            &FieldSchema {
                name: "inner",
                ty: MyStruct::SCHEMA,
            },
            &FieldSchema {
                name: "flag",
                ty: &TypeSchema::Bool,
            },
        ] as &[&_],
    };
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
enum DeepEnum {
    A(MyStruct),
    B { nested: Nested },
}

impl Schema for DeepEnum {
    const SCHEMA: &'static StaticSchema = &TypeSchema::Enum {
        name: "DeepEnum",
        variants: &[
            &VariantSchema::Tuple {
                name: "A",
                discriminant: 0,
                fields: &[MyStruct::SCHEMA] as &[&_],
            },
            &VariantSchema::Struct {
                name: "B",
                discriminant: 1,
                fields: &[&FieldSchema {
                    name: "nested",
                    ty: Nested::SCHEMA,
                }] as &[&_],
            },
        ] as &[&_],
    };
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Complex {
    tuple: (u32, bool),
    array: [u8; 3],
}

impl Schema for Complex {
    const SCHEMA: &'static StaticSchema = &TypeSchema::Struct {
        name: "Complex",
        fields: &[
            &FieldSchema {
                name: "tuple",
                ty: <(u32, bool)>::SCHEMA,
            },
            &FieldSchema {
                name: "array",
                ty: &TypeSchema::Array {
                    element: &TypeSchema::U8,
                    len: 3,
                },
            },
        ] as &[&_],
    };
}

#[rstest::rstest]
#[case::unit(((), Value::Unit))]
#[case::u8_max((u8::MAX, Value::U8(255)))]
#[case::u32((123u32, Value::U32(123)))]
#[case::i64_min((i64::MIN, Value::I64(i64::MIN)))]
#[case::i64_max((i64::MAX, Value::I64(i64::MAX)))]
#[case::f32_inf((12.5f32, Value::F32(12.5)))]
#[case::bool((true, Value::Bool(true)))]
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
#[case::struct_((MyStruct { a: 42, b: true }, Value::Struct { name: "MyStruct".to_owned(), fields: vec![("a".to_owned(), Value::U32(42)), ("b".to_owned(), Value::Bool(true))] }))]
#[case::enum_unit((MyEnum::Unit, Value::Enum { name: "MyEnum".to_owned(), variant: VariantValue::Unit { name: "Unit".to_owned() } }))]
#[case::enum_tuple((MyEnum::Tuple(10, false), Value::Enum { name: "MyEnum".to_owned(), variant: VariantValue::Tuple { name: "Tuple".to_owned(), fields: vec![Value::U32(10), Value::Bool(false)] } }))]
#[case::enum_struct((MyEnum::Struct { x: 1, y: 2 }, Value::Enum { name: "MyEnum".to_owned(), variant: VariantValue::Struct { name: "Struct".to_owned(), fields: vec![("x".to_owned(), Value::U8(1)), ("y".to_owned(), Value::U8(2))] } }))]
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
