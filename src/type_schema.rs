mod primitive_impls;
mod visitors;

use crate::{
    flavors::ser, utils::as_static_str, OwnedSchemaFlavor, SchemaFlavor, Value, ValueBuilder,
};
use ::{
    core::{fmt, ops::Deref as _},
    derive_where::derive_where,
    serde::{de, Deserialize, Serialize},
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, crate::Schema)]
#[non_exhaustive]
pub enum RefKind {
    Direct,
    Slice,
}

#[derive(crate::Schema)]
#[schema(bounds(SF::Str: crate::Schema))]
#[derive_where(Clone; )] // prevents compiler bounds check overflow & `SF: Debug` bound
#[derive_where(Debug; )] // prevents compiler bounds check overflow & `SF: Debug` bound
#[derive_where(PartialEq;)] // prevents compiler bounds check overflow & `SF: PartialEq` bound
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
        #[schema(ref(TypeSchema))]
        #[serde(serialize_with = "ser::serialize_ptr")]
        #[serde(deserialize_with = "SF::deserialize_ptr")]
        field: SF::Ptr<TypeSchema<'s, SF>>,
    },
    Tuple {
        name: SF::Str,
        #[schema(ref(TypeSchema, list))]
        #[serde(serialize_with = "ser::serialize_list_ptr")]
        #[serde(deserialize_with = "SF::deserialize_list")]
        fields: SF::List<TypeSchema<'s, SF>>,
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
            | Self::Struct { name, .. } => name,
        }
    }
}

#[derive(crate::Schema)]
#[schema(bounds(SF::Str: crate::Schema))]
#[derive_where(Clone; )] // prevents compiler bounds check overflow & `SF: Debug` bound
#[derive_where(Debug; )] // prevents compiler bounds check overflow & `SF: Debug` bound
#[derive_where(PartialEq;)] // prevents compiler bounds check overflow & `SF: PartialEq` bound
#[derive(Serialize, Deserialize)]
#[serde(bound(
    serialize = "SF::Str: Serialize",
    deserialize = "SF: OwnedSchemaFlavor<'s>, SF::Str: Deserialize<'de>"
))]
#[non_exhaustive]
pub enum TypeSchema<'s, SF: SchemaFlavor<'s>> {
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
        #[schema(ref(TypeSchema))]
        #[serde(serialize_with = "ser::serialize_ptr")]
        #[serde(deserialize_with = "SF::deserialize_ptr")]
        element: SF::Ptr<Self>,
        len: usize,
    },

    Slice {
        #[schema(ref(TypeSchema))]
        #[serde(serialize_with = "ser::serialize_ptr")]
        #[serde(deserialize_with = "SF::deserialize_ptr")]
        element: SF::Ptr<Self>,
    },

    Map {
        #[schema(ref(TypeSchema))]
        #[serde(serialize_with = "ser::serialize_ptr")]
        #[serde(deserialize_with = "SF::deserialize_ptr")]
        key: SF::Ptr<Self>,
        #[schema(ref(TypeSchema))]
        #[serde(serialize_with = "ser::serialize_ptr")]
        #[serde(deserialize_with = "SF::deserialize_ptr")]
        value: SF::Ptr<Self>,
    },

    Tuple {
        #[schema(ref(TypeSchema, list))]
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
        #[schema(ref(TypeSchema))]
        #[serde(serialize_with = "ser::serialize_ptr")]
        #[serde(deserialize_with = "SF::deserialize_ptr")]
        SF::Ptr<Self>,
    ),
}

#[derive(crate::Schema)]
#[schema(bounds(SF::Str: crate::Schema))]
#[derive_where(Clone; )] // prevents compiler bounds check overflow & `SF: Debug` bound
#[derive_where(Debug; )] // prevents compiler bounds check overflow & `SF: Debug` bound
#[derive_where(PartialEq;)] // prevents compiler bounds check overflow & `SF: PartialEq` bound
#[derive(Serialize, Deserialize)]
#[serde(bound(
    serialize = "SF::Str: Serialize",
    deserialize = "SF: OwnedSchemaFlavor<'s>, SF::Str: Deserialize<'de>"
))]
pub struct FieldSchema<'s, SF: SchemaFlavor<'s>> {
    pub name: SF::Str,
    // pub key: u32, // Maybe for future protocols
    #[schema(ref(TypeSchema))]
    #[serde(serialize_with = "ser::serialize_ptr")]
    #[serde(deserialize_with = "SF::deserialize_ptr")]
    pub ty: SF::Ptr<TypeSchema<'s, SF>>,
}

impl<'s, SF> fmt::Display for Data<'s, SF>
where
    SF: SchemaFlavor<'s>,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unit { name } => write!(f, "{}", &**name),
            Self::NewType { name, field } => {
                write!(f, "{} ({})", &**name, &**field)
            }
            Self::Tuple { name, fields } => {
                write!(f, "{} (", &**name)?;
                for (idx, field) in fields.deref().iter().enumerate() {
                    if idx != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", &**field)?;
                }
                write!(f, ")")
            }
            Self::Struct { name, fields } => {
                write!(f, "{} {{ ", &**name)?;
                for (idx, field) in fields.deref().iter().enumerate() {
                    if idx != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", &*field.name, &*field.ty)?;
                }
                write!(f, " }}")
            }
        }
    }
}

impl<'s, SF> fmt::Display for TypeSchema<'s, SF>
where
    SF: SchemaFlavor<'s>,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeSchema::Ref { name, kind } => match kind {
                RefKind::Direct => write!(f, "-> {}", &**name),
                RefKind::Slice => write!(f, "-> [{}]", &**name),
            },
            TypeSchema::Unit => write!(f, "()"),
            TypeSchema::Bool => write!(f, "bool"),
            TypeSchema::Str => write!(f, "str"),
            TypeSchema::Char => write!(f, "char"),

            TypeSchema::U8 => write!(f, "u8"),
            TypeSchema::U16 => write!(f, "u16"),
            TypeSchema::U32 => write!(f, "u32"),
            TypeSchema::U64 => write!(f, "u64"),

            TypeSchema::I8 => write!(f, "i8"),
            TypeSchema::I16 => write!(f, "i16"),
            TypeSchema::I32 => write!(f, "i32"),
            TypeSchema::I64 => write!(f, "i64"),

            TypeSchema::F32 => write!(f, "f32"),
            TypeSchema::F64 => write!(f, "f64"),
            TypeSchema::U128 => write!(f, "u128"),
            TypeSchema::I128 => write!(f, "i128"),

            TypeSchema::Array { element, len } => {
                write!(f, "[{}; {}]", &**element, len)
            }

            TypeSchema::Slice { element } => {
                write!(f, "[{}]", &**element)
            }

            TypeSchema::Map { key, value } => {
                write!(f, "Map<{}, {}>", &**key, &**value)
            }

            TypeSchema::Tuple { elements } => {
                write!(f, "(")?;
                for (i, elem) in elements.deref().iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", &**elem)?;
                }
                write!(f, ")")
            }

            TypeSchema::Struct { data } => write!(f, "{data}"),

            TypeSchema::Enum {
                name: enum_name,
                variants,
            } => {
                write!(f, "{} {{ ", &**enum_name)?;

                for (idx, variant) in variants.deref().iter().enumerate() {
                    let (discriminant, data) = &**variant;
                    if idx != 0 {
                        write!(f, " | ")?;
                    }
                    match &data {
                        Data::Unit { name } => {
                            write!(f, "{} = {}", &**name, discriminant)?;
                        }
                        Data::Struct { name, fields } => {
                            write!(f, "{} = {}({{ ", &**name, discriminant)?;
                            for (fidx, field) in fields.deref().iter().enumerate() {
                                if fidx != 0 {
                                    write!(f, ", ")?;
                                }
                                write!(f, "{}: {}", &*field.name, &*field.ty)?;
                            }
                            write!(f, " }})")?;
                        }
                        Data::Tuple { name, fields } => {
                            write!(f, "{} = {}(", &**name, discriminant)?;
                            for (fidx, field) in fields.deref().iter().enumerate() {
                                if fidx != 0 {
                                    write!(f, ", ")?;
                                }
                                write!(f, "{}", &**field)?;
                            }
                            write!(f, ")")?;
                        }
                        Data::NewType { name, field } => {
                            write!(f, "{} = {}({})", &**name, discriminant, &**field)?;
                        }
                    }
                }

                write!(f, " }}")
            }

            TypeSchema::Option(schema) => write!(f, "Option<{}>", &**schema),
        }
    }
}

impl<'s, SF> TypeSchema<'s, SF>
where
    SF: SchemaFlavor<'s>,
{
    /// Deserialize a [`Value`] from the given deserializer using this schema.
    ///
    /// [`TypeSchema::Ref`] nodes are resolved automatically.
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
            TypeSchema::Ref { name, kind } => {
                let target = resolver.and_then(|res| res.resolve(name)).ok_or_else(|| {
                    de::Error::custom(format_args!("unresolved schema ref `{}`", &**name))
                })?;

                match kind {
                    RefKind::Direct => {
                        target.decode_value_with_resolver::<_, VB>(deserializer, resolver)
                    }
                    RefKind::Slice => deserializer
                        .deserialize_seq(visitors::SliceVisitor::<SF, VB>::new(target, resolver)),
                }
            }

            TypeSchema::Unit => {
                <()>::deserialize(deserializer)?;
                Ok(Value::Unit)
            }
            TypeSchema::Bool => Ok(Value::Bool(bool::deserialize(deserializer)?)),
            TypeSchema::Str => Ok(Value::Str(<VB::Str>::deserialize(deserializer)?)),
            TypeSchema::Char => Ok(Value::Char(char::deserialize(deserializer)?)),
            TypeSchema::U8 => Ok(Value::U8(u8::deserialize(deserializer)?)),
            TypeSchema::U16 => Ok(Value::U16(u16::deserialize(deserializer)?)),
            TypeSchema::U32 => Ok(Value::U32(u32::deserialize(deserializer)?)),
            TypeSchema::U64 => Ok(Value::U64(u64::deserialize(deserializer)?)),
            TypeSchema::I8 => Ok(Value::I8(i8::deserialize(deserializer)?)),
            TypeSchema::I16 => Ok(Value::I16(i16::deserialize(deserializer)?)),
            TypeSchema::I32 => Ok(Value::I32(i32::deserialize(deserializer)?)),
            TypeSchema::I64 => Ok(Value::I64(i64::deserialize(deserializer)?)),
            TypeSchema::F32 => Ok(Value::F32(f32::deserialize(deserializer)?)),
            TypeSchema::F64 => Ok(Value::F64(f64::deserialize(deserializer)?)),
            TypeSchema::U128 => Ok(Value::U128(u128::deserialize(deserializer)?)),
            TypeSchema::I128 => Ok(Value::I128(i128::deserialize(deserializer)?)),

            TypeSchema::Array { element, len } => deserializer.deserialize_tuple(
                *len,
                visitors::ArrayVisitor::<SF, VB>::new(element, *len, resolver),
            ),
            TypeSchema::Slice { element } => deserializer
                .deserialize_seq(visitors::SliceVisitor::<SF, VB>::new(element, resolver)),
            TypeSchema::Map { key, value } => {
                deserializer
                    .deserialize_map(visitors::MapVisitor::<SF, VB>::new(key, value, resolver))
            }
            TypeSchema::Tuple { elements } => deserializer.deserialize_tuple(
                elements.len(),
                visitors::TupleVisitor::<SF, VB>::new(elements, resolver),
            ),

            TypeSchema::Struct { data } => match data {
                Data::Unit { name } => deserializer.deserialize_unit_struct(
                    as_static_str(name),
                    visitors::UnitStructVisitor::<SF, VB>::new(name),
                ),

                Data::NewType { name, field } => {
                    let entry = visitors::Resolver::new(name, self, resolver);
                    deserializer.deserialize_newtype_struct(
                        as_static_str(name),
                        visitors::NewTypeStructVisitor::<SF, VB>::new(name, field, Some(&entry)),
                    )
                }
                Data::Tuple { name, fields } => {
                    let entry = visitors::Resolver::new(name, self, resolver);
                    deserializer.deserialize_tuple_struct(
                        as_static_str(name),
                        fields.len(),
                        visitors::TupleStructVisitor::<SF, VB>::new(name, fields, Some(&entry)),
                    )
                }
                Data::Struct { name, fields } => {
                    let entry = visitors::Resolver::new(name, self, resolver);
                    deserializer.deserialize_struct(
                        as_static_str(name), // dunno
                        // Cannot send empty list as postcard uses the length to encode
                        visitors::names(fields.len()), // dirty ass hack
                        visitors::StructVisitor::<SF, VB>::new(name, fields, Some(&entry)),
                    )
                }
            },

            TypeSchema::Enum { name, variants } => {
                let entry = visitors::Resolver::new(name, self, resolver);
                deserializer.deserialize_enum(
                    as_static_str(name), // dunno
                    // Cannot send empty list as postcard uses the length to encode
                    visitors::names(variants.len()), // dirty ass hack
                    visitors::EnumVisitor::<SF, VB>::new(name, variants, Some(&entry)),
                )
            }

            TypeSchema::Option(schema) => deserializer
                .deserialize_option(visitors::OptionVisitor::<SF, VB>::new(schema, resolver)),
        }
    }
}
