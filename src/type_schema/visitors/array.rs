use super::Seed;
use crate::{SchemaFlavor, TypeSchema, Value, ValueBuilder, ValueFlavor};
use ::{
    core::marker::PhantomData,
    serde::{
        Deserialize,
        de::{self, SeqAccess, Visitor},
    },
};

pub struct ArrayVisitor<'s, SF: SchemaFlavor<'s>, VF: ValueFlavor> {
    element: &'s SF::Ptr<TypeSchema<'s, SF>>,
    len: usize,
    _p: PhantomData<VF>,
}

impl<'s, SF: SchemaFlavor<'s>, VF: ValueFlavor> ArrayVisitor<'s, SF, VF> {
    pub fn new(element: &'s SF::Ptr<TypeSchema<'s, SF>>, len: usize) -> Self {
        Self {
            element,
            len,
            _p: PhantomData,
        }
    }
}

impl<'de, 's, SF, VF> Visitor<'de> for ArrayVisitor<'s, SF, VF>
where
    SF: SchemaFlavor<'s>,
    VF: ValueBuilder,
    VF::Str: Deserialize<'de>,
{
    type Value = Value<VF>;

    fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "array of length {}", self.len)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = VF::list_with_capacity(self.len);

        for _ in 0..self.len {
            let v = seq
                .next_element_seed(Seed {
                    schema: self.element,
                    _p: PhantomData,
                })?
                .ok_or_else(|| de::Error::invalid_length(values.len(), &self))?;

            VF::list_push(&mut values, v);
        }

        Ok(Value::Array(values))
    }
}
