use super::Seed;
use crate::{FieldSchema, SchemaFlavor, Value, ValueBuilder};
use ::{
    core::marker::PhantomData,
    serde::{
        Deserialize,
        de::{self, MapAccess, SeqAccess, Visitor},
    },
};

pub struct StructVisitor<'s, SF, VF>
where
    SF: SchemaFlavor<'s>,
    VF: ValueBuilder,
{
    name: &'s SF::Str,
    fields: &'s SF::List<FieldSchema<'s, SF>>,

    _p: PhantomData<VF>,
}

impl<'s, SF, VF> StructVisitor<'s, SF, VF>
where
    SF: SchemaFlavor<'s>,
    VF: ValueBuilder,
{
    pub fn new(name: &'s SF::Str, fields: &'s SF::List<FieldSchema<'s, SF>>) -> Self {
        Self {
            name,
            fields,
            _p: PhantomData,
        }
    }
}

impl<'de, 's, SF, VF> Visitor<'de> for StructVisitor<'s, SF, VF>
where
    SF: SchemaFlavor<'s>,
    VF: ValueBuilder,
    VF::Str: Deserialize<'de>,
{
    type Value = Value<VF>;

    fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "struct {}", &**self.name)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        use ::core::ops::Deref as _;

        let fields = self.fields.deref();
        let mut values = VF::list_with_capacity(fields.len());

        for (i, field) in fields.iter().enumerate() {
            let value = seq
                .next_element_seed(Seed {
                    schema: &field.ty,
                    _p: PhantomData,
                })?
                .ok_or_else(|| de::Error::invalid_length(i, &self))?;

            VF::list_push(&mut values, (VF::make_str(&field.name), value));
        }

        if seq.next_element::<de::IgnoredAny>()?.is_some() {
            return Err(de::Error::invalid_length(fields.len() + 1, &self));
        }

        Ok(Value::Struct {
            name: VF::make_str(self.name),
            fields: values,
        })
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        use ::core::ops::Deref as _;

        let fields = self.fields.deref();

        let mut slots = VF::list_from_iter((0..fields.len()).map(|_| None));

        while let Some(key) = map.next_key::<VF::Str>()? {
            let (field, slot @ &mut None) = fields
                .iter()
                .zip(slots.iter_mut())
                .find(|(f, _)| *f.name == *key)
                .ok_or_else(|| de::Error::custom(format_args!("unknown field `{}`", &*key)))?
            else {
                return Err(de::Error::custom(format_args!(
                    "duplicate field `{}`",
                    &*key
                )));
            };

            let value = map.next_value_seed(Seed {
                schema: &field.ty,
                _p: PhantomData,
            })?;

            *slot = Some(value);
        }

        let mut values = VF::list_with_capacity(fields.len());

        for (field, slot) in fields.iter().zip(slots.iter_mut()) {
            let value = slot.take().ok_or_else(|| {
                de::Error::custom(format_args!("missing field `{}`", &*field.name))
            })?;

            VF::list_push(&mut values, (VF::make_str(&field.name), value));
        }

        Ok(Value::Struct {
            name: VF::make_str(self.name),
            fields: values,
        })
    }
}
