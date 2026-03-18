use super::Seed;
use crate::{SchemaFlavor, TypeSchema, Value, ValueBuilder, ValueFlavor};
use ::{
    core::marker::PhantomData,
    serde::{
        Deserialize,
        de::{SeqAccess, Visitor},
    },
};

pub struct SliceVisitor<'s, SF: SchemaFlavor<'s>, VF: ValueFlavor<'s>> {
    element: &'s SF::Ptr<TypeSchema<'s, SF>>,
    _p: PhantomData<VF>,
}

impl<'s, SF: SchemaFlavor<'s>, VF: ValueFlavor<'s>> SliceVisitor<'s, SF, VF> {
    pub fn new(element: &'s SF::Ptr<TypeSchema<'s, SF>>) -> Self {
        Self {
            element,
            _p: PhantomData,
        }
    }
}

impl<'de, 's, SF, VF> Visitor<'de> for SliceVisitor<'s, SF, VF>
where
    SF: SchemaFlavor<'s>,
    VF: ValueBuilder<'s>,
    VF::Str: Deserialize<'de>,
{
    type Value = Value<'s, VF>;

    fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Slice")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = VF::list();

        while let Some(v) = seq.next_element_seed(Seed {
            schema: self.element,
            _p: PhantomData,
        })? {
            VF::list_push(&mut values, v);
        }

        Ok(Value::Slice(values))
    }
}
