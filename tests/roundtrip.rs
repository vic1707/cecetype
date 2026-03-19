use schema::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct MyStruct {
    a: u32,
    b: bool,
}

impl Schema for MyStruct {
    const SCHEMA: &'static StaticSchema = &TypeSchema::Struct(&StructSchema {
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
    });
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
enum MyEnum {
    Unit,
    Tuple(u32, bool),
    Struct { x: u8, y: u8 },
}

impl Schema for MyEnum {
    const SCHEMA: &'static StaticSchema = &TypeSchema::Enum(&EnumSchema {
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
    });
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Nested {
    inner: MyStruct,
    flag: bool,
}

impl Schema for Nested {
    const SCHEMA: &'static StaticSchema = &TypeSchema::Struct(&StructSchema {
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
    });
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
enum DeepEnum {
    A(MyStruct),
    B { nested: Nested },
}

impl Schema for DeepEnum {
    const SCHEMA: &'static StaticSchema = &TypeSchema::Enum(&EnumSchema {
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
                fields: &[
                    &FieldSchema {
                        name: "nested",
                        ty: Nested::SCHEMA,
                    },
                ] as &[&_],
            },
        ] as &[&_],
    });
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Complex {
    tuple: (u32, bool),
    array: [u8; 3],
}

impl Schema for Complex {
    const SCHEMA: &'static StaticSchema = &TypeSchema::Struct(&StructSchema {
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
    });
}

#[rstest::rstest(
    _protocol => [Json, Postcard],
    data_expected => [
        // --- primitives ---
        (123u32, Value::U32(123)),
        (true, Value::Bool(true)),

        // --- string / char ---
        ("hello", Value::Str("hello".to_owned())),
        ('x', Value::Char('x')),

        // --- tuple ---
        (
            (1u32, false),
            Value::Tuple(vec![
                Value::U32(1),
                Value::Bool(false),
            ])
        ),

        // --- array ---
        (
            [1u8, 2, 3],
            Value::Array(vec![
                Value::U8(1),
                Value::U8(2),
                Value::U8(3),
            ])
        ),
        // --- simple struct ---
        (
            MyStruct { a: 42, b: true },
            Value::Struct { 
                name: "MyStruct".to_owned(), 
                fields: vec![
                    ("a".to_owned(), Value::U32(42)),
                    ("b".to_owned(), Value::Bool(true)),
                ]
            }
        ),

        // --- enum tuple ---
        (
            MyEnum::Tuple(10, false),
            Value::Enum {
                name: "MyEnum".to_owned(),
                variant: VariantValue::Tuple {
                    name: "Tuple".to_owned(), 
                    fields: vec![Value::U32(10), Value::Bool(false)]
                }
            }
        ),

        // --- enum struct ---
        (
            MyEnum::Struct { x: 1, y: 2 },
            Value::Enum {
                name: "MyEnum".to_owned(),
                variant: VariantValue::Struct {
                    name: "Struct".to_owned(),
                    fields: vec![
                        ("x".to_owned(), Value::U8(1)),
                        ("y".to_owned(), Value::U8(2)),
                    ]
                }
            }
        ),

        // --- nested struct ---
        (
            Nested {
                inner: MyStruct { a: 1, b: false },
                flag: true,
            },
            Value::Struct {
                name: "Nested".to_owned(),
                fields: vec![
                    (
                        "inner".to_owned(),
                        Value::Struct {
                            name: "MyStruct".to_owned(),
                            fields: vec![
                                ("a".to_owned(), Value::U32(1)),
                                ("b".to_owned(), Value::Bool(false)),
                            ]
                        }
                    ),
                    ("flag".to_owned(), Value::Bool(true)),
                ]
            }
        ),

        // --- tuple + array ---
        (
            Complex {
                tuple: (7, true),
                array: [1, 2, 3],
            },
            Value::Struct {
                name: "Complex".to_owned(),
                fields: vec![
                    (
                        "tuple".to_owned(),
                        Value::Tuple(vec![
                            Value::U32(7),
                            Value::Bool(true),
                        ])
                    ),
                    (
                        "array".to_owned(),
                        Value::Array(vec![
                            Value::U8(1),
                            Value::U8(2),
                            Value::U8(3),
                        ])
                    ),
                ]
            }
        ),

        // --- deep enum ---
        (
            DeepEnum::B {
                nested: Nested {
                    inner: MyStruct { a: 9, b: true },
                    flag: false,
                }
            },
            Value::Enum {
                name: "DeepEnum".to_owned(),
                variant: VariantValue::Struct {
                    name: "B".to_owned(),
                    fields: vec![
                        (
                            "nested".to_owned(),
                            Value::Struct {
                                name: "Nested".to_owned(),
                                fields: vec![
                                    (
                                        "inner".to_owned(),
                                        Value::Struct {
                                            name: "MyStruct".to_owned(),
                                            fields: vec![
                                                ("a".to_owned(), Value::U32(9)),
                                                ("b".to_owned(), Value::Bool(true)),
                                            ]
                                        }
                                    ),
                                    ("flag".to_owned(), Value::Bool(false)),
                                ]
                            }
                        )
                    ]
                }
            }
        ),
    ],
)]
fn test_json<R: Roundtrip, D: Serialize + Schema>(
    _protocol: R,
    data_expected: (D, OwnedValue)
) {
    let (data, expected) = data_expected;
    assert!(R::roundtrip(&data) == expected)
}

trait Roundtrip {
    fn roundtrip<T: Serialize + Schema>(value: &'_ T) -> OwnedValue<'_>;
}

struct Json;

impl Roundtrip for Json {
    fn roundtrip<T: Serialize + Schema>(value: &'_ T) -> OwnedValue<'_> {
        let json = ::serde_json::to_string(value).unwrap();
        let mut de = ::serde_json::Deserializer::from_str(&json);

        T::SCHEMA.decode_value(&mut de).unwrap()
    }
}

struct Postcard;

impl Roundtrip for Postcard {
    fn roundtrip<T: Serialize + Schema>(value: &'_ T) -> OwnedValue<'_> {
        let bytes = ::postcard::to_vec::<_, 1024>(value).unwrap();
        let mut de = ::postcard::Deserializer::from_bytes(&bytes);

        T::SCHEMA.decode_value(&mut de).unwrap()
    }
}