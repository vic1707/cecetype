use super::Seed;
use crate::{SchemaFlavor, TypeSchema, Value, ValueBuilder, ValueFlavor};
use ::{
    core::{fmt, marker::PhantomData},
    serde::{
        de::{MapAccess, Visitor},
        Deserialize,
    },
};

pub struct MapVisitor<'s, SF: SchemaFlavor<'s>, VF: ValueFlavor> {
    key: &'s SF::Ptr<TypeSchema<'s, SF>>,
    value: &'s SF::Ptr<TypeSchema<'s, SF>>,
    _p: PhantomData<VF>,
}

impl<'s, SF: SchemaFlavor<'s>, VF: ValueFlavor> MapVisitor<'s, SF, VF> {
    pub const fn new(
        key: &'s SF::Ptr<TypeSchema<'s, SF>>,
        value: &'s SF::Ptr<TypeSchema<'s, SF>>,
    ) -> Self {
        Self {
            key,
            value,
            _p: PhantomData,
        }
    }
}

impl<'de, 's, SF, VF> Visitor<'de> for MapVisitor<'s, SF, VF>
where
    SF: SchemaFlavor<'s>,
    VF: ValueBuilder,
    VF::Str: Deserialize<'de>,
{
    type Value = Value<VF>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "Map")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut entries = map
            .size_hint()
            .map_or_else(VF::list, VF::list_with_capacity);

        while let Some(key) = map.next_key_seed(Seed {
            schema: self.key,
            _p: PhantomData,
        })? {
            let value = map.next_value_seed(Seed {
                schema: self.value,
                _p: PhantomData,
            })?;
            VF::list_push(&mut entries, (key, value));
        }

        Ok(Value::Map(entries))
    }
}
