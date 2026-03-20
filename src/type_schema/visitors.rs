mod array;
mod r#enum;
mod slice;
mod r#struct;
mod tuple;
pub use self::{
    array::ArrayVisitor,
    r#enum::EnumVisitor,
    slice::SliceVisitor,
    r#struct::{NewTypeStructVisitor, StructVisitor, TupleStructVisitor, UnitStructVisitor},
    tuple::TupleVisitor,
};

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
