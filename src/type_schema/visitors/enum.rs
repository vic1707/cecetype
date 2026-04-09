use super::{Resolver, StructVisitor, TupleVisitor};
use crate::{
    type_schema::{visitors::Seed, Data},
    value::Data as ValueData,
    SchemaFlavor, TypeSchema, Value, ValueBuilder, ValueFlavor,
};
use ::{
    core::{fmt, marker::PhantomData, ops::Deref as _},
    serde::{
        de::{self, EnumAccess, VariantAccess as _, Visitor},
        Deserialize,
    },
};

enum VariantId<VF: ValueFlavor> {
    Index(u32),
    Name(VF::Str),
}

pub struct EnumVisitor<'a, 's, SF: SchemaFlavor<'s>, VB: ValueBuilder> {
    name: &'s SF::Str,
    variants: &'s SF::List<(u32, Data<'s, SF>)>,
    resolver: Option<&'a Resolver<'a, 's, SF>>,

    _p: PhantomData<VB>,
}

impl<'a, 's, SF: SchemaFlavor<'s>, VB: ValueBuilder> EnumVisitor<'a, 's, SF, VB> {
    pub const fn new(
        name: &'s SF::Str,
        variants: &'s SF::List<(u32, Data<'s, SF>)>,
        resolver: Option<&'a Resolver<'a, 's, SF>>,
    ) -> Self {
        Self {
            name,
            variants,
            resolver,
            _p: PhantomData,
        }
    }
}

impl<'de, 's, SF, VB> Visitor<'de> for EnumVisitor<'_, 's, SF, VB>
where
    SF: SchemaFlavor<'s>,
    VB: ValueBuilder,
    VB::Str: Deserialize<'de>,
{
    type Value = Value<VB>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "enum {}", self.name.as_ref())
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: EnumAccess<'de>,
    {
        let (variant_identifier, variant_access) = data.variant::<VariantId<VB>>()?;

        let (discriminant, variant_schema) = &**match &variant_identifier {
            VariantId::Name(variant_name) => self
                .variants
                .iter()
                .find(|variant| variant.1.name() == variant_name.as_ref()),
            VariantId::Index(idx) => self
                .variants
                .deref()
                .iter()
                .find(|variant| variant.0 == *idx),
        }
        .ok_or_else(|| {
            de::Error::custom(format_args!("unknown variant: `{variant_identifier}`"))
        })?;

        let value_data = match &variant_schema {
            Data::Unit { .. } => {
                variant_access.unit_variant()?;

                ValueData::Unit {
                    name: VB::make_str(variant_schema.name()),
                }
            }

            Data::Tuple { fields, .. } => {
                let Value::Tuple(fields_value) = variant_access.tuple_variant(
                    fields.len(),
                    TupleVisitor::<SF, VB>::new(fields, self.resolver),
                )?
                else {
                    unreachable!()
                };

                ValueData::Tuple {
                    name: VB::make_str(variant_schema.name()),
                    fields: fields_value,
                }
            }

            Data::NewType {
                field: field_schema,
                ..
            } => {
                let field = VB::make_ptr(variant_access.newtype_variant_seed(Seed {
                    schema: field_schema,
                    resolver: self.resolver,
                    _p: PhantomData,
                })?);

                ValueData::NewType {
                    name: VB::make_str(variant_schema.name()),
                    field,
                }
            }

            Data::Struct { name, fields, .. } => {
                let Value::Struct {
                    data:
                        ValueData::Struct {
                            fields: fields_value,
                            ..
                        },
                } = variant_access.struct_variant(
                    // Cannot send empty list as postcard uses the length to encode
                    super::names(fields.len()),
                    StructVisitor::<SF, VB>::new(name, fields, self.resolver),
                )?
                else {
                    unreachable!()
                };

                ValueData::Struct {
                    name: VB::make_str(variant_schema.name()),
                    fields: fields_value,
                }
            }
        };

        Ok(Value::Enum {
            name: VB::make_str(self.name),
            discriminant: *discriminant,
            data: value_data,
        })
    }
}

pub struct OptionVisitor<'a, 's, SF: SchemaFlavor<'s>, VB: ValueBuilder> {
    some: &'s TypeSchema<'s, SF>,
    resolver: Option<&'a Resolver<'a, 's, SF>>,

    _p: PhantomData<VB>,
}

impl<'a, 's, SF: SchemaFlavor<'s>, VB: ValueBuilder> OptionVisitor<'a, 's, SF, VB> {
    pub const fn new(
        some: &'s TypeSchema<'s, SF>,
        resolver: Option<&'a Resolver<'a, 's, SF>>,
    ) -> Self {
        Self {
            some,
            resolver,
            _p: PhantomData,
        }
    }
}

impl<'de, 's, SF, VB> Visitor<'de> for OptionVisitor<'_, 's, SF, VB>
where
    SF: SchemaFlavor<'s>,
    VB: ValueBuilder,
    VB::Str: Deserialize<'de>,
{
    type Value = Value<VB>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "Option<{}>", self.some)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Option(None))
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = self
            .some
            .decode_value_with_resolver::<_, VB>(deserializer, self.resolver)?;
        Ok(Value::Option(Some(VB::make_ptr(value))))
    }
}

impl<VF: ValueFlavor> fmt::Display for VariantId<VF> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Index(idx) => write!(f, "[id: {idx}]"),
            Self::Name(name) => write!(f, "{}", name.as_ref()),
        }
    }
}

struct VariantIdVisitor<VB: ValueBuilder>(PhantomData<VB>);

impl<VB> Visitor<'_> for VariantIdVisitor<VB>
where
    VB: ValueBuilder,
{
    type Value = VariantId<VB>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a variant name or index")
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        u32::try_from(v)
            .map(VariantId::Index)
            .map_err(|_err| E::custom(format_args!("invalid variant id `{v}`")))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        u32::try_from(v)
            .map(VariantId::Index)
            .map_err(|_err| E::custom(format_args!("invalid variant id `{v}`")))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(VariantId::Name(VB::make_str(v)))
    }
}

impl<'de, VB> Deserialize<'de> for VariantId<VB>
where
    VB: ValueBuilder,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // We cannot derive this: `EnumAccess::variant` asks the format for an
        // identifier, and RON only drives that path through `deserialize_identifier`.
        deserializer.deserialize_identifier(VariantIdVisitor::<VB>(PhantomData))
    }
}
