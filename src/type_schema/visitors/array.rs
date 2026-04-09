use super::{Resolver, Seed};
use crate::{SchemaFlavor, TypeSchema, Value, ValueBuilder};
use ::{
    core::{fmt, marker::PhantomData},
    serde::{
        de::{self, SeqAccess, Visitor},
        Deserialize,
    },
};

pub struct ArrayVisitor<'a, 's, SF: SchemaFlavor<'s>, VB: ValueBuilder> {
    element: &'s TypeSchema<'s, SF>,
    len: usize,
    resolver: Option<&'a Resolver<'a, 's, SF>>,
    _p: PhantomData<VB>,
}

impl<'a, 's, SF: SchemaFlavor<'s>, VB: ValueBuilder> ArrayVisitor<'a, 's, SF, VB> {
    pub const fn new(
        element: &'s TypeSchema<'s, SF>,
        len: usize,
        resolver: Option<&'a Resolver<'a, 's, SF>>,
    ) -> Self {
        Self {
            element,
            len,
            resolver,
            _p: PhantomData,
        }
    }
}

impl<'de, 's, SF, VB> Visitor<'de> for ArrayVisitor<'_, 's, SF, VB>
where
    SF: SchemaFlavor<'s>,
    VB: ValueBuilder,
    VB::Str: Deserialize<'de>,
{
    type Value = Value<VB>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "array of length {}", self.len)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = VB::list_with_capacity(self.len);

        for i in 0..self.len {
            let el = seq
                .next_element_seed(Seed {
                    schema: self.element,
                    resolver: self.resolver,
                    _p: PhantomData,
                })?
                .ok_or_else(|| de::Error::invalid_length(i, &self))?;

            VB::list_push(&mut values, el);
        }

        if seq.next_element::<de::IgnoredAny>()?.is_some() {
            return Err(de::Error::invalid_length(self.len + 1, &self));
        }

        Ok(Value::Array(values))
    }
}
