mod array;
mod r#enum;
mod map;
mod slice;
mod r#struct;
mod tuple;

pub use self::{
    array::ArrayVisitor,
    map::MapVisitor,
    r#enum::{EnumVisitor, OptionVisitor},
    r#struct::{NewTypeStructVisitor, StructVisitor, TupleStructVisitor, UnitStructVisitor},
    slice::SliceVisitor,
    tuple::TupleVisitor,
};
use crate::{SchemaFlavor, TypeSchema, Value, ValueBuilder};
use ::{
    core::{fmt, marker::PhantomData},
    serde::{
        de::{DeserializeSeed, SeqAccess, Visitor},
        Deserialize,
    },
};

/// Stack-linked-list for resolving `TypeSchema::Ref` names during deserialization.
///
/// Each entry maps a type name to its schema. When entering a named node
/// (Struct, Enum, etc.), a new `Resolver` is pushed onto the stack. When a
/// `Ref { name, kind }` is encountered, the chain is walked to find the schema.
///
/// Zero heap allocation — lives entirely on the call stack.
pub struct Resolver<'a, 's, SF: SchemaFlavor<'s>> {
    name: &'s str,
    schema: &'s TypeSchema<'s, SF>,
    parent: Option<&'a Self>,
}

impl<'a, 's, SF: SchemaFlavor<'s>> Resolver<'a, 's, SF> {
    pub const fn new(
        name: &'s str,
        schema: &'s TypeSchema<'s, SF>,
        parent: Option<&'a Self>,
    ) -> Self {
        Self {
            name,
            schema,
            parent,
        }
    }

    pub fn resolve(&self, name: &str) -> Option<&'s TypeSchema<'s, SF>> {
        if self.name == name {
            Some(self.schema)
        } else {
            self.parent?.resolve(name)
        }
    }
}

struct Seed<'a, 's, SF: SchemaFlavor<'s>, VF: ValueBuilder> {
    schema: &'s TypeSchema<'s, SF>,
    resolver: Option<&'a Resolver<'a, 's, SF>>,

    _p: PhantomData<VF>,
}

impl<'de, 's, SF, VF> DeserializeSeed<'de> for Seed<'_, 's, SF, VF>
where
    SF: SchemaFlavor<'s>,
    VF: ValueBuilder,
    VF::Str: Deserialize<'de>,
{
    type Value = Value<VF>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        self.schema
            .decode_value_with_resolver(deserializer, self.resolver)
    }
}

/// Visitor for deserializing a sequence where the element schema is a direct
/// `&TypeSchema` reference (from Ref resolution), rather than wrapped in `SF::Ptr`.
pub struct RefSliceVisitor<'a, 's, SF: SchemaFlavor<'s>, VF: ValueBuilder> {
    element: &'s TypeSchema<'s, SF>,
    resolver: Option<&'a Resolver<'a, 's, SF>>,
    _p: PhantomData<VF>,
}

impl<'a, 's, SF: SchemaFlavor<'s>, VF: ValueBuilder> RefSliceVisitor<'a, 's, SF, VF> {
    pub const fn new(
        element: &'s TypeSchema<'s, SF>,
        resolver: Option<&'a Resolver<'a, 's, SF>>,
    ) -> Self {
        Self {
            element,
            resolver,
            _p: PhantomData,
        }
    }
}

impl<'de, 's, SF, VF> Visitor<'de> for RefSliceVisitor<'_, 's, SF, VF>
where
    SF: SchemaFlavor<'s>,
    VF: ValueBuilder,
    VF::Str: Deserialize<'de>,
{
    type Value = Value<VF>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "Slice (ref)")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = seq
            .size_hint()
            .map_or_else(VF::list, VF::list_with_capacity);

        while let Some(el) = seq.next_element_seed(Seed {
            schema: self.element,
            resolver: self.resolver,
            _p: PhantomData,
        })? {
            VF::list_push(&mut values, el);
        }

        Ok(Value::Slice(values))
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
