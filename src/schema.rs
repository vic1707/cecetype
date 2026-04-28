#![expect(clippy::module_name_repetitions, reason = "_")]
mod primitive_impls;
mod visitors;

use crate::{
    flavors::{OwnedSchemaFlavor, SchemaFlavor, ValueBuilder, ser},
    utils::as_static_str,
    value::Value,
};
use ::{
    core::{fmt, ops::Deref as _},
    serde::{Deserialize, Serialize, de},
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, crate::Schema)]
#[non_exhaustive]
pub enum RefKind {
    Direct,
    Slice,
}

#[derive(crate::Schema)]
#[schema(bounds(SF::Str: crate::Schema))]
#[::derive_where::derive_where(Clone, Debug, PartialEq;)] // prevents compiler bounds check overflow & `SF` bounds
#[derive(Serialize, Deserialize)]
#[serde(bound(
    serialize = "SF::Str: Serialize",
    deserialize = "SF: OwnedSchemaFlavor<'s>, SF::Str: Deserialize<'de>"
))]
#[non_exhaustive]
pub enum Data<'s, SF: SchemaFlavor<'s>> {
    Unit {
        name: SF::Str,
    },
    NewType {
        name: SF::Str,
        #[schema(ref(Schema))]
        #[serde(serialize_with = "ser::serialize_ptr")]
        #[serde(deserialize_with = "SF::deserialize_ptr")]
        field: SF::Ptr<Schema<'s, SF>>,
    },
    Tuple {
        name: SF::Str,
        #[schema(ref(Schema, list))]
        #[serde(serialize_with = "ser::serialize_list_ptr")]
        #[serde(deserialize_with = "SF::deserialize_list")]
        fields: SF::List<Schema<'s, SF>>,
    },
    Struct {
        name: SF::Str,
        #[schema(as([FieldSchema<'s, SF>]))]
        #[serde(serialize_with = "ser::serialize_list_ptr")]
        #[serde(deserialize_with = "SF::deserialize_list")]
        fields: SF::List<FieldSchema<'s, SF>>,
    },
}

impl<'s, SF: SchemaFlavor<'s>> Data<'s, SF> {
    #[inline]
    pub fn name(&self) -> &str {
        match self {
            Self::Unit { name }
            | Self::NewType { name, .. }
            | Self::Tuple { name, .. }
            | Self::Struct { name, .. } => name.as_ref(),
        }
    }
}

#[derive(crate::Schema)]
#[schema(bounds(SF::Str: crate::Schema))]
#[::derive_where::derive_where(Clone, Debug, PartialEq;)] // prevents compiler bounds check overflow & `SF` bounds
#[derive(Serialize, Deserialize)]
#[serde(bound(
    serialize = "SF::Str: Serialize",
    deserialize = "SF: OwnedSchemaFlavor<'s>, SF::Str: Deserialize<'de>"
))]
#[non_exhaustive]
pub enum Schema<'s, SF: SchemaFlavor<'s>> {
    Ref {
        name: SF::Str,
        kind: RefKind,
    }, // special case to avoid recursive/cyclic types

    Unit,

    Bool,

    Str,
    Char,

    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    U128,
    I128,

    Array {
        #[schema(ref(Schema))]
        #[serde(serialize_with = "ser::serialize_ptr")]
        #[serde(deserialize_with = "SF::deserialize_ptr")]
        element: SF::Ptr<Self>,
        len: usize,
    },

    Slice {
        #[schema(ref(Schema))]
        #[serde(serialize_with = "ser::serialize_ptr")]
        #[serde(deserialize_with = "SF::deserialize_ptr")]
        element: SF::Ptr<Self>,
    },

    Map {
        #[schema(ref(Schema))]
        #[serde(serialize_with = "ser::serialize_ptr")]
        #[serde(deserialize_with = "SF::deserialize_ptr")]
        key: SF::Ptr<Self>,
        #[schema(ref(Schema))]
        #[serde(serialize_with = "ser::serialize_ptr")]
        #[serde(deserialize_with = "SF::deserialize_ptr")]
        value: SF::Ptr<Self>,
    },

    Tuple {
        #[schema(ref(Schema, list))]
        #[serde(serialize_with = "ser::serialize_list_ptr")]
        #[serde(deserialize_with = "SF::deserialize_list")]
        elements: SF::List<Self>,
    },

    Struct {
        // TODO: tuple variant when `yaml_serde` supports nested enums
        data: Data<'s, SF>,
    },

    Enum {
        name: SF::Str,
        #[schema(as([(u32, Data<'s, SF>)]))]
        #[serde(serialize_with = "ser::serialize_list_ptr")]
        #[serde(deserialize_with = "SF::deserialize_list")]
        variants: SF::List<(u32, Data<'s, SF>)>,
    },

    Option(
        #[schema(ref(Schema))]
        #[serde(serialize_with = "ser::serialize_ptr")]
        #[serde(deserialize_with = "SF::deserialize_ptr")]
        SF::Ptr<Self>,
    ),
}

#[derive(crate::Schema)]
#[schema(bounds(SF::Str: crate::Schema))]
#[::derive_where::derive_where(Clone, Debug, PartialEq;)] // prevents compiler bounds check overflow & `SF` bounds
#[derive(Serialize, Deserialize)]
#[serde(bound(
    serialize = "SF::Str: Serialize",
    deserialize = "SF: OwnedSchemaFlavor<'s>, SF::Str: Deserialize<'de>"
))]
pub struct FieldSchema<'s, SF: SchemaFlavor<'s>> {
    pub name: SF::Str,
    // pub key: u32, // Maybe for future protocols
    #[schema(ref(Schema))]
    #[serde(serialize_with = "ser::serialize_ptr")]
    #[serde(deserialize_with = "SF::deserialize_ptr")]
    pub ty: SF::Ptr<Schema<'s, SF>>,
}

impl<'s, SF> fmt::Display for Data<'s, SF>
where
    SF: SchemaFlavor<'s>,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unit { name } => write!(f, "{}", name.as_ref()),
            Self::NewType { name, field } => {
                write!(f, "{} ({})", name.as_ref(), &**field)
            }
            Self::Tuple { name, fields } => {
                write!(f, "{} (", name.as_ref())?;
                for (idx, field) in fields.deref().iter().enumerate() {
                    if idx != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", &**field)?;
                }
                write!(f, ")")
            }
            Self::Struct { name, fields } => {
                write!(f, "{} {{ ", name.as_ref())?;
                for (idx, field) in fields.deref().iter().enumerate() {
                    if idx != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", field.name.as_ref(), &*field.ty)?;
                }
                write!(f, " }}")
            }
        }
    }
}

impl<'s, SF> fmt::Display for Schema<'s, SF>
where
    SF: SchemaFlavor<'s>,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Schema::Ref { name, kind } => match kind {
                RefKind::Direct => write!(f, "-> {}", name.as_ref()),
                RefKind::Slice => write!(f, "-> [{}]", name.as_ref()),
            },
            Schema::Unit => write!(f, "()"),
            Schema::Bool => write!(f, "bool"),
            Schema::Str => write!(f, "str"),
            Schema::Char => write!(f, "char"),

            Schema::U8 => write!(f, "u8"),
            Schema::U16 => write!(f, "u16"),
            Schema::U32 => write!(f, "u32"),
            Schema::U64 => write!(f, "u64"),

            Schema::I8 => write!(f, "i8"),
            Schema::I16 => write!(f, "i16"),
            Schema::I32 => write!(f, "i32"),
            Schema::I64 => write!(f, "i64"),

            Schema::F32 => write!(f, "f32"),
            Schema::F64 => write!(f, "f64"),
            Schema::U128 => write!(f, "u128"),
            Schema::I128 => write!(f, "i128"),

            Schema::Array { element, len } => {
                write!(f, "[{}; {}]", &**element, len)
            }

            Schema::Slice { element } => {
                write!(f, "[{}]", &**element)
            }

            Schema::Map { key, value } => {
                write!(f, "Map<{}, {}>", &**key, &**value)
            }

            Schema::Tuple { elements } => {
                write!(f, "(")?;
                for (i, elem) in elements.deref().iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", &**elem)?;
                }
                write!(f, ")")
            }

            Schema::Struct { data } => write!(f, "{data}"),

            Schema::Enum {
                name: enum_name,
                variants,
            } => {
                write!(f, "{} {{ ", enum_name.as_ref())?;

                for (idx, variant) in variants.deref().iter().enumerate() {
                    let (discriminant, data) = &**variant;
                    if idx != 0 {
                        write!(f, " | ")?;
                    }
                    match &data {
                        Data::Unit { name } => {
                            write!(f, "{} = {}", name.as_ref(), discriminant)?;
                        }
                        Data::Struct { name, fields } => {
                            write!(f, "{} = {}({{ ", name.as_ref(), discriminant)?;
                            for (fidx, field) in fields.deref().iter().enumerate() {
                                if fidx != 0 {
                                    write!(f, ", ")?;
                                }
                                write!(f, "{}: {}", field.name.as_ref(), &*field.ty)?;
                            }
                            write!(f, " }})")?;
                        }
                        Data::Tuple { name, fields } => {
                            write!(f, "{} = {}(", name.as_ref(), discriminant)?;
                            for (fidx, field) in fields.deref().iter().enumerate() {
                                if fidx != 0 {
                                    write!(f, ", ")?;
                                }
                                write!(f, "{}", &**field)?;
                            }
                            write!(f, ")")?;
                        }
                        Data::NewType { name, field } => {
                            write!(f, "{} = {}({})", name.as_ref(), discriminant, &**field)?;
                        }
                    }
                }

                write!(f, " }}")
            }

            Schema::Option(schema) => write!(f, "Option<{}>", &**schema),
        }
    }
}

impl<'s, SF> Schema<'s, SF>
where
    SF: SchemaFlavor<'s>,
{
    /// Deserialize a [`Value`] from the given deserializer using this schema.
    ///
    /// [`Schema::Ref`] nodes are resolved automatically.
    #[inline]
    pub fn decode_value<'de, D, VB>(&'s self, deserializer: D) -> Result<Value<VB>, D::Error>
    where
        D: serde::Deserializer<'de>,
        VB: ValueBuilder,
        VB::Str: Deserialize<'de>,
    {
        self.decode_value_with_resolver::<_, VB>(deserializer, None)
    }

    pub(crate) fn decode_value_with_resolver<'de, D, VB>(
        &'s self,
        deserializer: D,
        resolver: Option<&visitors::Resolver<'_, 's, SF>>,
    ) -> Result<Value<VB>, D::Error>
    where
        D: serde::Deserializer<'de>,
        VB: ValueBuilder,
        VB::Str: Deserialize<'de>,
    {
        match self {
            Schema::Ref { name, kind } => {
                let target = resolver
                    .and_then(|res| res.resolve(name.as_ref()))
                    .ok_or_else(|| {
                        de::Error::custom(format_args!("unresolved schema ref `{}`", name.as_ref()))
                    })?;

                match kind {
                    RefKind::Direct => {
                        target.decode_value_with_resolver::<_, VB>(deserializer, resolver)
                    }
                    RefKind::Slice => deserializer
                        .deserialize_seq(visitors::SliceVisitor::<SF, VB>::new(target, resolver)),
                }
            }

            Schema::Unit => {
                <()>::deserialize(deserializer)?;
                Ok(Value::Unit)
            }
            Schema::Bool => Ok(Value::Bool(bool::deserialize(deserializer)?)),
            Schema::Str => Ok(Value::Str(<VB::Str>::deserialize(deserializer)?)),
            Schema::Char => Ok(Value::Char(char::deserialize(deserializer)?)),
            Schema::U8 => Ok(Value::U8(u8::deserialize(deserializer)?)),
            Schema::U16 => Ok(Value::U16(u16::deserialize(deserializer)?)),
            Schema::U32 => Ok(Value::U32(u32::deserialize(deserializer)?)),
            Schema::U64 => Ok(Value::U64(u64::deserialize(deserializer)?)),
            Schema::I8 => Ok(Value::I8(i8::deserialize(deserializer)?)),
            Schema::I16 => Ok(Value::I16(i16::deserialize(deserializer)?)),
            Schema::I32 => Ok(Value::I32(i32::deserialize(deserializer)?)),
            Schema::I64 => Ok(Value::I64(i64::deserialize(deserializer)?)),
            Schema::F32 => Ok(Value::F32(f32::deserialize(deserializer)?)),
            Schema::F64 => Ok(Value::F64(f64::deserialize(deserializer)?)),
            Schema::U128 => Ok(Value::U128(u128::deserialize(deserializer)?)),
            Schema::I128 => Ok(Value::I128(i128::deserialize(deserializer)?)),

            Schema::Array { element, len } => deserializer.deserialize_tuple(
                *len,
                visitors::ArrayVisitor::<SF, VB>::new(element, *len, resolver),
            ),
            Schema::Slice { element } => {
                deserializer
                    .deserialize_seq(visitors::SliceVisitor::<SF, VB>::new(element, resolver))
            }
            Schema::Map { key, value } => {
                deserializer
                    .deserialize_map(visitors::MapVisitor::<SF, VB>::new(key, value, resolver))
            }
            Schema::Tuple { elements } => deserializer.deserialize_tuple(
                elements.len(),
                visitors::TupleVisitor::<SF, VB>::new(elements, resolver),
            ),

            Schema::Struct { data } => match data {
                Data::Unit { name } => deserializer.deserialize_unit_struct(
                    as_static_str(name),
                    visitors::UnitStructVisitor::<SF, VB>::new(name),
                ),

                Data::NewType { name, field } => {
                    let entry = visitors::Resolver::new(name.as_ref(), self, resolver);
                    deserializer.deserialize_newtype_struct(
                        as_static_str(name),
                        visitors::NewTypeStructVisitor::<SF, VB>::new(name, field, Some(&entry)),
                    )
                }
                Data::Tuple { name, fields } => {
                    let entry = visitors::Resolver::new(name.as_ref(), self, resolver);
                    deserializer.deserialize_tuple_struct(
                        as_static_str(name),
                        fields.len(),
                        visitors::TupleStructVisitor::<SF, VB>::new(name, fields, Some(&entry)),
                    )
                }
                Data::Struct { name, fields } => {
                    let entry = visitors::Resolver::new(name.as_ref(), self, resolver);
                    deserializer.deserialize_struct(
                        as_static_str(name), // dunno
                        // Cannot send empty list as postcard uses the length to encode
                        visitors::names(fields.len()), // dirty ass hack
                        visitors::StructVisitor::<SF, VB>::new(name, fields, Some(&entry)),
                    )
                }
            },

            Schema::Enum { name, variants } => {
                let entry = visitors::Resolver::new(name.as_ref(), self, resolver);
                deserializer.deserialize_enum(
                    as_static_str(name), // dunno
                    // Cannot send empty list as postcard uses the length to encode
                    visitors::names(variants.len()), // dirty ass hack
                    visitors::EnumVisitor::<SF, VB>::new(name, variants, Some(&entry)),
                )
            }

            Schema::Option(schema) => {
                deserializer
                    .deserialize_option(visitors::OptionVisitor::<SF, VB>::new(schema, resolver))
            }
        }
    }
}
