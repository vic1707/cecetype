use super::{Resolver, Seed};
use crate::{SchemaFlavor, TypeSchema, Value, ValueBuilder};
use ::{
    core::{fmt, marker::PhantomData},
    serde::{
        de::{MapAccess, Visitor},
        Deserialize,
    },
};

pub struct MapVisitor<'a, 's, SF: SchemaFlavor<'s>, VB: ValueBuilder> {
    key: &'s TypeSchema<'s, SF>,
    value: &'s TypeSchema<'s, SF>,
    resolver: Option<&'a Resolver<'a, 's, SF>>,
    _p: PhantomData<VB>,
}

impl<'a, 's, SF: SchemaFlavor<'s>, VB: ValueBuilder> MapVisitor<'a, 's, SF, VB> {
    pub const fn new(
        key: &'s TypeSchema<'s, SF>,
        value: &'s TypeSchema<'s, SF>,
        resolver: Option<&'a Resolver<'a, 's, SF>>,
    ) -> Self {
        Self {
            key,
            value,
            resolver,
            _p: PhantomData,
        }
    }
}

impl<'de, 's, SF, VB> Visitor<'de> for MapVisitor<'_, 's, SF, VB>
where
    SF: SchemaFlavor<'s>,
    VB: ValueBuilder,
    VB::Str: Deserialize<'de>,
{
    type Value = Value<VB>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "Map")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut entries = map
            .size_hint()
            .map_or_else(VB::list, VB::list_with_capacity);

        while let Some(key) = map.next_key_seed(Seed {
            schema: self.key,
            resolver: self.resolver,
            _p: PhantomData,
        })? {
            let value = map.next_value_seed(Seed {
                schema: self.value,
                resolver: self.resolver,
                _p: PhantomData,
            })?;
            VB::list_push(&mut entries, (key, value));
        }

        Ok(Value::Map(entries))
    }
}
