#![cfg(test)]
#![expect(
    clippy::unwrap_used,
    clippy::used_underscore_binding,
    reason = "test file"
)]

mod common;

use ::{
    core::marker::PhantomData,
    dimly::{
        Schema, StaticSchema,
        flavors::Owned,
        schema,
        value::{Data, Value},
    },
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
#[case::ref_self(PhantomData::<Recurse>, r#"{ "RefSelf": "Leaf" }"#, Value::Enum { name: "Recurse".to_owned(), discriminant: 1, data: Data::NewType { name: "RefSelf".to_owned(), field: Box::new(Value::Enum { name: "Recurse".to_owned(), discriminant: 0, data: Data::Unit { name: "Leaf".to_owned() } }) } })]
#[case::ref_list(PhantomData::<Recurse>, r#"{ "RefList": ["Leaf", { "RefSelf": "Leaf" }] }"#, Value::Enum { name: "Recurse".to_owned(), discriminant: 2, data: Data::NewType { name: "RefList".to_owned(), field: Box::new(Value::Slice(vec![Value::Enum { name: "Recurse".to_owned(), discriminant: 0, data: Data::Unit { name: "Leaf".to_owned() } }, Value::Enum { name: "Recurse".to_owned(), discriminant: 1, data: Data::NewType { name: "RefSelf".to_owned(), field: Box::new(Value::Enum { name: "Recurse".to_owned(), discriminant: 0, data: Data::Unit { name: "Leaf".to_owned() } }) } }])) } })]
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
