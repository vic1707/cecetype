mod array;
mod r#enum;
mod map;
mod slice;
mod r#struct;
mod tuple;

pub use self::{
    array::ArrayVisitor,
    r#enum::{EnumVisitor, OptionVisitor},
    map::MapVisitor,
    slice::SliceVisitor,
    r#struct::{NewTypeStructVisitor, StructVisitor, TupleStructVisitor, UnitStructVisitor},
    tuple::TupleVisitor,
};
use crate::{
    flavors::{SchemaFlavor, ValueBuilder},
    schema::Schema,
    value::Value,
};
use ::{
    core::marker::PhantomData,
    serde::{Deserialize, de::DeserializeSeed},
};

/// Stack-linked-list for resolving `Schema::Ref` names during deserialization.
///
/// Each entry maps a type name to its schema. When entering a named node
/// (Struct, Enum, etc.), a new `Resolver` is pushed onto the stack. When a
/// `Ref { name, kind }` is encountered, the chain is walked to find the schema.
pub struct Resolver<'a, 's, SF: SchemaFlavor<'s>> {
    name: &'s str,
    schema: &'s Schema<'s, SF>,
    parent: Option<&'a Self>,
}

impl<'a, 's, SF: SchemaFlavor<'s>> Resolver<'a, 's, SF> {
    pub const fn new(name: &'s str, schema: &'s Schema<'s, SF>, parent: Option<&'a Self>) -> Self {
        Self {
            name,
            schema,
            parent,
        }
    }

    pub fn resolve(&self, name: &str) -> Option<&'s Schema<'s, SF>> {
        if self.name == name {
            Some(self.schema)
        } else {
            self.parent?.resolve(name)
        }
    }
}

struct Seed<'a, 's, SF: SchemaFlavor<'s>, VB: ValueBuilder> {
    schema: &'s Schema<'s, SF>,
    resolver: Option<&'a Resolver<'a, 's, SF>>,

    _p: PhantomData<VB>,
}

impl<'de, 's, SF, VB> DeserializeSeed<'de> for Seed<'_, 's, SF, VB>
where
    SF: SchemaFlavor<'s>,
    VB: ValueBuilder,
    VB::Str: Deserialize<'de>,
{
    type Value = Value<VB>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        self.schema
            .decode_value_with_resolver(deserializer, self.resolver)
    }
}
/// Returns a slice of `len` empty `&'static str` values without allocating.
///
/// # Overview
/// Fabricates a `&[&'static str]` by reinterpreting a pointer to a single `&'static str`
/// as if it were an array of length `len`. Only safe if the slice is used solely
/// to read `.len()`.
///
/// # Safety
/// This is **undefined behavior** according to Rust:
/// - [`::core::slice::from_raw_parts`] requires `len` valid, contiguous elements.
/// - Here only one element exists.
/// - Accessing any index > 0 is an out-of-bounds read.
///
/// # Safer alternative
/// Use a real backing array:
/// ```rust
/// static EMPTY: &str = "";
/// static BACKING: [&str; 512] = [EMPTY; _];
///
/// fn names(len: usize) -> &'static [&'static str] {
///     &BACKING[..len]
/// }
/// ```
/// This greatly increases memory usage but is fully sound.
#[doc(hidden)]
#[expect(clippy::undocumented_unsafe_blocks, reason = "docs above")]
pub const fn names(len: usize) -> &'static [&'static str] {
    use ::core::slice;
    static EMPTY: &str = "";

    unsafe { slice::from_raw_parts(&raw const EMPTY, len) }
}
