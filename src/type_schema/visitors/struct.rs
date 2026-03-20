use super::Seed;
use crate::{FieldSchema, SchemaFlavor, TypeSchema, Value, ValueBuilder};
use ::{
    core::{marker::PhantomData, ops::Deref as _},
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
        write!(f, "struct {}", self.name.deref())
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
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
        let fields = self.fields.deref();

        let mut slots = VF::list_from_iter((0..fields.len()).map(|_| None));

        while let Some(key) = map.next_key::<VF::Str>()? {
            let (field, slot @ &mut None) = fields
                .iter()
                .zip(slots.iter_mut())
                .find(|(f, _)| *f.name == *key)
                .ok_or_else(|| {
                    de::Error::custom(format_args!("unknown field `{}`", key.deref()))
                })?
            else {
                return Err(de::Error::custom(format_args!(
                    "duplicate field `{}`",
                    key.deref()
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
                de::Error::custom(format_args!("missing field `{}`", field.name.deref()))
            })?;

            VF::list_push(&mut values, (VF::make_str(&field.name), value));
        }

        Ok(Value::Struct {
            name: VF::make_str(self.name),
            fields: values,
        })
    }
}

pub struct UnitStructVisitor<'s, SF, VF>
where
    SF: SchemaFlavor<'s>,
    VF: ValueBuilder,
{
    name: &'s SF::Str,

    _p: PhantomData<VF>,
}

impl<'s, SF, VF> UnitStructVisitor<'s, SF, VF>
where
    SF: SchemaFlavor<'s>,
    VF: ValueBuilder,
{
    pub fn new(name: &'s SF::Str) -> Self {
        Self {
            name,
            _p: PhantomData,
        }
    }
}

impl<'de, 's, SF, VF> Visitor<'de> for UnitStructVisitor<'s, SF, VF>
where
    SF: SchemaFlavor<'s>,
    VF: ValueBuilder,
    VF::Str: Deserialize<'de>,
{
    type Value = Value<VF>;

    fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "unit struct {}", self.name.deref())
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::UnitStruct {
            name: VF::make_str(self.name),
        })
    }
}

pub struct TupleStructVisitor<'s, SF, VF>
where
    SF: SchemaFlavor<'s>,
    VF: ValueBuilder,
{
    name: &'s SF::Str,
    fields: &'s SF::List<TypeSchema<'s, SF>>,

    _p: PhantomData<VF>,
}

impl<'s, SF, VF> TupleStructVisitor<'s, SF, VF>
where
    SF: SchemaFlavor<'s>,
    VF: ValueBuilder,
{
    pub fn new(name: &'s SF::Str, fields: &'s SF::List<TypeSchema<'s, SF>>) -> Self {
        Self {
            name,
            fields,
            _p: PhantomData,
        }
    }
}

impl<'de, 's, SF, VF> Visitor<'de> for TupleStructVisitor<'s, SF, VF>
where
    SF: SchemaFlavor<'s>,
    VF: ValueBuilder,
    VF::Str: Deserialize<'de>,
{
    type Value = Value<VF>;

    fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "newtype struct {}", self.name.deref())
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let fields = self.fields.deref();
        let mut values = VF::list_with_capacity(fields.len());

        for (i, schema) in fields.iter().enumerate() {
            let value = seq
                .next_element_seed(Seed {
                    schema,
                    _p: PhantomData,
                })?
                .ok_or_else(|| de::Error::invalid_length(i, &self))?;

            VF::list_push(&mut values, value);
        }

        if seq.next_element::<de::IgnoredAny>()?.is_some() {
            return Err(de::Error::invalid_length(fields.len() + 1, &self));
        }

        Ok(Value::TupleStruct {
            name: VF::make_str(self.name),
            fields: values,
        })
    }
}

pub struct NewTypeStructVisitor<'s, SF, VF>
where
    SF: SchemaFlavor<'s>,
    VF: ValueBuilder,
{
    name: &'s SF::Str,
    field: &'s TypeSchema<'s, SF>,

    _p: PhantomData<VF>,
}

impl<'s, SF, VF> NewTypeStructVisitor<'s, SF, VF>
where
    SF: SchemaFlavor<'s>,
    VF: ValueBuilder,
{
    pub fn new(name: &'s SF::Str, fields: &'s TypeSchema<'s, SF>) -> Self {
        Self {
            name,
            field: fields,
            _p: PhantomData,
        }
    }
}

impl<'de, 's, SF, VF> Visitor<'de> for NewTypeStructVisitor<'s, SF, VF>
where
    SF: SchemaFlavor<'s>,
    VF: ValueBuilder,
    VF::Str: Deserialize<'de>,
{
    type Value = Value<VF>;

    fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "newtype struct {}", self.name.deref())
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let value = self.field.decode_value::<_, VF>(deserializer)?;

        Ok(Value::NewTypeStruct {
            name: VF::make_str(self.name),
            field: VF::make_ptr(value),
        })
    }
}
