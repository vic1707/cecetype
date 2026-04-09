use super::{Resolver, Seed};
use crate::{value::Data as ValueData, FieldSchema, SchemaFlavor, TypeSchema, Value, ValueBuilder};
use ::{
    core::{fmt, iter, marker::PhantomData},
    serde::{
        de::{self, MapAccess, SeqAccess, Visitor},
        Deserialize,
    },
};

pub struct StructVisitor<'a, 's, SF, VB>
where
    SF: SchemaFlavor<'s>,
    VB: ValueBuilder,
{
    name: &'s SF::Str,
    fields: &'s SF::List<FieldSchema<'s, SF>>,
    resolver: Option<&'a Resolver<'a, 's, SF>>,

    _p: PhantomData<VB>,
}

impl<'a, 's, SF, VB> StructVisitor<'a, 's, SF, VB>
where
    SF: SchemaFlavor<'s>,
    VB: ValueBuilder,
{
    pub const fn new(
        name: &'s SF::Str,
        fields: &'s SF::List<FieldSchema<'s, SF>>,
        resolver: Option<&'a Resolver<'a, 's, SF>>,
    ) -> Self {
        Self {
            name,
            fields,
            resolver,
            _p: PhantomData,
        }
    }
}

impl<'de, 's, SF, VB> Visitor<'de> for StructVisitor<'_, 's, SF, VB>
where
    SF: SchemaFlavor<'s>,
    VB: ValueBuilder,
    VB::Str: Deserialize<'de>,
{
    type Value = Value<VB>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "struct {}", self.name.as_ref())
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let fields = &**self.fields;
        let mut values = VB::list_with_capacity(fields.len());

        for (i, field) in fields.iter().enumerate() {
            let value = seq
                .next_element_seed(Seed {
                    schema: &field.ty,
                    resolver: self.resolver,
                    _p: PhantomData,
                })?
                .ok_or_else(|| de::Error::invalid_length(i, &self))?;

            VB::list_push(&mut values, (VB::make_str(&field.name), value));
        }

        if seq.next_element::<de::IgnoredAny>()?.is_some() {
            return Err(de::Error::invalid_length(fields.len() + 1, &self));
        }

        Ok(Value::Struct {
            data: ValueData::Struct {
                name: VB::make_str(self.name),
                fields: values,
            },
        })
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let fields = &**self.fields;

        let mut slots = VB::list_from_iter(iter::repeat_with(|| None).take(fields.len()));

        while let Some(key) = map.next_key::<VB::Str>()? {
            let (field, slot @ &mut None) = fields
                .iter()
                .zip(slots.iter_mut())
                .find(|(field, _)| field.name.as_ref() == key.as_ref())
                .ok_or_else(|| {
                    de::Error::custom(format_args!(
                        "unknown field `{}` in struct `{}`",
                        key.as_ref(), self.name.as_ref()
                    ))
                })?
            else {
                return Err(de::Error::custom(format_args!(
                    "duplicate field `{}` in struct `{}`",
                    key.as_ref(), self.name.as_ref()
                )));
            };

            let value = map.next_value_seed(Seed {
                schema: &field.ty,
                resolver: self.resolver,
                _p: PhantomData,
            })?;

            *slot = Some(value);
        }

        let mut values = VB::list_with_capacity(fields.len());

        for (field, slot) in fields.iter().zip(slots.iter_mut()) {
            let value = slot.take().ok_or_else(|| {
                de::Error::custom(format_args!(
                    "missing field `{}` in struct `{}`",
                    field.name.as_ref(), self.name.as_ref()
                ))
            })?;

            VB::list_push(&mut values, (VB::make_str(&field.name), value));
        }

        Ok(Value::Struct {
            data: ValueData::Struct {
                name: VB::make_str(self.name),
                fields: values,
            },
        })
    }
}

pub struct UnitStructVisitor<'s, SF, VB>
where
    SF: SchemaFlavor<'s>,
    VB: ValueBuilder,
{
    name: &'s SF::Str,

    _p: PhantomData<VB>,
}

impl<'s, SF, VB> UnitStructVisitor<'s, SF, VB>
where
    SF: SchemaFlavor<'s>,
    VB: ValueBuilder,
{
    pub const fn new(name: &'s SF::Str) -> Self {
        Self {
            name,
            _p: PhantomData,
        }
    }
}

impl<'de, 's, SF, VB> Visitor<'de> for UnitStructVisitor<'s, SF, VB>
where
    SF: SchemaFlavor<'s>,
    VB: ValueBuilder,
    VB::Str: Deserialize<'de>,
{
    type Value = Value<VB>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "unit struct {}", self.name.as_ref())
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Struct {
            data: ValueData::Unit {
                name: VB::make_str(self.name),
            },
        })
    }
}

pub struct TupleStructVisitor<'a, 's, SF, VB>
where
    SF: SchemaFlavor<'s>,
    VB: ValueBuilder,
{
    name: &'s SF::Str,
    fields: &'s SF::List<TypeSchema<'s, SF>>,
    resolver: Option<&'a Resolver<'a, 's, SF>>,

    _p: PhantomData<VB>,
}

impl<'a, 's, SF, VB> TupleStructVisitor<'a, 's, SF, VB>
where
    SF: SchemaFlavor<'s>,
    VB: ValueBuilder,
{
    pub const fn new(
        name: &'s SF::Str,
        fields: &'s SF::List<TypeSchema<'s, SF>>,
        resolver: Option<&'a Resolver<'a, 's, SF>>,
    ) -> Self {
        Self {
            name,
            fields,
            resolver,
            _p: PhantomData,
        }
    }
}

impl<'de, 's, SF, VB> Visitor<'de> for TupleStructVisitor<'_, 's, SF, VB>
where
    SF: SchemaFlavor<'s>,
    VB: ValueBuilder,
    VB::Str: Deserialize<'de>,
{
    type Value = Value<VB>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "tuple struct {}", self.name.as_ref())
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let fields = &**self.fields;
        let mut values = VB::list_with_capacity(fields.len());

        for (i, schema) in fields.iter().enumerate() {
            let value = seq
                .next_element_seed(Seed {
                    schema,
                    resolver: self.resolver,
                    _p: PhantomData,
                })?
                .ok_or_else(|| de::Error::invalid_length(i, &self))?;

            VB::list_push(&mut values, value);
        }

        if seq.next_element::<de::IgnoredAny>()?.is_some() {
            return Err(de::Error::invalid_length(fields.len() + 1, &self));
        }

        Ok(Value::Struct {
            data: ValueData::Tuple {
                name: VB::make_str(self.name),
                fields: values,
            },
        })
    }
}

pub struct NewTypeStructVisitor<'a, 's, SF, VB>
where
    SF: SchemaFlavor<'s>,
    VB: ValueBuilder,
{
    name: &'s SF::Str,
    field: &'s TypeSchema<'s, SF>,
    resolver: Option<&'a Resolver<'a, 's, SF>>,

    _p: PhantomData<VB>,
}

impl<'a, 's, SF, VB> NewTypeStructVisitor<'a, 's, SF, VB>
where
    SF: SchemaFlavor<'s>,
    VB: ValueBuilder,
{
    pub const fn new(
        name: &'s SF::Str,
        field: &'s TypeSchema<'s, SF>,
        resolver: Option<&'a Resolver<'a, 's, SF>>,
    ) -> Self {
        Self {
            name,
            field,
            resolver,
            _p: PhantomData,
        }
    }
}

impl<'de, 's, SF, VB> Visitor<'de> for NewTypeStructVisitor<'_, 's, SF, VB>
where
    SF: SchemaFlavor<'s>,
    VB: ValueBuilder,
    VB::Str: Deserialize<'de>,
{
    type Value = Value<VB>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "newtype struct {}", self.name.as_ref())
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let value = self
            .field
            .decode_value_with_resolver::<_, VB>(deserializer, self.resolver)?;

        Ok(Value::Struct {
            data: ValueData::NewType {
                name: VB::make_str(self.name),
                field: VB::make_ptr(value),
            },
        })
    }
}
