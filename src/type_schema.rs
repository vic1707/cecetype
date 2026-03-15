mod flavor;
mod primitive_impls;

pub use flavor::*;

#[derive(Debug)]
pub enum TypeSchema<'s, F: SchemaFlavor<'s>> {
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
        element: F::Ptr<TypeSchema<'s, F>>,
        len: usize,
    },

    Slice {
        element: F::Ptr<TypeSchema<'s, F>>,
    },

    Struct(F::Ptr<StructSchema<'s, F>>),
    Enum(F::Ptr<EnumSchema<'s, F>>),
}

#[derive(Debug)]
pub struct StructSchema<'s, F: flavor::SchemaFlavor<'s>> {
    pub name: F::Str,
    pub fields: F::List<FieldSchema<'s, F>>,
}

#[derive(Debug)]
pub struct FieldSchema<'s, F: flavor::SchemaFlavor<'s>> {
    pub name: F::Str,
    pub key: u32,
    pub ty: F::Ptr<TypeSchema<'s, F>>,
}

#[derive(Debug)]
pub struct EnumSchema<'s, F: flavor::SchemaFlavor<'s>> {
    pub name: F::Str,
    pub variants: F::List<VariantSchema<'s, F>>,
}

#[derive(Debug)]
pub struct VariantSchema<'s, F: flavor::SchemaFlavor<'s>> {
    pub name: F::Str,
    pub key: u32,
    pub payload: Option<F::Ptr<TypeSchema<'s, F>>>,
}

impl<'s, F> core::fmt::Display for TypeSchema<'s, F>
where
    F: SchemaFlavor<'s>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use core::ops::Deref as _;

        match self {
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

        writeln!(f, "struct {} {{", &*self.name)?;

        for field in self.fields.deref() {
            writeln!(f, "  {}: {},", &*field.name, field.ty.deref())?;
        }

        write!(f, "}}")
    }
}

impl<'s, F> core::fmt::Display for EnumSchema<'s, F>
where
    F: SchemaFlavor<'s>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use core::ops::Deref as _;

        writeln!(f, "enum {} {{", &*self.name)?;

        for variant in self.variants.deref() {
            match &variant.payload {
                Some(payload) => {
                    writeln!(
                        f,
                        "  {}: {}({}),",
                        variant.key,
                        &*variant.name,
                        payload.deref()
                    )?;
                }
                None => {
                    writeln!(f, "  {}: {},", variant.key, &*variant.name,)?;
                }
            }
        }

        write!(f, "}}")
    }
}
