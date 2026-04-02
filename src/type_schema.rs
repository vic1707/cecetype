mod primitive_impls;
mod visitors;

use crate::{flavors::ser, OwnedSchemaFlavor, SchemaFlavor, Value, ValueBuilder};
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
#[schema(bounds(F::Str: crate::Schema))]
#[derive_where(Clone; )] // prevents compiler bounds check overflow & `F: Debug` bound
#[derive_where(Debug; )] // prevents compiler bounds check overflow & `F: Debug` bound
#[derive_where(PartialEq;)] // prevents compiler bounds check overflow & `F: PartialEq` bound
#[derive(Serialize, Deserialize)]
#[serde(bound(
    serialize = "F::Str: Serialize",
    deserialize = "F: OwnedSchemaFlavor<'s>, F::Str: Deserialize<'de>"
))]
#[non_exhaustive]
pub enum TypeSchema<'s, F: SchemaFlavor<'s>> {
    Ref {
        name: F::Str,
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
        #[serde(deserialize_with = "F::deserialize_ptr")]
        element: F::Ptr<Self>,
        len: usize,
    },

    Slice {
        #[schema(ref(TypeSchema))]
        #[serde(serialize_with = "ser::serialize_ptr")]
        #[serde(deserialize_with = "F::deserialize_ptr")]
        element: F::Ptr<Self>,
    },

    Map {
        #[schema(ref(TypeSchema))]
        #[serde(serialize_with = "ser::serialize_ptr")]
        #[serde(deserialize_with = "F::deserialize_ptr")]
        key: F::Ptr<Self>,
        #[schema(ref(TypeSchema))]
        #[serde(serialize_with = "ser::serialize_ptr")]
        #[serde(deserialize_with = "F::deserialize_ptr")]
        value: F::Ptr<Self>,
    },

    Tuple {
        #[schema(ref(TypeSchema, list))]
        #[serde(serialize_with = "ser::serialize_list_ptr")]
        #[serde(deserialize_with = "F::deserialize_list")]
        elements: F::List<Self>,
    },

    UnitStruct {
        name: F::Str,
    },
    NewTypeStruct {
        name: F::Str,
        #[schema(ref(TypeSchema))]
        #[serde(serialize_with = "ser::serialize_ptr")]
        #[serde(deserialize_with = "F::deserialize_ptr")]
        field: F::Ptr<Self>,
    },
    TupleStruct {
        name: F::Str,
        #[schema(ref(TypeSchema, list))]
        #[serde(serialize_with = "ser::serialize_list_ptr")]
        #[serde(deserialize_with = "F::deserialize_list")]
        fields: F::List<Self>,
    },
    Struct {
        name: F::Str,
        #[schema(as([FieldSchema<'s, F>]))]
        #[serde(serialize_with = "ser::serialize_list_ptr")]
        #[serde(deserialize_with = "F::deserialize_list")]
        fields: F::List<FieldSchema<'s, F>>,
    },

    Enum {
        name: F::Str,
        #[schema(as([VariantSchema<'s, F>]))]
        #[serde(serialize_with = "ser::serialize_list_ptr")]
        #[serde(deserialize_with = "F::deserialize_list")]
        variants: F::List<VariantSchema<'s, F>>,
    },

    Option(
        #[schema(ref(TypeSchema))]
        #[serde(serialize_with = "ser::serialize_ptr")]
        #[serde(deserialize_with = "F::deserialize_ptr")]
        F::Ptr<Self>,
    ),
}

#[derive(crate::Schema)]
#[schema(bounds(F::Str: crate::Schema))]
#[derive_where(Clone; )] // prevents compiler bounds check overflow & `F: Debug` bound
#[derive_where(Debug; )] // prevents compiler bounds check overflow & `F: Debug` bound
#[derive_where(PartialEq;)] // prevents compiler bounds check overflow & `F: PartialEq` bound
#[derive(Serialize, Deserialize)]
#[serde(bound(
    serialize = "F::Str: Serialize",
    deserialize = "F: OwnedSchemaFlavor<'s>, F::Str: Deserialize<'de>"
))]
pub struct FieldSchema<'s, F: SchemaFlavor<'s>> {
    pub name: F::Str,
    // pub key: u32, // Maybe for future protocols
    #[schema(ref(TypeSchema))]
    #[serde(serialize_with = "ser::serialize_ptr")]
    #[serde(deserialize_with = "F::deserialize_ptr")]
    pub ty: F::Ptr<TypeSchema<'s, F>>,
}

#[derive(crate::Schema)]
#[schema(bounds(F::Str: crate::Schema))]
#[derive_where(Clone; )] // prevents compiler bounds check overflow & `F: Debug` bound
#[derive_where(Debug; )] // prevents compiler bounds check overflow & `F: Debug` bound
#[derive_where(PartialEq;)] // prevents compiler bounds check overflow & `F: PartialEq` bound
#[derive(Serialize, Deserialize)]
#[serde(bound(
    serialize = "F::Str: Serialize",
    deserialize = "F: OwnedSchemaFlavor<'s>, F::Str: Deserialize<'de>"
))]
// TODO: add Repr (Tagged(External/Internal/Adjacent), Untagged)
// Currently we assume externally tagged
#[non_exhaustive]
pub enum VariantSchema<'s, F: SchemaFlavor<'s>> {
    Unit {
        name: F::Str,
        discriminant: u32,
    },
    NewType {
        name: F::Str,
        discriminant: u32,
        #[schema(ref(TypeSchema))]
        #[serde(serialize_with = "ser::serialize_ptr")]
        #[serde(deserialize_with = "F::deserialize_ptr")]
        field: F::Ptr<TypeSchema<'s, F>>,
    },
    Tuple {
        name: F::Str,
        discriminant: u32,
        #[schema(ref(TypeSchema, list))]
        #[serde(serialize_with = "ser::serialize_list_ptr")]
        #[serde(deserialize_with = "F::deserialize_list")]
        fields: F::List<TypeSchema<'s, F>>,
    },
    Struct {
        name: F::Str,
        discriminant: u32,
        #[schema(ref(FieldSchema, list))]
        // #[schema(as([FieldSchema<'s, F>]))]
        #[serde(serialize_with = "ser::serialize_list_ptr")]
        #[serde(deserialize_with = "F::deserialize_list")]
        fields: F::List<FieldSchema<'s, F>>,
    },
}

impl<'s, F: SchemaFlavor<'s>> VariantSchema<'s, F> {
    fn name(&self) -> &str {
        match self {
            Self::Struct { name, .. }
            | Self::Unit { name, .. }
            | Self::Tuple { name, .. }
            | Self::NewType { name, .. } => name,
        }
    }
    const fn discriminant(&self) -> &u32 {
        match self {
            Self::Struct { discriminant, .. }
            | Self::Unit { discriminant, .. }
            | Self::Tuple { discriminant, .. }
            | Self::NewType { discriminant, .. } => discriminant,
        }
    }
}

impl<'s, F> fmt::Display for TypeSchema<'s, F>
where
    F: SchemaFlavor<'s>,
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

            TypeSchema::UnitStruct { name } => write!(f, "{}", &**name),
            TypeSchema::NewTypeStruct { name, field } => {
                write!(f, "{} ({})", &**name, &**field)
            }
            TypeSchema::TupleStruct { name, fields } => {
                write!(f, "{} (", &**name)?;
                for (idx, field) in fields.deref().iter().enumerate() {
                    if idx != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", &**field)?;
                }
                write!(f, ")")
            }
            TypeSchema::Struct { name, fields } => {
                write!(f, "{} {{ ", &**name)?;
                for (idx, field) in fields.deref().iter().enumerate() {
                    if idx != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", &*field.name, &*field.ty)?;
                }
                write!(f, " }}")
            }

            TypeSchema::Enum {
                name: enum_name,
                variants,
            } => {
                write!(f, "{} {{ ", &**enum_name)?;

                for (idx, variant) in variants.deref().iter().enumerate() {
                    if idx != 0 {
                        write!(f, " | ")?;
                    }
                    match &**variant {
                        VariantSchema::Unit { name, discriminant } => {
                            write!(f, "{} = {}", &**name, discriminant)?;
                        }
                        VariantSchema::Struct {
                            name,
                            discriminant,
                            fields,
                        } => {
                            write!(f, "{} = {}({{ ", &**name, discriminant)?;
                            for (fidx, field) in fields.deref().iter().enumerate() {
                                if fidx != 0 {
                                    write!(f, ", ")?;
                                }
                                write!(f, "{}: {}", &*field.name, &*field.ty)?;
                            }
                            write!(f, " }})")?;
                        }
                        VariantSchema::Tuple {
                            name,
                            discriminant,
                            fields,
                        } => {
                            write!(f, "{} = {}(", &**name, discriminant)?;
                            for (fidx, field) in fields.deref().iter().enumerate() {
                                if fidx != 0 {
                                    write!(f, ", ")?;
                                }
                                write!(f, "{}", &**field)?;
                            }
                            write!(f, ")")?;
                        }
                        VariantSchema::NewType {
                            name,
                            discriminant,
                            field,
                        } => {
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
    /// This is the main entry point for schema-driven deserialization.
    /// [`TypeSchema::Ref`] nodes are resolved automatically: named nodes
    /// (`Struct`, `Enum`, `NewTypeStruct`, `TupleStruct`) register themselves
    /// in an internal resolver as they are entered, so nested `Ref` nodes can
    /// look them up by name.
    #[inline]
    pub fn decode_value<'de, D, VF>(&'s self, deserializer: D) -> Result<Value<VF>, D::Error>
    where
        D: serde::Deserializer<'de>,
        VF: ValueBuilder,
        VF::Str: Deserialize<'de>,
    {
        self.decode_value_with_resolver::<_, VF>(deserializer, None)
    }

    /// Deserialize a [`Value`] from the given deserializer, resolving any
    /// [`TypeSchema::Ref`] nodes against the provided resolver chain.
    ///
    /// Named nodes (`Struct`, `Enum`, `UnitStruct`, `NewTypeStruct`,
    /// `TupleStruct`) push themselves onto the resolver chain so that nested
    /// `Ref` nodes can find them.
    pub(crate) fn decode_value_with_resolver<'de, D, VF>(
        &'s self,
        deserializer: D,
        resolver: Option<&visitors::Resolver<'_, 's, SF>>,
    ) -> Result<Value<VF>, D::Error>
    where
        D: serde::Deserializer<'de>,
        VF: ValueBuilder,
        VF::Str: Deserialize<'de>,
    {
        match self {
            TypeSchema::Ref { name, kind } => {
                let target = resolver.and_then(|res| res.resolve(name)).ok_or_else(|| {
                    de::Error::custom(format_args!("unresolved schema ref `{}`", &**name))
                })?;

                match kind {
                    RefKind::Direct => {
                        target.decode_value_with_resolver::<_, VF>(deserializer, resolver)
                    }
                    RefKind::Slice => {
                        deserializer.deserialize_seq(visitors::RefSliceVisitor::<SF, VF>::new(
                            target, resolver,
                        ))
                    }
                }
            }

            TypeSchema::Unit => {
                <()>::deserialize(deserializer)?;
                Ok(Value::Unit)
            }
            TypeSchema::Bool => Ok(Value::Bool(bool::deserialize(deserializer)?)),
            TypeSchema::Str => Ok(Value::Str(<VF::Str>::deserialize(deserializer)?)),
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
                visitors::ArrayVisitor::<SF, VF>::new(element, *len, resolver),
            ),
            TypeSchema::Slice { element } => deserializer
                .deserialize_seq(visitors::SliceVisitor::<SF, VF>::new(element, resolver)),
            TypeSchema::Map { key, value } => {
                deserializer
                    .deserialize_map(visitors::MapVisitor::<SF, VF>::new(key, value, resolver))
            }
            TypeSchema::Tuple { elements } => deserializer.deserialize_tuple(
                elements.len(),
                visitors::TupleVisitor::<SF, VF>::new(elements, resolver),
            ),

            TypeSchema::UnitStruct { name } => deserializer
                .deserialize_unit_struct("", visitors::UnitStructVisitor::<SF, VF>::new(name)),

            TypeSchema::NewTypeStruct { name, field } => {
                let entry = visitors::Resolver::new(name, self, resolver);
                deserializer.deserialize_newtype_struct(
                    "",
                    visitors::NewTypeStructVisitor::<SF, VF>::new(name, field, Some(&entry)),
                )
            }
            TypeSchema::TupleStruct { name, fields } => {
                let entry = visitors::Resolver::new(name, self, resolver);
                deserializer.deserialize_tuple_struct(
                    "",
                    fields.len(),
                    visitors::TupleStructVisitor::<SF, VF>::new(name, fields, Some(&entry)),
                )
            }
            TypeSchema::Struct { name, fields } => {
                let entry = visitors::Resolver::new(name, self, resolver);
                deserializer.deserialize_struct(
                    "", // dunno
                    // Cannot send empty list as postcard uses the length to encode
                    visitors::names(fields.len()), // dirty ass hack
                    visitors::StructVisitor::<SF, VF>::new(name, fields, Some(&entry)),
                )
            }

            TypeSchema::Enum { name, variants } => {
                let entry = visitors::Resolver::new(name, self, resolver);
                deserializer.deserialize_enum(
                    "", // dunno
                    // Cannot send empty list as postcard uses the length to encode
                    visitors::names(variants.len()), // dirty ass hack
                    visitors::EnumVisitor::<SF, VF>::new(name, variants, Some(&entry)),
                )
            }

            TypeSchema::Option(schema) => deserializer
                .deserialize_option(visitors::OptionVisitor::<SF, VF>::new(schema, resolver)),
        }
    }
}
