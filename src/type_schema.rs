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
