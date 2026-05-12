#![cfg(test)]
#![cfg(feature = "alloc")]
#![expect(
    clippy::unwrap_used,
    clippy::used_underscore_binding,
    reason = "test file"
)]

mod common;

use ::{
    cecetype::{
        Schema, StaticSchema,
        flavors::Owned,
        schema,
        value::{Data, Value},
    },
    core::marker::PhantomData,
};

#[derive(serde::Serialize, serde::Deserialize, Schema, PartialEq, Eq, Debug, Clone)]
enum Recurse {
    Leaf,
    #[schema(ref(Recurse))]
    RefSelf(Box<Self>),
    #[schema(ref(Recurse, list))]
    RefList(Vec<Self>),
}

#[rstest::rstest]
#[case::ref_self(PhantomData::<Recurse>, r#"{ "RefSelf": "Leaf" }"#, Value::Enum { enum_name: "Recurse".to_owned(), discriminant: 1, variant_name: "RefSelf".to_owned(), data: Data::NewType { field: Box::new(Value::Enum { enum_name: "Recurse".to_owned(), discriminant: 0, variant_name: "Leaf".to_owned(), data: Data::Unit }) } })]
#[case::ref_list(PhantomData::<Recurse>, r#"{ "RefList": ["Leaf", { "RefSelf": "Leaf" }] }"#, Value::Enum { enum_name: "Recurse".to_owned(), discriminant: 2, variant_name: "RefList".to_owned(), data: Data::NewType { field: Box::new(Value::Slice(vec![Value::Enum { enum_name: "Recurse".to_owned(), discriminant: 0, variant_name: "Leaf".to_owned(), data: Data::Unit }, Value::Enum { enum_name: "Recurse".to_owned(), discriminant: 1, variant_name: "RefSelf".to_owned(), data: Data::NewType { field: Box::new(Value::Enum { enum_name: "Recurse".to_owned(), discriminant: 0, variant_name: "Leaf".to_owned(), data: Data::Unit }) } }])) } })]
fn test_recursive_decoding<T: Schema + ?Sized>(
    #[case] _type: PhantomData<T>,
    #[case] json: &str,
    #[case] expected: Value<Owned>,
) {
    let value = T::SCHEMA
        .decode_value::<_, Owned>(&mut serde_json::Deserializer::from_str(json))
        .unwrap();

    assert_eq!(value, expected);
}

#[test]
fn unresolved_ref_error() {
    let ref_schema: &StaticSchema = &schema::Schema::Ref {
        name: "MissingType",
        kind: schema::RefKind::Direct,
    };

    let err = ref_schema
        .decode_value::<_, Owned>(&mut serde_json::Deserializer::from_str("42"))
        .unwrap_err();

    assert!(
        err.to_string()
            .contains("unresolved schema ref `MissingType`")
    );
}
