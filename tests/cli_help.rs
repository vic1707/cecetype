#![cfg(test)]
#![cfg(all(feature = "alloc", feature = "cli"))]
#![expect(
    clippy::unwrap_used,
    clippy::used_underscore_binding,
    clippy::panic,
    reason = "test file"
)]

extern crate alloc;

mod common;

use self::common::{
    BasicEnum, BasicStruct, GenericEnum, NestedOption, NestedStruct, NewTypeStruct, StructWithEnum,
    StructWithUnitField, Transparent, TupleStruct, UnitStruct, WithOption, WithOptionalStruct,
};
use ::{
    alloc::{
        collections::BTreeMap,
        string::{String, ToString as _},
        vec::Vec,
    },
    cecetype::{
        Schema,
        flavors::Owned,
        parse::cli::{
            Parser,
            help::{FoundRef, Help},
        },
        schema,
    },
    core::marker::PhantomData,
};

#[test]
fn primitive_request_and_response() {
    let help = Help::new(
        "ping",
        "Simple ping",
        <u32 as Schema>::SCHEMA,
        <bool as Schema>::SCHEMA,
    )
    .unwrap();
    let output = help.to_string();
    assert_eq!(
        output,
        "\
ping -- Simple ping

USAGE:
	ping <u32>

EXAMPLE:
	ping 0

RESPONSE:
	bool

"
    );
}

#[test]
fn unit_request() {
    let help = Help::new(
        "noop",
        "Does nothing",
        <() as Schema>::SCHEMA,
        <() as Schema>::SCHEMA,
    )
    .unwrap();
    let output = help.to_string();
    assert_eq!(
        output,
        "\
noop -- Does nothing

USAGE:
	noop <void>

EXAMPLE:
	noop 

RESPONSE:
	()

"
    );
}

#[test]
fn basic_struct() {
    let help = Help::new(
        "set_basic",
        "Sets basic configuration",
        BasicStruct::SCHEMA,
        <() as Schema>::SCHEMA,
    )
    .unwrap();
    let output = help.to_string();
    assert_eq!(
        output,
        "\
set_basic -- Sets basic configuration

USAGE:
	set_basic <`BasicStruct`>

TYPES:
	BasicStruct	<a: <u32>> <b: <bool>>

EXAMPLE:
	set_basic 0 true

RESPONSE:
	()

"
    );
}

#[test]
fn struct_with_option() {
    let help = Help::new(
        "update",
        "Updates a record",
        WithOption::SCHEMA,
        <bool as Schema>::SCHEMA,
    )
    .unwrap();
    let output = help.to_string();
    assert_eq!(
        output,
        "\
update -- Updates a record

USAGE:
	update <`WithOption`>

TYPES:
	WithOption	<required: <u32>> <optional: <u8>?> <also_required: <bool>>

EXAMPLE:
	update 0 some(0) true

RESPONSE:
	bool

"
    );
}

#[test]
fn basic_enum() {
    let help = Help::new(
        "action",
        "Performs an action",
        BasicEnum::SCHEMA,
        <u32 as Schema>::SCHEMA,
    )
    .unwrap();
    let output = help.to_string();
    assert_eq!(
        output,
        "\
action -- Performs an action

USAGE:
	action <`BasicEnum`>

TYPES:
	BasicEnum   
		Unit       	
		Tuple      	<u32> <bool>
		Struct     	<x: <u8>> <y: <u8>>
		Nested     	<payload: <`NestedStruct`>>

	NestedStruct	<inner: <`BasicStruct`>> <tuple: (<u32> <bool>)> <array: [<u8>, <u8>, <u8>]>

	BasicStruct 	<a: <u32>> <b: <bool>>

EXAMPLE:
	action Tuple 0 true

RESPONSE:
	u32

"
    );
}

#[test]
fn nested_struct() {
    let help = Help::new(
        "nested",
        "Works with nested data",
        NestedStruct::SCHEMA,
        <u8 as Schema>::SCHEMA,
    )
    .unwrap();
    let output = help.to_string();
    assert_eq!(
        output,
        "\
nested -- Works with nested data

USAGE:
	nested <`NestedStruct`>

TYPES:
	NestedStruct	<inner: <`BasicStruct`>> <tuple: (<u32> <bool>)> <array: [<u8>, <u8>, <u8>]>

	BasicStruct 	<a: <u32>> <b: <bool>>

EXAMPLE:
	nested (0 true) (0 true) [0, 0, 0]

RESPONSE:
	u8

"
    );
}

#[test]
fn unit_struct() {
    let help = Help::new(
        "unit",
        "Unit struct command",
        UnitStruct::SCHEMA,
        <() as Schema>::SCHEMA,
    )
    .unwrap();
    let output = help.to_string();
    assert_eq!(
        output,
        "\
unit -- Unit struct command

USAGE:
	unit <`UnitStruct`>

TYPES:
	UnitStruct	

EXAMPLE:
	unit 

RESPONSE:
	()

"
    );
}

#[test]
fn newtype_struct() {
    let help = Help::new(
        "wrap",
        "Newtype wrapper",
        NewTypeStruct::SCHEMA,
        <() as Schema>::SCHEMA,
    )
    .unwrap();
    let output = help.to_string();
    assert_eq!(
        output,
        "\
wrap -- Newtype wrapper

USAGE:
	wrap <`NewTypeStruct`>

TYPES:
	NewTypeStruct	<u8>

EXAMPLE:
	wrap 0

RESPONSE:
	()

"
    );
}

#[test]
fn tuple_struct() {
    let help = Help::new(
        "pair",
        "A tuple struct",
        TupleStruct::SCHEMA,
        <() as Schema>::SCHEMA,
    )
    .unwrap();
    let output = help.to_string();
    assert_eq!(
        output,
        "\
pair -- A tuple struct

USAGE:
	pair <`TupleStruct`>

TYPES:
	TupleStruct	<u8> <void>

EXAMPLE:
	pair 0

RESPONSE:
	()

"
    );
}

#[test]
fn option_request() {
    let help = Help::new(
        "maybe",
        "Optional input",
        <Option<u32> as Schema>::SCHEMA,
        <() as Schema>::SCHEMA,
    )
    .unwrap();
    let output = help.to_string();
    assert_eq!(
        output,
        "\
maybe -- Optional input

USAGE:
	maybe <u32>?

EXAMPLE:
	maybe some(0)

RESPONSE:
	()

"
    );
}

#[test]
fn slice_request() {
    let help = Help::new(
        "list",
        "Takes a list",
        <[u8] as Schema>::SCHEMA,
        <() as Schema>::SCHEMA,
    )
    .unwrap();
    let output = help.to_string();
    assert_eq!(
        output,
        "\
list -- Takes a list

USAGE:
	list [<u8>...]

EXAMPLE:
	list [0]

RESPONSE:
	()

"
    );
}

#[test]
fn array_request() {
    let help = Help::new(
        "triple",
        "Takes three values",
        <[u8; 3] as Schema>::SCHEMA,
        <() as Schema>::SCHEMA,
    )
    .unwrap();
    let output = help.to_string();
    assert_eq!(
        output,
        "\
triple -- Takes three values

USAGE:
	triple [<u8>, <u8>, <u8>]

EXAMPLE:
	triple [0, 0, 0]

RESPONSE:
	()

"
    );
}

#[test]
fn struct_with_optional_struct() {
    let help = Help::new(
        "opt_struct",
        "Struct with optional inner struct",
        WithOptionalStruct::SCHEMA,
        <() as Schema>::SCHEMA,
    )
    .unwrap();
    let output = help.to_string();
    assert_eq!(
        output,
        "\
opt_struct -- Struct with optional inner struct

USAGE:
	opt_struct <`WithOptionalStruct`>

TYPES:
	WithOptionalStruct	<inner: <`BasicStruct`>?> <tail: <u8>>

	BasicStruct       	<a: <u32>> <b: <bool>>

EXAMPLE:
	opt_struct some(0 true) 0

RESPONSE:
	()

"
    );
}

#[test]
fn struct_with_nested_option() {
    let help = Help::new(
        "nested_opt",
        "Struct with nested option",
        NestedOption::SCHEMA,
        <() as Schema>::SCHEMA,
    )
    .unwrap();
    let output = help.to_string();
    assert_eq!(
        output,
        "\
nested_opt -- Struct with nested option

USAGE:
	nested_opt <`NestedOption`>

TYPES:
	NestedOption	<inner: <u8>??> <required: <u8>>

EXAMPLE:
	nested_opt some(some(0)) 0

RESPONSE:
	()

"
    );
}

#[test]
fn struct_with_unit_field() {
    let help = Help::new(
        "with_unit",
        "Struct with a unit marker",
        StructWithUnitField::SCHEMA,
        <() as Schema>::SCHEMA,
    )
    .unwrap();
    let output = help.to_string();
    assert_eq!(
        output,
        "\
with_unit -- Struct with a unit marker

USAGE:
	with_unit <`StructWithUnitField`>

TYPES:
	StructWithUnitField	<marker: <void>> <value: <u8>>

EXAMPLE:
	with_unit 0

RESPONSE:
	()

"
    );
}

#[test]
fn optional_unit() {
    let help = Help::new(
        "opt_unit",
        "Optional unit",
        <Option<()> as Schema>::SCHEMA,
        <() as Schema>::SCHEMA,
    )
    .unwrap();
    let output = help.to_string();
    assert_eq!(
        output,
        "\
opt_unit -- Optional unit

USAGE:
	opt_unit <void>?

EXAMPLE:
	opt_unit some()

RESPONSE:
	()

"
    );
}

#[test]
fn array_of_unit() {
    let help = Help::new(
        "arr_unit",
        "Array of unit",
        <[(); 3] as Schema>::SCHEMA,
        <() as Schema>::SCHEMA,
    )
    .unwrap();
    let output = help.to_string();
    assert_eq!(
        output,
        "\
arr_unit -- Array of unit

USAGE:
	arr_unit [<void>, <void>, <void>]

EXAMPLE:
	arr_unit [, , ]

RESPONSE:
	()

"
    );
}

#[test]
fn tuple_with_unit() {
    let help = Help::new(
        "tup_unit",
        "Tuple with unit",
        <(u8, (), f32) as Schema>::SCHEMA,
        <() as Schema>::SCHEMA,
    )
    .unwrap();
    let output = help.to_string();
    assert_eq!(
        output,
        "\
tup_unit -- Tuple with unit

USAGE:
	tup_unit (<u8> <void> <f32>)

EXAMPLE:
	tup_unit 0 1.0

RESPONSE:
	()

"
    );
}

#[test]
fn struct_with_enum() {
    let help = Help::new(
        "struct_enum",
        "Struct with enum",
        StructWithEnum::SCHEMA,
        <() as Schema>::SCHEMA,
    )
    .unwrap();
    let output = help.to_string();
    assert_eq!(
        output,
        "\
struct_enum -- Struct with enum

USAGE:
	struct_enum <`StructWithEnum`>

TYPES:
	StructWithEnum	<tuple: (<u32> <bool>)> <inner: <`BasicEnum`>> <array: [<u8>, <u8>, <u8>]>

	BasicEnum     
		Unit         	
		Tuple        	<u32> <bool>
		Struct       	<x: <u8>> <y: <u8>>
		Nested       	<payload: <`NestedStruct`>>

	NestedStruct  	<inner: <`BasicStruct`>> <tuple: (<u32> <bool>)> <array: [<u8>, <u8>, <u8>]>

	BasicStruct   	<a: <u32>> <b: <bool>>

EXAMPLE:
	struct_enum (0 true) Tuple 0 true [0, 0, 0]

RESPONSE:
	()

"
    );
}

#[rstest::rstest]
#[case::u8(PhantomData::<u8>)]
#[case::u32(PhantomData::<u32>)]
#[case::i32(PhantomData::<i32>)]
#[case::f32(PhantomData::<f32>)]
#[case::bool(PhantomData::<bool>)]
#[case::string(PhantomData::<String>)]
#[case::option_u32(PhantomData::<Option<u32>>)]
#[case::array(PhantomData::<[u8; 3]>)]
#[case::vec(PhantomData::<Vec<u8>>)]
#[case::map(PhantomData::<BTreeMap<String, u32>>)]
#[case::tuple(PhantomData::<(bool, f32)>)]
#[case::unit(PhantomData::<()>)]
#[case::basic_struct(PhantomData::<BasicStruct>)]
#[case::with_option(PhantomData::<WithOption>)]
#[case::nested_struct(PhantomData::<NestedStruct>)]
#[case::basic_enum(PhantomData::<BasicEnum>)]
#[case::unit_struct(PhantomData::<UnitStruct>)]
#[case::newtype_struct(PhantomData::<NewTypeStruct>)]
#[case::tuple_struct(PhantomData::<TupleStruct>)]
#[case::with_optional_struct(PhantomData::<WithOptionalStruct>)]
#[case::struct_with_unit_field(PhantomData::<StructWithUnitField>)]
#[case::transparent(PhantomData::<Transparent>)]
#[case::generic_enum(PhantomData::<GenericEnum<BasicStruct>>)]
#[case::optional_unit(PhantomData::<Option<()>>)]
#[case::array_of_unit(PhantomData::<[(); 3]>)]
#[case::tuple_with_unit(PhantomData::<(u8, (), f32)>)]
#[case::struct_with_enum(PhantomData::<StructWithEnum>)]
fn example_roundtrip<T: Schema>(#[case] _ty: PhantomData<T>) {
    let help = Help::new("cmd", "", T::SCHEMA, <() as Schema>::SCHEMA).unwrap();
    let example = help.example().to_string();
    let mut source = Parser::new(example.trim());

    T::SCHEMA
        .build_value::<Owned, _>(&mut source)
        .unwrap_or_else(|err| {
            panic!(
                "
		Failed to parse example:
			{example}
		Error:
			{err:?}
   		"
            )
        });
}

#[rstest::rstest]
#[case::schema(PhantomData::<schema::Schema<Owned>>)]
#[case::value(PhantomData::<schema::Data<Owned>>)]
fn ref_is_rejected<T: Schema>(#[case] _ty: PhantomData<T>) {
    // as request
    let FoundRef(_) =
        Help::new("cmd", "", <T as Schema>::SCHEMA, <() as Schema>::SCHEMA).unwrap_err();
    // as response
    let FoundRef(_) =
        Help::new("cmd", "", <() as Schema>::SCHEMA, <T as Schema>::SCHEMA).unwrap_err();
}
