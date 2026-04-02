use super::{Resolver, StructVisitor, TupleVisitor};
use crate::{
    type_schema::visitors::Seed, SchemaFlavor, TypeSchema, Value, ValueBuilder, ValueFlavor,
    VariantSchema,
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

pub struct EnumVisitor<'a, 's, SF: SchemaFlavor<'s>, VF: ValueBuilder> {
    name: &'s SF::Str,
    variants: &'s SF::List<VariantSchema<'s, SF>>,
    resolver: Option<&'a Resolver<'a, 's, SF>>,

    _p: PhantomData<VF>,
}

impl<'a, 's, SF: SchemaFlavor<'s>, VF: ValueBuilder> EnumVisitor<'a, 's, SF, VF> {
    pub const fn new(
        name: &'s SF::Str,
        variants: &'s SF::List<VariantSchema<'s, SF>>,
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

impl<'de, 's, SF, VF> Visitor<'de> for EnumVisitor<'_, 's, SF, VF>
where
    SF: SchemaFlavor<'s>,
    VF: ValueBuilder,
    VF::Str: Deserialize<'de>,
{
    type Value = Value<VF>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "enum {}", &**self.name)
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: EnumAccess<'de>,
    {
        let (variant_identifier, variant_access) = data.variant::<VariantId<VF>>()?;

        let variant_schema = match &variant_identifier {
            VariantId::Name(variant_name) => self
                .variants
                .iter()
                .find(|variant| variant.name() == &**variant_name),
            VariantId::Index(idx) => self
                .variants
                .deref()
                .iter()
                .find(|variant| variant.discriminant() == idx),
        }
        .ok_or_else(|| {
            de::Error::custom(format_args!("unknown variant: `{variant_identifier}`"))
        })?;

        let value = match &**variant_schema {
            VariantSchema::Unit { .. } => {
                variant_access.unit_variant()?;

                Value::EnumUnit {
                    name: VF::make_str(self.name),
                    discriminant: *variant_schema.discriminant(),
                    variant_name: VF::make_str(variant_schema.name()),
                }
            }

            VariantSchema::Tuple { fields, .. } => {
                let Value::Tuple(fields_schema) = variant_access.tuple_variant(
                    fields.len(),
                    TupleVisitor::<SF, VF>::new(fields, self.resolver),
                )?
                else {
                    unreachable!()
                };

                Value::EnumTuple {
                    name: VF::make_str(self.name),
                    discriminant: *variant_schema.discriminant(),
                    variant_name: VF::make_str(variant_schema.name()),
                    fields: fields_schema,
                }
            }

            VariantSchema::NewType {
                field: field_schema,
                ..
            } => {
                let field = VF::make_ptr(variant_access.newtype_variant_seed(Seed {
                    schema: field_schema,
                    resolver: self.resolver,
                    _p: PhantomData,
                })?);

                Value::EnumNewType {
                    name: VF::make_str(self.name),
                    discriminant: *variant_schema.discriminant(),
                    variant_name: VF::make_str(variant_schema.name()),
                    field,
                }
            }

            VariantSchema::Struct { name, fields, .. } => {
                let Value::Struct {
                    fields: fields_schema,
                    ..
                } = variant_access.struct_variant(
                    // Cannot send empty list as postcard uses the length to encode
                    super::names(fields.len()),
                    StructVisitor::<SF, VF>::new(name, fields, self.resolver),
                )?
                else {
                    unreachable!()
                };

                Value::EnumStruct {
                    name: VF::make_str(self.name),
                    discriminant: *variant_schema.discriminant(),
                    variant_name: VF::make_str(variant_schema.name()),
                    fields: fields_schema,
                }
            }
        };

        Ok(value)
    }
}

pub struct OptionVisitor<'a, 's, SF: SchemaFlavor<'s>, VF: ValueBuilder> {
    some: &'s TypeSchema<'s, SF>,
    resolver: Option<&'a Resolver<'a, 's, SF>>,

    _p: PhantomData<VF>,
}

impl<'a, 's, SF: SchemaFlavor<'s>, VF: ValueBuilder> OptionVisitor<'a, 's, SF, VF> {
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

impl<'de, 's, SF, VF> Visitor<'de> for OptionVisitor<'_, 's, SF, VF>
where
    SF: SchemaFlavor<'s>,
    VF: ValueBuilder,
    VF::Str: Deserialize<'de>,
{
    type Value = Value<VF>;

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
            .decode_value_with_resolver::<_, VF>(deserializer, self.resolver)?;
        Ok(Value::Option(Some(VF::make_ptr(value))))
    }
}

impl<VF: ValueFlavor> fmt::Display for VariantId<VF> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Index(idx) => write!(f, "[id: {idx}]"),
            Self::Name(name) => write!(f, "{}", &**name),
        }
    }
}

struct VariantIdVisitor<VF: ValueBuilder>(PhantomData<VF>);

impl<VF> Visitor<'_> for VariantIdVisitor<VF>
where
    VF: ValueBuilder,
{
    type Value = VariantId<VF>;

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
        Ok(VariantId::Name(VF::make_str(v)))
    }
}

impl<'de, VF> Deserialize<'de> for VariantId<VF>
where
    VF: ValueBuilder,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // We cannot derive this: `EnumAccess::variant` asks the format for an
        // identifier, and RON only drives that path through `deserialize_identifier`.
        deserializer.deserialize_identifier(VariantIdVisitor::<VF>(PhantomData))
    }
}
