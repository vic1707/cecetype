mod r#enum;
mod r#struct;
use super::Schema;
pub use self::{r#enum::*, r#struct::*};

#[derive(Debug)]
pub enum TypeSchema {
    Bool,
    U8,
    U16,
    U32,
    I32,
    I64,

    Array {
        element: &'static TypeSchema,
        len: usize,
    },

    Slice {
        element: &'static TypeSchema,
    },

    MapStruct(&'static MapStructSchema),
    ArrayStruct(&'static ArrayStructSchema),

    MapEnum(&'static MapEnumSchema),
    ArrayEnum(&'static ArrayEnumSchema),
}

macro_rules! primitive_schema {
    ($ty:ty, $variant:ident) => {
        impl Schema for $ty {
            const SCHEMA: &'static TypeSchema = &TypeSchema::$variant;
        }
    };
}

primitive_schema!(bool, Bool);
primitive_schema!(u8, U8);
primitive_schema!(u16, U16);
primitive_schema!(u32, U32);
primitive_schema!(i32, I32);
primitive_schema!(i64, I64);

impl<T: Schema, const N: usize> Schema for [T; N] {
    const SCHEMA: &'static TypeSchema = &TypeSchema::Array {
        element: T::SCHEMA,
        len: N,
    };
}

impl<T: Schema> Schema for [T] {
    const SCHEMA: &'static TypeSchema = &TypeSchema::Slice { element: T::SCHEMA };
}

impl<T: Schema> Schema for &T {
    const SCHEMA: &'static TypeSchema = T::SCHEMA;
}

impl<T: Schema> Schema for &mut T {
    const SCHEMA: &'static TypeSchema = T::SCHEMA;
}
