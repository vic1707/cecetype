mod primitive_impls;

use crate::{OwnedSchemaFlavor, SchemaFlavor, flavors::ser};
use ::serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(bound(
    serialize = "F::Str: Serialize",
    deserialize = "F: OwnedSchemaFlavor<'s>, F::Str: Deserialize<'de>"
))]
pub enum TypeSchema<'s, F: SchemaFlavor<'s>> {
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

    Array {
        #[serde(serialize_with = "ser::serialize_ptr")]
        #[serde(deserialize_with = "F::deserialize_ptr")]
        element: F::Ptr<TypeSchema<'s, F>>,
        len: usize,
    },

    Slice {
        #[serde(serialize_with = "ser::serialize_ptr")]
        #[serde(deserialize_with = "F::deserialize_ptr")]
        element: F::Ptr<TypeSchema<'s, F>>,
    },

    Tuple {
        #[serde(serialize_with = "ser::serialize_list")]
        #[serde(deserialize_with = "F::deserialize_list")]
        elements: F::List<TypeSchema<'s, F>>,
    },

    Struct(
        #[serde(serialize_with = "ser::serialize_ptr")]
        #[serde(deserialize_with = "F::deserialize_ptr")]
        F::Ptr<StructSchema<'s, F>>,
    ),

    Enum(
        #[serde(serialize_with = "ser::serialize_ptr")]
        #[serde(deserialize_with = "F::deserialize_ptr")]
        F::Ptr<EnumSchema<'s, F>>,
    ),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(bound(
    serialize = "F::Str: Serialize",
    deserialize = "F: OwnedSchemaFlavor<'s>, F::Str: Deserialize<'de>"
))]
pub struct StructSchema<'s, F: SchemaFlavor<'s>> {
    pub name: F::Str,
    #[serde(serialize_with = "ser::serialize_list")]
    #[serde(deserialize_with = "F::deserialize_list")]
    pub fields: F::List<FieldSchema<'s, F>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(bound(
    serialize = "F::Str: Serialize",
    deserialize = "F: OwnedSchemaFlavor<'s>, F::Str: Deserialize<'de>"
))]
pub struct FieldSchema<'s, F: SchemaFlavor<'s>> {
    pub name: F::Str,
    pub key: u32,
    #[serde(serialize_with = "ser::serialize_ptr")]
    #[serde(deserialize_with = "F::deserialize_ptr")]
    pub ty: F::Ptr<TypeSchema<'s, F>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(bound(
    serialize = "F::Str: Serialize",
    deserialize = "F: OwnedSchemaFlavor<'s>, F::Str: Deserialize<'de>"
))]
pub struct EnumSchema<'s, F: SchemaFlavor<'s>> {
    pub name: F::Str,
    #[serde(serialize_with = "ser::serialize_list")]
    #[serde(deserialize_with = "F::deserialize_list")]
    pub variants: F::List<VariantSchema<'s, F>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(bound(
    serialize = "F::Str: Serialize",
    deserialize = "F: OwnedSchemaFlavor<'s>, F::Str: Deserialize<'de>"
))]
pub enum VariantSchema<'s, F: SchemaFlavor<'s>> {
    Unit {
        name: F::Str,
        discriminant: i32,
    },
    Tuple {
        name: F::Str,
        discriminant: i32,
        #[serde(serialize_with = "ser::serialize_list")]
        #[serde(deserialize_with = "F::deserialize_list")]
        fields: F::List<TypeSchema<'s, F>>,
    },
    Struct {
        name: F::Str,
        discriminant: i32,
        #[serde(serialize_with = "ser::serialize_list")]
        #[serde(deserialize_with = "F::deserialize_list")]
        fields: F::List<FieldSchema<'s, F>>,
    },
}

impl<'s, F> core::fmt::Display for TypeSchema<'s, F>
where
    F: SchemaFlavor<'s>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use core::ops::Deref as _;

        match self {
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

            TypeSchema::Array { element, len } => {
                write!(f, "[{}; {}]", element.deref(), len)
            }

            TypeSchema::Slice { element } => {
                write!(f, "[{}]", element.deref())
            }

            TypeSchema::Tuple { elements } => {
                write!(f, "(")?;

                for (i, elem) in elements.deref().iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", elem.deref())?;
                }

                write!(f, ")")
            }

            TypeSchema::Struct(s) => write!(f, "{}", s.deref()),

            TypeSchema::Enum(e) => write!(f, "{}", e.deref()),
        }
    }
}

impl<'s, F> core::fmt::Display for StructSchema<'s, F>
where
    F: SchemaFlavor<'s>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use core::ops::Deref as _;

        write!(f, "{} {{ ", &*self.name)?;

        for (idx, field) in self.fields.deref().iter().enumerate() {
            if idx != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}: {}", &*field.name, field.ty.deref())?;
        }

        write!(f, " }}")
    }
}

impl<'s, F> core::fmt::Display for EnumSchema<'s, F>
where
    F: SchemaFlavor<'s>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use core::ops::Deref as _;

        write!(f, "{} {{ ", &*self.name)?;

        for (idx, variant) in self.variants.deref().iter().enumerate() {
            if idx != 0 {
                write!(f, " | ")?;
            }
            match &**variant {
                VariantSchema::Unit { name, discriminant } => {
                    write!(f, "{} = {}", &**name, discriminant)?
                }
                VariantSchema::Struct {
                    name,
                    discriminant,
                    fields,
                } => {
                    write!(f, "{} = {}({{ ", &**name, discriminant)?;
                    for (idx, field) in fields.deref().iter().enumerate() {
                        if idx != 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}: {}", &*field.name, field.ty.deref())?;
                    }
                    write!(f, " }})")?;
                }
                VariantSchema::Tuple {
                    name,
                    discriminant,
                    fields,
                } => {
                    write!(f, "{} = {}(", &**name, discriminant)?;
                    for (idx, field) in fields.deref().iter().enumerate() {
                        if idx != 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", field.deref())?;
                    }
                    write!(f, ")")?;
                }
            }
        }

        write!(f, " }}")
    }
}
