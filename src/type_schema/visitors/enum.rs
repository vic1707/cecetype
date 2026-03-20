use super::{StructVisitor, TupleVisitor};
use crate::{SchemaFlavor, Value, ValueBuilder, ValueFlavor, VariantSchema, VariantValue};
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
    Index(i32),
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

                Value::Enum {
                    name: VF::make_str(self.name),
                    variant: VariantValue::Unit {
                        name: VF::make_str(variant_schema.name()),
                    },
                }
            }

            VariantSchema::Tuple { fields, .. } => {
                let Value::Tuple(fields) = variant_access
                    .tuple_variant(fields.len(), TupleVisitor::<SF, VF>::new(fields))?
                else {
                    unreachable!()
                };

                Value::Enum {
                    name: VF::make_str(self.name),
                    variant: VariantValue::Tuple {
                        name: VF::make_str(variant_schema.name()),
                        fields,
                    },
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

                Value::Enum {
                    name: VF::make_str(self.name),
                    variant: VariantValue::Struct {
                        name: VF::make_str(variant_schema.name()),
                        fields,
                    },
                }
            }
        };

        Ok(value)
    }
}
