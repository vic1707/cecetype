#![cfg(all(feature = "alloc", feature = "cli"))]
#![expect(
    clippy::literal_string_with_formatting_args,
    clippy::unwrap_used,
    clippy::used_underscore_binding,
    reason = "test file"
)]

extern crate alloc;

mod common;

use self::common::*;
use ::{
    alloc::{boxed::Box, collections::BTreeMap, string::String, vec},
    cecetype::{
        OwnedValue, Schema, StaticSchema,
        flavors::Owned,
        parse::cli::Parser,
        schema,
        value::{Data, Value},
    },
    core::marker::PhantomData,
};

#[rstest::rstest]
#[case::unit(PhantomData::<()>, "", Value::Unit)]
#[case::u32(PhantomData::<u32>, "42", Value::U32(42))]
#[case::bool_yes(PhantomData::<bool>, "yes", Value::Bool(true))]
#[case::tuple(PhantomData::<(bool, f32)>, "yes 1.00", Value::Tuple(vec![Value::Bool(true), Value::F32(1.00)]))]
#[case::quoted_string(PhantomData::<String>, "\"hello\\nworld\"", Value::Str("hello\\nworld".to_owned()))]
#[case::dquoted_string(PhantomData::<String>, "'hello\\nworld'", Value::Str("hello\\nworld".to_owned()))]
#[case::empty_string(PhantomData::<String>, "''", Value::Str(String::new()))]
#[case::option_none(PhantomData::<Option<u8>>, "none", Value::Option(None))]
#[case::option_null(PhantomData::<Option<u8>>, "null", Value::Option(None))]
#[case::option_nothing(PhantomData::<Option<u8>>, "", Value::Option(None))]
#[case::basic_struct_positional(PhantomData::<BasicStruct>, "42 true", Value::Struct { name: "BasicStruct".to_owned(), data: Data::Struct { fields: vec![("a".to_owned(), Value::U32(42)), ("b".to_owned(), Value::Bool(true))] } })]
#[case::with_option_explicit_none(PhantomData::<WithOption>, "42 none true", Value::Struct { name: "WithOption".to_owned(), data: Data::Struct { fields: vec![("required".to_owned(), Value::U32(42)), ("optional".to_owned(), Value::Option(None)), ("also_required".to_owned(), Value::Bool(true))] } })]
#[case::with_option_some(PhantomData::<WithOption>, "42 some(7) false", Value::Struct { name: "WithOption".to_owned(), data: Data::Struct { fields: vec![("required".to_owned(), Value::U32(42)), ("optional".to_owned(), Value::Option(Some(Box::new(Value::U8(7))))), ("also_required".to_owned(), Value::Bool(false))] } })]
#[case::with_option_null(PhantomData::<WithOption>, "42 null true", Value::Struct { name: "WithOption".to_owned(), data: Data::Struct { fields: vec![("required".to_owned(), Value::U32(42)), ("optional".to_owned(), Value::Option(None)), ("also_required".to_owned(), Value::Bool(true))] } })]
#[case::with_optional_struct_some(PhantomData::<WithOptionalStruct>, "some(42 true) 7", Value::Struct { name: "WithOptionalStruct".to_owned(), data: Data::Struct { fields: vec![("inner".to_owned(), Value::Option(Some(Box::new(Value::Struct { name: "BasicStruct".to_owned(), data: Data::Struct { fields: vec![("a".to_owned(), Value::U32(42)), ("b".to_owned(), Value::Bool(true))] } })))), ("tail".to_owned(), Value::U8(7))] } })] // todo: flattening?
#[case::struct_with_unit_field(PhantomData::<StructWithUnitField>, "7", Value::Struct { name: "StructWithUnitField".to_owned(), data: Data::Struct { fields: vec![("marker".to_owned(), Value::Unit), ("value".to_owned(), Value::U8(7))] } })]
#[case::newtype_struct(PhantomData::<NewTypeStruct>, "7", Value::Struct { name: "NewTypeStruct".to_owned(), data: Data::NewType { field: Box::new(Value::U8(7)) } })]
#[case::tuple_struct(PhantomData::<TupleStruct>, "7", Value::Struct { name: "TupleStruct".to_owned(), data: Data::Tuple { fields: vec![Value::U8(7), Value::Unit] } })]
#[case::transparent_struct(PhantomData::<Transparent>, "7", Value::U8(7))]
#[case::nested_struct(PhantomData::<NestedStruct>, "(1 false) (7 true) [1, 2, 3]", Value::Struct { name: "NestedStruct".to_owned(), data: Data::Struct { fields: vec![("inner".to_owned(), Value::Struct { name: "BasicStruct".to_owned(), data: Data::Struct { fields: vec![("a".to_owned(), Value::U32(1)), ("b".to_owned(), Value::Bool(false))] } }), ("tuple".to_owned(), Value::Tuple(vec![Value::U32(7), Value::Bool(true)])), ("array".to_owned(), Value::Array(vec![Value::U8(1), Value::U8(2), Value::U8(3)]))] } })]
#[case::enum_nested(PhantomData::<BasicEnum>, "Nested (1 false) (7 true) [1, 2, 3]", Value::Enum { enum_name: "BasicEnum".to_owned(), discriminant: 3, variant_name: "Nested".to_owned(), data: Data::Struct { fields: vec![("payload".to_owned(), Value::Struct { name: "NestedStruct".to_owned(), data: Data::Struct { fields: vec![("inner".to_owned(), Value::Struct { name: "BasicStruct".to_owned(), data: Data::Struct { fields: vec![("a".to_owned(), Value::U32(1)), ("b".to_owned(), Value::Bool(false))] } }), ("tuple".to_owned(), Value::Tuple(vec![Value::U32(7), Value::Bool(true)])), ("array".to_owned(), Value::Array(vec![Value::U8(1), Value::U8(2), Value::U8(3)]))] } })] } })]
#[case::enum_unit(PhantomData::<BasicEnum>, "unit", Value::Enum { enum_name: "BasicEnum".to_owned(), discriminant: 0, variant_name: "Unit".to_owned(), data: Data::Unit })]
#[case::enum_tuple_variant(PhantomData::<BasicEnum>, "Tuple 42 true", Value::Enum { enum_name: "BasicEnum".to_owned(), discriminant: 1, variant_name: "Tuple".to_owned(), data: Data::Tuple { fields: vec![Value::U32(42), Value::Bool(true)] } })]
#[case::enum_struct_variant(PhantomData::<BasicEnum>, "Struct 3 4", Value::Enum { enum_name: "BasicEnum".to_owned(), discriminant: 2, variant_name: "Struct".to_owned(), data: Data::Struct { fields: vec![("x".to_owned(), Value::U8(3)), ("y".to_owned(), Value::U8(4))] } })]
#[case::enum_nested_flattened(PhantomData::<GenericEnum<BasicStruct>>, "toto 42 true", Value::Enum { enum_name: "GenericEnum".to_owned(), discriminant: 0, variant_name: "Toto".to_owned(), data: Data::NewType { field: Box::new(Value::Struct { name: "BasicStruct".to_owned(), data: Data::Struct { fields: vec![("a".to_owned(), Value::U32(42)), ("b".to_owned(), Value::Bool(true))] } }) } })]
#[case::vec_list(PhantomData::<Vec<u8>>, "[1, 2, 3]", Value::Slice(vec![Value::U8(1), Value::U8(2), Value::U8(3)]))]
#[case::map(PhantomData::<BTreeMap<String, u32>>, "{'hello': 1, 'world': 2}", Value::Map(vec![(Value::Str("hello".to_owned()), Value::U32(1)), (Value::Str("world".to_owned()), Value::U32(2))]))]
#[case::map_num_key(PhantomData::<BTreeMap<u32, u32>>, "{0: 1, 12: 2}", Value::Map(vec![(Value::U32(0), Value::U32(1)), (Value::U32(12), Value::U32(2))]))]
#[case::map_trailing_comma(PhantomData::<BTreeMap<String, u32>>, "{'hello': 1, 'world': 2,}", Value::Map(vec![(Value::Str("hello".to_owned()), Value::U32(1)), (Value::Str("world".to_owned()), Value::U32(2))]))]
#[case::map_empty(PhantomData::<BTreeMap<String, u32>>, "{}", Value::Map(vec![]))]
#[case::array(PhantomData::<[u8; 3]>, "[1, 2, 3]", Value::Array(vec![Value::U8(1), Value::U8(2), Value::U8(3)]))]
fn cli_parser_produces_expected_value<T: Schema>(
    #[case] _ty: PhantomData<T>,
    #[case] input: &str,
    #[case] expected: OwnedValue,
) {
    let mut source = Parser::new(input);

    assert_eq!(
        T::SCHEMA.build_value::<Owned, _>(&mut source).unwrap(),
        expected
    );
}

struct DanglingRef;

impl Schema for DanglingRef {
    const SCHEMA: &'static StaticSchema = &schema::Schema::Ref {
        name: "NoSuchType",
        kind: schema::RefKind::Direct,
    };
}

#[rstest::rstest]
#[case::option_invalid_none(PhantomData::<Option<u32>>, "''", "parser error: at <root>: unexpected token '', expected 'some'")]
#[case::option_quoted_none(PhantomData::<Option<u32>>, "'none'", "parser error: at <root>: unexpected token 'none', expected 'some'")]
#[case::u32_invalid_digit(PhantomData::<u32>, "toto", "parser error: at <root>: parse error: invalid integer: invalid digit found in string")]
#[case::bool_invalid_value(PhantomData::<bool>, "maybe", "parser error: at <root>: parse error: invalid bool")]
#[case::missing_required_field(PhantomData::<BasicStruct>, "42", "parser error: at <root>.b: unexpected end of input")]
#[case::mismatched_types_in_struct(PhantomData::<BasicStruct>, "true 42", "parser error: at <root>.a: parse error: invalid integer: invalid digit found in string")]
#[case::unclosed_group(PhantomData::<WithOptionalStruct>, "some(42 true 7", "parser error: at <root>.inner: unexpected token '7', expected ')'")]
#[case::invalid_enum_variant(PhantomData::<BasicEnum>, "UnknownVariant", "parser error: at <root>: variant 'UnknownVariant' not found")]
#[case::enum_missing_payload(PhantomData::<BasicEnum>, "Struct 3", "parser error: at <root>.Struct.y: unexpected end of input")]
#[case::array_too_short(PhantomData::<[u8; 3]>, "[1 2]", "parser error: at <root>: unexpected token '2', expected ','")]
#[case::array_too_long(PhantomData::<[u8; 3]>, "[1 2 3 4]", "parser error: at <root>: unexpected token '2', expected ','")]
#[case::unclosed_array(PhantomData::<Vec<u8>>, "[1, 2, 3", "parser error: at <root>: unexpected end of input")]
#[case::map_malformed_key_number(PhantomData::<BTreeMap<u32, u32>>, "{hello: }", "parser error: at <root>: parse error: invalid integer: invalid digit found in string")]
#[case::map_malformed_key_string(PhantomData::<BTreeMap<String, u32>>, "{hello: }", "parser error: at <root>: expected quoted word, got bare")]
#[case::map_missing_value_key_string(PhantomData::<BTreeMap<String, u32>>, "{'hello': }", r#"parser error: at <root>{"hello"}.$: unexpected token '}'"#)]
#[case::map_missing_value_key_number(PhantomData::<BTreeMap<u32, u32>>, "{12: }", "parser error: at <root>{12}.$: unexpected token '}'")]
#[case::map_malformed_separator(PhantomData::<BTreeMap<String, u32>>, "{'hello'=1}", r#"parser error: at <root>{"hello"}.$: unexpected token '=', expected ':'"#)]
#[case::transparent_type_error(PhantomData::<Transparent>, "not_a_number", "parser error: at <root>: parse error: invalid integer: invalid digit found in string")]
#[case::nested_path_error(PhantomData::<GenericEnum<NestedStruct>>, "Toto (not_a_number true) 7 true [1 2 3]", "parser error: at <root>.Toto.inner.a: parse error: invalid integer: invalid digit found in string")]
#[case::mismatch_paren_braket(PhantomData::<Option<u8>>, "some(1]", "parser error: at <root>: unexpected token ']', expected ')'")]
#[case::mismatch_braket_brace(PhantomData::<[u8; 3]>, "[1 2 3}", "parser error: at <root>: unexpected token '2', expected ','")]
#[case::missing_group_paren(PhantomData::<NestedStruct>, "1 false 7 true [1, 2, 3]", "parser error: at <root>.inner: unexpected token '1', expected '('")]
#[case::unclosed_group_paren(PhantomData::<NestedStruct>, "(1 false (7 true) [1, 2, 3]", "parser error: at <root>.inner: unexpected token '(', expected ')'")]
#[case::trailing_word(PhantomData::<u32>, "42 extra", "parser error: at <root>: unexpected token 'extra'")]
#[case::trailing_paren(PhantomData::<u32>, "42)", "parser error: at <root>: unexpected token ')'")]
#[case::unresolved_ref(PhantomData::<DanglingRef>, "anything", "unresolved schema ref: 'NoSuchType'")] // no path as this isn't a parsing error
fn cli_parser_expects_error<T: Schema>(
    #[case] _ty: PhantomData<T>,
    #[case] input: &str,
    #[case] error_msg: &str,
) {
    let mut source = Parser::new(input);

    let err = T::SCHEMA.build_value::<Owned, _>(&mut source).unwrap_err();

    assert_eq!(format!("{err}"), error_msg);
}

#[cfg(feature = "miette")]
#[test]
fn cli_parser_error_exposes_miette_diagnostic_span() {
    use ::miette::Diagnostic as _;

    let mut source = Parser::new("some(42 maybe) 7");

    let err = WithOptionalStruct::SCHEMA
        .build_value::<Owned, _>(&mut source)
        .unwrap_err();

    assert_eq!(err.code().unwrap().to_string(), "cecetype::cli::parse_atom");
    assert!(err.source_code().is_some());

    let labels = err.labels().unwrap().collect::<alloc::vec::Vec<_>>();
    assert_eq!(labels[0].offset(), 8);
    assert_eq!(labels[0].len(), 5);
}

#[cfg(feature = "miette")]
#[test]
fn cli_parser_error_prints_miette_report() {
    let mut source = Parser::new("some(42 maybe) 7");

    let err = WithOptionalStruct::SCHEMA
        .build_value::<Owned, _>(&mut source)
        .unwrap_err();
    let mut rendered = String::new();
    ::miette::NarratableReportHandler::new()
        .render_report(&mut rendered, &err)
        .unwrap();

    assert_eq!(
        rendered,
        "\
parser error: at <root>.inner.b: parse error: invalid bool
    Diagnostic severity: error
Begin snippet starting at line 1, column 1

snippet line 1: some(42 maybe) 7
    label at line 1, columns 9 to 13: while parsing <root>.inner.b
diagnostic code: cecetype::cli::parse_atom
"
    );
}

#[cfg(feature = "miette")]
#[test]
fn cli_unresolved_ref_error_prints_miette_report() {
    let mut source = Parser::new("anything");

    let err = DanglingRef::SCHEMA
        .build_value::<Owned, _>(&mut source)
        .unwrap_err();
    let mut rendered = String::new();
    ::miette::NarratableReportHandler::new()
        .render_report(&mut rendered, &err)
        .unwrap();

    assert_eq!(
        rendered,
        "\
unresolved schema ref: 'NoSuchType'
    Diagnostic severity: error
diagnostic help: make sure referenced schemas are registered before parsing
diagnostic code: cecetype::parse::unresolved_schema_ref
"
    );
}

#[cfg(feature = "miette-fancy")]
#[test]
fn cli_parser_error_prints_graphical_miette_report() {
    use ::core::fmt;

    struct Render<'a, D> {
        handler: &'a dyn ::miette::ReportHandler,
        diagnostic: &'a D,
    }

    impl<D: ::miette::Diagnostic> fmt::Debug for Render<'_, D> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.handler.debug(self.diagnostic, f)
        }
    }

    let mut source = Parser::new("some(42 maybe) 7");

    let err = WithOptionalStruct::SCHEMA
        .build_value::<Owned, _>(&mut source)
        .unwrap_err();
    let handler = ::miette::MietteHandlerOpts::new()
        .force_graphical(true)
        .color(false)
        .unicode(false)
        .terminal_links(false)
        .width(80)
        .build();
    let rendered = format!(
        "{:?}",
        Render {
            handler: &handler,
            diagnostic: &err,
        }
    );

    assert_eq!(
        rendered,
        "\
cecetype::cli::parse_atom

  x parser error: at <root>.inner.b: parse error: invalid bool
   ,----
 1 | some(42 maybe) 7
   :         ^^|^^
   :           `-- while parsing <root>.inner.b
   `----
"
    );
}

#[cfg(feature = "miette-fancy")]
#[test]
fn cli_unresolved_ref_error_prints_graphical_miette_report() {
    use ::core::fmt;

    struct Render<'a, D> {
        handler: &'a dyn ::miette::ReportHandler,
        diagnostic: &'a D,
    }

    impl<D: ::miette::Diagnostic> fmt::Debug for Render<'_, D> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.handler.debug(self.diagnostic, f)
        }
    }

    let mut source = Parser::new("anything");

    let err = DanglingRef::SCHEMA
        .build_value::<Owned, _>(&mut source)
        .unwrap_err();
    let handler = ::miette::MietteHandlerOpts::new()
        .force_graphical(true)
        .color(false)
        .unicode(false)
        .terminal_links(false)
        .width(80)
        .build();
    let rendered = format!(
        "{:?}",
        Render {
            handler: &handler,
            diagnostic: &err,
        }
    );

    assert_eq!(
        rendered,
        "\
cecetype::parse::unresolved_schema_ref

  x unresolved schema ref: 'NoSuchType'
  help: make sure referenced schemas are registered before parsing
"
    );
}
