use super::{StructVisitor, TupleVisitor};
use crate::{
    SchemaFlavor, TypeSchema, Value, ValueBuilder, ValueFlavor, VariantSchema,
    type_schema::visitors::Seed,
};
use ::{
    core::{fmt, marker::PhantomData, ops::Deref as _},
    serde::{
        Deserialize,
        de::{EnumAccess, VariantAccess as _, Visitor},
    },
};

#[derive(Deserialize)]
#[serde(untagged)]
enum VariantId<VF: ValueFlavor> {
    Index(u32),
    Name(VF::Str),
}

impl<VF: ValueFlavor> fmt::Display for VariantId<VF> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Index(idx) => write!(f, "[id: {idx}]"),
            Self::Name(name) => write!(f, "{}", name.deref()),
        }
    }
}

pub struct EnumVisitor<'s, SF: SchemaFlavor<'s>, VF: ValueBuilder> {
    name: &'s SF::Str,
    variants: &'s SF::List<VariantSchema<'s, SF>>,

    _p: PhantomData<VF>,
}

impl<'s, SF: SchemaFlavor<'s>, VF: ValueBuilder> EnumVisitor<'s, SF, VF> {
    pub fn new(name: &'s SF::Str, variants: &'s SF::List<VariantSchema<'s, SF>>) -> Self {
        Self {
            name,
            variants,
            _p: PhantomData,
        }
    }
}

impl<'de, 's, SF, VF> Visitor<'de> for EnumVisitor<'s, SF, VF>
where
    SF: SchemaFlavor<'s>,
    VF: ValueBuilder,
    VF::Str: Deserialize<'de>,
{
    type Value = Value<VF>;

    fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "enum {}", self.name.deref())
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: EnumAccess<'de>,
    {
        let (variant_identifier, variant_access) = data.variant::<VariantId<VF>>()?;

        let variant_schema = match variant_identifier {
            VariantId::Name(ref variant_name) => self
                .variants
                .deref()
                .iter()
                .find(|v| v.name() == variant_name.deref()),
            VariantId::Index(idx) => self
                .variants
                .deref()
                .iter()
                .find(|v| v.discriminant() == &idx),
        }
        .ok_or_else(|| {
            serde::de::Error::custom(format_args!("unknown variant: `{variant_identifier}`"))
        })?;

        let value = match variant_schema.deref() {
            VariantSchema::Unit { .. } => {
                variant_access.unit_variant()?;

                Value::EnumUnit {
                    name: VF::make_str(self.name),
                    discriminant: *variant_schema.discriminant(),
                    variant_name: VF::make_str(variant_schema.name()),
                }
            }

            VariantSchema::Tuple { fields, .. } => {
                let Value::Tuple(fields) = variant_access
                    .tuple_variant(fields.len(), TupleVisitor::<SF, VF>::new(fields))?
                else {
                    unreachable!()
                };

                Value::EnumTuple {
                    name: VF::make_str(self.name),
                    discriminant: *variant_schema.discriminant(),
                    variant_name: VF::make_str(variant_schema.name()),
                    fields,
                }
            }

            VariantSchema::NewType { field, .. } => {
                let field = VF::make_ptr(variant_access.newtype_variant_seed(Seed {
                    schema: field,
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
                // TODO: ok fields empty?
                let Value::Struct { fields, .. } = variant_access.struct_variant(
                    super::_S[fields.len()],
                    StructVisitor::<SF, VF>::new(name, fields),
                )?
                else {
                    unreachable!()
                };

                Value::EnumStruct {
                    name: VF::make_str(self.name),
                    discriminant: *variant_schema.discriminant(),
                    variant_name: VF::make_str(variant_schema.name()),
                    fields,
                }
            }
        };

        Ok(value)
    }
}

pub struct OptionVisitor<'s, SF: SchemaFlavor<'s>, VF: ValueBuilder> {
    some: &'s TypeSchema<'s, SF>,

    _p: PhantomData<VF>,
}

impl<'s, SF: SchemaFlavor<'s>, VF: ValueBuilder> OptionVisitor<'s, SF, VF> {
    pub fn new(some: &'s TypeSchema<'s, SF>) -> Self {
        Self {
            some,
            _p: PhantomData,
        }
    }
}

impl<'de, 's, SF, VF> Visitor<'de> for OptionVisitor<'s, SF, VF>
where
    SF: SchemaFlavor<'s>,
    VF: ValueBuilder,
    VF::Str: Deserialize<'de>,
{
    type Value = Value<VF>;

    fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Option<{}>", self.some)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Option(None))
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = self.some.decode_value::<_, VF>(deserializer)?;
        Ok(Value::Option(Some(VF::make_ptr(value))))
    }
}
