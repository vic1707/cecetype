#![cfg(test)]
#![expect(
    clippy::unwrap_used,
    clippy::used_underscore_binding,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "test file"
)]

mod common;

use self::common::protocols;
use ::{
    schema::{Owned, Schema, Value},
    serde::{Deserialize, Serialize},
};

// ── Recursive tree: struct with Vec<Box<Node>> children ──────────────────────

#[derive(Serialize, Deserialize, Schema, PartialEq, Eq, Debug)]
struct TreeNode {
    label: u32,
    #[schema(ref(TreeNode, list))]
    children: Vec<Self>,
}

#[test]
fn tree_leaf_json() {
    let node = TreeNode {
        label: 1,
        children: vec![],
    };

    let wire = ::serde_json::to_string(&node).unwrap();
    let value = TreeNode::SCHEMA
        .decode_value::<_, Owned>(&mut ::serde_json::Deserializer::from_str(&wire))
        .unwrap();

    assert_eq!(
        value,
        Value::Struct {
            name: "TreeNode".to_owned(),
            fields: vec![
                ("label".to_owned(), Value::U32(1)),
                ("children".to_owned(), Value::Slice(vec![])),
            ],
        }
    );
}

#[test]
fn tree_nested_json() {
    let tree = TreeNode {
        label: 10,
        children: vec![
            TreeNode {
                label: 20,
                children: vec![],
            },
            TreeNode {
                label: 30,
                children: vec![TreeNode {
                    label: 40,
                    children: vec![],
                }],
            },
        ],
    };

    let wire = ::serde_json::to_string(&tree).unwrap();
    let value = TreeNode::SCHEMA
        .decode_value::<_, Owned>(&mut ::serde_json::Deserializer::from_str(&wire))
        .unwrap();

    let leaf = |label| Value::Struct {
        name: "TreeNode".to_owned(),
        fields: vec![
            ("label".to_owned(), Value::U32(label)),
            ("children".to_owned(), Value::Slice(vec![])),
        ],
    };

    assert_eq!(
        value,
        Value::Struct {
            name: "TreeNode".to_owned(),
            fields: vec![
                ("label".to_owned(), Value::U32(10)),
                (
                    "children".to_owned(),
                    Value::Slice(vec![
                        leaf(20),
                        Value::Struct {
                            name: "TreeNode".to_owned(),
                            fields: vec![
                                ("label".to_owned(), Value::U32(30)),
                                ("children".to_owned(), Value::Slice(vec![leaf(40)])),
                            ],
                        },
                    ])
                ),
            ],
        }
    );
}

// ── Recursive enum: self-referential expression AST ──────────────────────────

#[derive(Serialize, Deserialize, Schema, PartialEq, Eq, Debug)]
enum Expr {
    Lit(i32),
    Add(
        #[schema(ref(Expr))] Box<Self>,
        #[schema(ref(Expr))] Box<Self>,
    ),
    Neg(#[schema(ref(Expr))] Box<Self>),
}

#[test]
fn expr_literal_json() {
    let expr = Expr::Lit(42);

    let wire = ::serde_json::to_string(&expr).unwrap();
    let value = Expr::SCHEMA
        .decode_value::<_, Owned>(&mut ::serde_json::Deserializer::from_str(&wire))
        .unwrap();

    assert_eq!(
        value,
        Value::EnumNewType {
            name: "Expr".to_owned(),
            discriminant: 0,
            variant_name: "Lit".to_owned(),
            field: Box::new(Value::I32(42)),
        }
    );
}

#[test]
fn expr_nested_json() {
    // Add(Neg(Lit(1)), Lit(2))
    let expr = Expr::Add(
        Box::new(Expr::Neg(Box::new(Expr::Lit(1)))),
        Box::new(Expr::Lit(2)),
    );

    let wire = ::serde_json::to_string(&expr).unwrap();
    let value = Expr::SCHEMA
        .decode_value::<_, Owned>(&mut ::serde_json::Deserializer::from_str(&wire))
        .unwrap();

    assert_eq!(
        value,
        Value::EnumTuple {
            name: "Expr".to_owned(),
            discriminant: 1,
            variant_name: "Add".to_owned(),
            fields: vec![
                Value::EnumNewType {
                    name: "Expr".to_owned(),
                    discriminant: 2,
                    variant_name: "Neg".to_owned(),
                    field: Box::new(Value::EnumNewType {
                        name: "Expr".to_owned(),
                        discriminant: 0,
                        variant_name: "Lit".to_owned(),
                        field: Box::new(Value::I32(1)),
                    }),
                },
                Value::EnumNewType {
                    name: "Expr".to_owned(),
                    discriminant: 0,
                    variant_name: "Lit".to_owned(),
                    field: Box::new(Value::I32(2)),
                },
            ],
        }
    );
}

// ── Error case: unresolved Ref ───────────────────────────────────────────────

#[test]
fn unresolved_ref_error() {
    // Manually construct a schema with a Ref that won't be in the resolver.
    // We use TreeNode's children field (which has a Ref to "TreeNode").
    // If we deserialize against a raw Ref schema with no surrounding named node,
    // the resolver will be None and we should get an error.

    use schema::{RefKind, Static, TypeSchema};

    let ref_schema: &TypeSchema<Static> = &TypeSchema::Ref {
        name: "Nonexistent",
        kind: RefKind::Direct,
    };

    let result =
        ref_schema.decode_value::<_, Owned>(&mut ::serde_json::Deserializer::from_str("42"));

    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("unresolved schema ref `Nonexistent`"),
        "unexpected error: {err}"
    );
}

// ── Multi-protocol roundtrip for recursive struct ────────────────────────────

#[rstest::rstest]
fn tree_roundtrip_multiprotocol<F: protocols::Format>(
    #[values(
        protocols::Json,
        protocols::Postcard,
        protocols::Yaml,
        protocols::Ron,
        protocols::SerdeCbor,
        protocols::MessagePack
    )]
    _protocol: F,
) {
    let tree = TreeNode {
        label: 1,
        children: vec![
            TreeNode {
                label: 2,
                children: vec![],
            },
            TreeNode {
                label: 3,
                children: vec![TreeNode {
                    label: 4,
                    children: vec![],
                }],
            },
        ],
    };

    let wire = F::encode(&tree).unwrap();
    let value = F::decode_value::<TreeNode>(&wire).unwrap();

    // Verify the structure is correct — check the root and first-level children
    let Value::Struct { name, fields } = &value else {
        panic!("expected struct, got {value:?}");
    };
    assert_eq!(name, "TreeNode");
    assert_eq!(fields.len(), 2);

    let (label_name, label_val) = &fields[0];
    assert_eq!(label_name, "label");
    assert_eq!(*label_val, Value::U32(1));

    let (children_name, children_val) = &fields[1];
    assert_eq!(children_name, "children");
    let Value::Slice(children) = children_val else {
        panic!("expected slice for children");
    };
    assert_eq!(children.len(), 2);
}

#[derive(Serialize, Deserialize, Schema, PartialEq, Eq, Debug)]
enum Chain {
    End(u32),
    #[schema(ref(Chain))]
    Link(Box<Self>),
}

#[test]
fn chain_base_json() {
    let chain = Chain::End(42);

    let wire = ::serde_json::to_string(&chain).unwrap();
    let value = Chain::SCHEMA
        .decode_value::<_, Owned>(&mut ::serde_json::Deserializer::from_str(&wire))
        .unwrap();

    assert_eq!(
        value,
        Value::EnumNewType {
            name: "Chain".to_owned(),
            discriminant: 0,
            variant_name: "End".to_owned(),
            field: Box::new(Value::U32(42)),
        }
    );
}

#[test]
fn chain_nested_json() {
    let chain = Chain::Link(Box::new(Chain::Link(Box::new(Chain::End(7)))));

    let wire = ::serde_json::to_string(&chain).unwrap();
    let value = Chain::SCHEMA
        .decode_value::<_, Owned>(&mut ::serde_json::Deserializer::from_str(&wire))
        .unwrap();

    assert_eq!(
        value,
        Value::EnumNewType {
            name: "Chain".to_owned(),
            discriminant: 1,
            variant_name: "Link".to_owned(),
            field: Box::new(Value::EnumNewType {
                name: "Chain".to_owned(),
                discriminant: 1,
                variant_name: "Link".to_owned(),
                field: Box::new(Value::EnumNewType {
                    name: "Chain".to_owned(),
                    discriminant: 0,
                    variant_name: "End".to_owned(),
                    field: Box::new(Value::U32(7)),
                }),
            }),
        }
    );
}

#[derive(Serialize, Deserialize, Schema, PartialEq, Eq, Debug)]
enum TreeEnum {
    Leaf(u32),
    #[schema(ref(TreeEnum, list))]
    Branch(Vec<Self>),
}

#[test]
fn tree_enum_leaf_json() {
    let tree = TreeEnum::Leaf(5);

    let wire = ::serde_json::to_string(&tree).unwrap();
    let value = TreeEnum::SCHEMA
        .decode_value::<_, Owned>(&mut ::serde_json::Deserializer::from_str(&wire))
        .unwrap();

    assert_eq!(
        value,
        Value::EnumNewType {
            name: "TreeEnum".to_owned(),
            discriminant: 0,
            variant_name: "Leaf".to_owned(),
            field: Box::new(Value::U32(5)),
        }
    );
}

#[test]
fn tree_enum_nested_json() {
    let tree = TreeEnum::Branch(vec![
        TreeEnum::Leaf(1),
        TreeEnum::Branch(vec![TreeEnum::Leaf(2), TreeEnum::Leaf(3)]),
    ]);

    let wire = ::serde_json::to_string(&tree).unwrap();
    let value = TreeEnum::SCHEMA
        .decode_value::<_, Owned>(&mut ::serde_json::Deserializer::from_str(&wire))
        .unwrap();

    assert_eq!(
        value,
        Value::EnumNewType {
            name: "TreeEnum".to_owned(),
            discriminant: 1,
            variant_name: "Branch".to_owned(),
            field: Box::new(Value::Slice(vec![
                Value::EnumNewType {
                    name: "TreeEnum".to_owned(),
                    discriminant: 0,
                    variant_name: "Leaf".to_owned(),
                    field: Box::new(Value::U32(1)),
                },
                Value::EnumNewType {
                    name: "TreeEnum".to_owned(),
                    discriminant: 1,
                    variant_name: "Branch".to_owned(),
                    field: Box::new(Value::Slice(vec![
                        Value::EnumNewType {
                            name: "TreeEnum".to_owned(),
                            discriminant: 0,
                            variant_name: "Leaf".to_owned(),
                            field: Box::new(Value::U32(2)),
                        },
                        Value::EnumNewType {
                            name: "TreeEnum".to_owned(),
                            discriminant: 0,
                            variant_name: "Leaf".to_owned(),
                            field: Box::new(Value::U32(3)),
                        },
                    ])),
                },
            ])),
        }
    );
}

#[rstest::rstest]
fn chain_roundtrip_multiprotocol<F: protocols::Format>(
    #[values(
        protocols::Json,
        protocols::Postcard,
        // YAML doesn't support serializing nested enums
        protocols::Ron,
        protocols::SerdeCbor,
        protocols::MessagePack
    )]
    _protocol: F,
) {
    let chain = Chain::Link(Box::new(Chain::Link(Box::new(Chain::End(99)))));

    let wire = F::encode(&chain).unwrap();
    let value = F::decode_value::<Chain>(&wire).unwrap();

    let Value::EnumNewType {
        name,
        variant_name,
        field,
        ..
    } = &value
    else {
        panic!("expected EnumNewType, got {value:?}");
    };
    assert_eq!(name, "Chain");
    assert_eq!(variant_name, "Link");

    // Second level
    let Value::EnumNewType {
        name: inner_name,
        variant_name: inner_variant,
        field: inner_field,
        ..
    } = field.as_ref()
    else {
        panic!("expected inner EnumNewType, got {field:?}");
    };
    assert_eq!(inner_name, "Chain");
    assert_eq!(inner_variant, "Link");

    // Third level
    let Value::EnumNewType {
        variant_name: leaf_variant,
        field: leaf_field,
        ..
    } = inner_field.as_ref()
    else {
        panic!("expected leaf EnumNewType, got {inner_field:?}");
    };
    assert_eq!(leaf_variant, "End");
    assert_eq!(**leaf_field, Value::U32(99));
}

// ── Multi-protocol roundtrip for recursive enum ──────────────────────────────

#[rstest::rstest]
fn expr_roundtrip_multiprotocol<F: protocols::Format>(
    #[values(
        protocols::Json,
        protocols::Postcard,
        // YAML doesn't support serializing nested enums
        protocols::Ron,
        protocols::SerdeCbor,
        protocols::MessagePack
    )]
    _protocol: F,
) {
    // Neg(Lit(99))
    let expr = Expr::Neg(Box::new(Expr::Lit(99)));

    let wire = F::encode(&expr).unwrap();
    let value = F::decode_value::<Expr>(&wire).unwrap();

    let Value::EnumNewType {
        name,
        variant_name,
        field,
        ..
    } = &value
    else {
        panic!("expected EnumNewType, got {value:?}");
    };
    assert_eq!(name, "Expr");
    assert_eq!(variant_name, "Neg");

    let Value::EnumNewType {
        name: inner_name,
        variant_name: inner_variant,
        field: inner_field,
        ..
    } = field.as_ref()
    else {
        panic!("expected inner EnumNewType, got {field:?}");
    };
    assert_eq!(inner_name, "Expr");
    assert_eq!(inner_variant, "Lit");
    assert_eq!(**inner_field, Value::I32(99));
}
