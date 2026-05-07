#![expect(clippy::min_ident_chars, dead_code, reason = "shared test fixtures")]

pub mod protocols;

use ::{
    cecetype::*,
    serde::{Deserialize, Serialize},
};

#[derive(Serialize, Deserialize, Schema, PartialEq, Eq, Debug)]
pub struct BasicStruct {
    pub a: u32,
    pub b: bool,
}

#[derive(Serialize, Deserialize, Schema, PartialEq, Eq, Debug)]
pub struct UnitStruct;

#[derive(Serialize, Deserialize, Schema, PartialEq, Eq, Debug)]
pub struct NewTypeStruct(pub u8);

#[derive(Serialize, Deserialize, Schema, PartialEq, Eq, Debug)]
pub struct TupleStruct(pub u8, pub ());

#[derive(Serialize, Deserialize, Schema, PartialEq, Eq, Debug)]
pub enum BasicEnum {
    Unit,
    Tuple(u32, bool),
    Struct { x: u8, y: u8 },
    Nested { payload: NestedStruct },
}

#[derive(Serialize, Deserialize, Schema, PartialEq, Eq, Debug)]
pub struct NestedStruct {
    pub inner: BasicStruct,
    pub tuple: (u32, bool),
    pub array: [u8; 3],
}

#[derive(Serialize, Deserialize, Schema, PartialEq, Eq, Debug)]
pub struct GenericStruct<T>(pub T);

#[derive(Serialize, Deserialize, Schema, PartialEq, Eq, Debug)]
pub enum GenericEnum<T> {
    Toto(T),
}

#[derive(Serialize, Deserialize, Schema, PartialEq, Eq, Debug)]
#[serde(rename = "TOTO")]
pub struct Renamed;

#[derive(Serialize, Deserialize, Schema, PartialEq, Eq, Debug)]
pub enum RenamedVariant {
    #[serde(rename = "FOO")]
    Foo,
}

#[derive(Serialize, Deserialize, Schema, PartialEq, Eq, Debug)]
#[serde(rename = "TOTO")]
pub struct RenamedField {
    #[serde(rename = "FOO")]
    pub foo: u8,
}

#[derive(Serialize, Deserialize, Schema, PartialEq, Eq, Debug)]
pub enum SkippedVariant {
    Foo,
    #[serde(skip)]
    Bar,
    Baz,
}

#[derive(Serialize, Deserialize, Schema, PartialEq, Eq, Debug)]
pub struct SkippedField {
    pub foo: u8,
    #[serde(skip)]
    pub bar: u8,
    pub baz: u8,
}

#[derive(Serialize, Deserialize, Schema, PartialEq, Eq, Debug)]
pub struct SkippedTupleFieldMeansNewType(pub u8, #[serde(skip)] pub u16);

#[derive(Serialize, Deserialize, Schema, PartialEq, Eq, Debug)]
pub enum EnumSkippedTupleFieldMeansNewType {
    Toto(u8, #[serde(skip)] u16),
}

#[derive(Serialize, Deserialize, Schema, PartialEq, Eq, Debug, Clone)]
#[serde(from = "u8", into = "u8")]
pub struct FromIntoU8 {
    pub inner: u8,
}

impl From<u8> for FromIntoU8 {
    fn from(value: u8) -> Self {
        Self { inner: value }
    }
}

impl From<FromIntoU8> for u8 {
    #[inline]
    fn from(val: FromIntoU8) -> Self {
        val.inner
    }
}

#[derive(Serialize, Deserialize, Schema, PartialEq, Eq, Debug, Clone)]
#[serde(transparent)]
pub struct Transparent {
    pub foo: u8,
    #[serde(skip)]
    pub bar: u16,
}
