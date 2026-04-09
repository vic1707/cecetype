use super::{Resolver, Seed};
use crate::{SchemaFlavor, TypeSchema, Value, ValueBuilder};
use ::{
    core::{fmt, marker::PhantomData},
    serde::{
        de::{SeqAccess, Visitor},
        Deserialize,
    },
};

pub struct SliceVisitor<'a, 's, SF: SchemaFlavor<'s>, VB: ValueBuilder> {
    element: &'s TypeSchema<'s, SF>,
    resolver: Option<&'a Resolver<'a, 's, SF>>,
    _p: PhantomData<VB>,
}

impl<'a, 's, SF: SchemaFlavor<'s>, VB: ValueBuilder> SliceVisitor<'a, 's, SF, VB> {
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

impl<'de, 's, SF, VB> Visitor<'de> for SliceVisitor<'_, 's, SF, VB>
where
    SF: SchemaFlavor<'s>,
    VB: ValueBuilder,
    VB::Str: Deserialize<'de>,
{
    type Value = Value<VB>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "Slice")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = seq
            .size_hint()
            .map_or_else(VB::list, VB::list_with_capacity);

        while let Some(el) = seq.next_element_seed(Seed {
            schema: self.element,
            resolver: self.resolver,
            _p: PhantomData,
        })? {
            VB::list_push(&mut values, el);
        }

        Ok(Value::Slice(values))
    }
}
