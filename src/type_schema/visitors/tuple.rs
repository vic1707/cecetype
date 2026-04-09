use super::{Resolver, Seed};
use crate::{SchemaFlavor, TypeSchema, Value, ValueBuilder};
use ::{
    core::{fmt, marker::PhantomData},
    serde::{
        de::{self, SeqAccess, Visitor},
        Deserialize,
    },
};

pub struct TupleVisitor<'a, 's, SF: SchemaFlavor<'s>, VB: ValueBuilder> {
    elements: &'s SF::List<TypeSchema<'s, SF>>,
    resolver: Option<&'a Resolver<'a, 's, SF>>,
    _p: PhantomData<VB>,
}

impl<'a, 's, SF: SchemaFlavor<'s>, VB: ValueBuilder> TupleVisitor<'a, 's, SF, VB> {
    pub const fn new(
        elements: &'s SF::List<TypeSchema<'s, SF>>,
        resolver: Option<&'a Resolver<'a, 's, SF>>,
    ) -> Self {
        Self {
            elements,
            resolver,
            _p: PhantomData,
        }
    }
}

impl<'de, 's, SF, VB> Visitor<'de> for TupleVisitor<'_, 's, SF, VB>
where
    SF: SchemaFlavor<'s>,
    VB: ValueBuilder,
    VB::Str: Deserialize<'de>,
{
    type Value = Value<VB>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "tuple")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = VB::list_with_capacity(self.elements.len());

        for schema in &**self.elements {
            let el = seq
                .next_element_seed(Seed {
                    schema,
                    resolver: self.resolver,
                    _p: PhantomData,
                })?
                .ok_or_else(|| de::Error::invalid_length(values.len(), &self))?;

            VB::list_push(&mut values, el);
        }

        if seq.next_element::<de::IgnoredAny>()?.is_some() {
            return Err(de::Error::invalid_length(self.elements.len() + 1, &self));
        }

        Ok(Value::Tuple(values))
    }
}
