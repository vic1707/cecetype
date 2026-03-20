use super::Seed;
use crate::{SchemaFlavor, TypeSchema, Value, ValueBuilder, ValueFlavor};
use ::{
    core::{marker::PhantomData, ops::Deref as _},
    serde::{
        de::{self, SeqAccess, Visitor},
        Deserialize,
    },
};

pub struct TupleVisitor<'s, SF: SchemaFlavor<'s>, VF: ValueFlavor> {
    elements: &'s SF::List<TypeSchema<'s, SF>>,
    _p: PhantomData<VF>,
}

impl<'s, SF: SchemaFlavor<'s>, VF: ValueFlavor> TupleVisitor<'s, SF, VF> {
    pub fn new(elements: &'s SF::List<TypeSchema<'s, SF>>) -> Self {
        Self {
            elements,
            _p: PhantomData,
        }
    }
}

impl<'de, 's, SF, VF> Visitor<'de> for TupleVisitor<'s, SF, VF>
where
    SF: SchemaFlavor<'s>,
    VF: ValueBuilder,
    VF::Str: Deserialize<'de>,
{
    type Value = Value<VF>;

    fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "tuple")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = VF::list_with_capacity(self.elements.len());

        for schema in self.elements.deref() {
            let v = seq
                .next_element_seed(Seed {
                    schema,
                    _p: PhantomData,
                })?
                .ok_or_else(|| de::Error::invalid_length(values.len(), &self))?;

            VF::list_push(&mut values, v);
        }

        if seq.next_element::<de::IgnoredAny>()?.is_some() {
            return Err(de::Error::invalid_length(self.elements.len() + 1, &self));
        }

        Ok(Value::Tuple(values))
    }
}
