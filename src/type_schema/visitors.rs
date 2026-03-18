mod array;
mod slice;
mod tuple;
pub use self::{array::ArrayVisitor, tuple::TupleVisitor, slice::SliceVisitor};

use crate::{SchemaFlavor, TypeSchema, Value, ValueBuilder};
use ::{
    core::marker::PhantomData,
    serde::{Deserialize, de::DeserializeSeed},
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
