mod array;
mod r#enum;
mod slice;
mod r#struct;
mod tuple;
pub use self::{
    array::ArrayVisitor,
    r#enum::{EnumVisitor, OptionVisitor},
    r#struct::{NewTypeStructVisitor, StructVisitor, TupleStructVisitor, UnitStructVisitor},
    slice::SliceVisitor,
    tuple::TupleVisitor,
};

use crate::{SchemaFlavor, TypeSchema, Value, ValueBuilder};
use ::{
    core::marker::PhantomData,
    serde::{de::DeserializeSeed, Deserialize},
};

struct Seed<'s, SF: SchemaFlavor<'s>, VF: ValueBuilder> {
    schema: &'s TypeSchema<'s, SF>,

    _p: PhantomData<VF>,
}

impl<'de, 's, SF, VF> DeserializeSeed<'de> for Seed<'s, SF, VF>
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
        self.schema.decode_value(deserializer)
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
