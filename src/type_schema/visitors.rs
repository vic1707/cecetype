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

#[doc(hidden)]
pub const _S: [&[&str]; 16] = [
    &[""; 0], &[""; 1], &[""; 2], &[""; 3], &[""; 4], &[""; 5], &[""; 6], &[""; 7], &[""; 8],
    &[""; 9], &[""; 10], &[""; 11], &[""; 12], &[""; 13], &[""; 14], &[""; 15],
];
