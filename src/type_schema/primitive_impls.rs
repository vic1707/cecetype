use crate::{EnumSchema, Schema, StaticSchema, TypeSchema, VariantSchema};

macro_rules! primitive_schema {
    ($ty:ty, $variant:ident) => {
        impl Schema for $ty {
            const SCHEMA: &'static StaticSchema = &TypeSchema::$variant;
        }
    };
}

primitive_schema!((), Unit);
primitive_schema!(bool, Bool);
primitive_schema!(u8, U8);
primitive_schema!(u16, U16);
primitive_schema!(u32, U32);
primitive_schema!(u64, U64);
primitive_schema!(i8, I8);
primitive_schema!(i16, I16);
primitive_schema!(i32, I32);
primitive_schema!(i64, I64);
primitive_schema!(f32, F32);
primitive_schema!(f64, F64);
primitive_schema!(char, Char);

impl<T: Schema, const N: usize> Schema for [T; N] {
    const SCHEMA: &'static StaticSchema = &TypeSchema::Array {
        element: T::SCHEMA,
        len: N,
    };
}

impl<T: Schema> Schema for [T] {
    const SCHEMA: &'static StaticSchema = &TypeSchema::Slice {
        element: T::SCHEMA,
    };
}

impl<T: Schema> Schema for &T {
    const SCHEMA: &'static StaticSchema = T::SCHEMA;
}

impl<T: Schema> Schema for &mut T {
    const SCHEMA: &'static StaticSchema = T::SCHEMA;
}

impl<T: Schema, E: Schema> Schema for Result<T, E> {
    const SCHEMA: &'static StaticSchema = &TypeSchema::Enum(&EnumSchema {
        name: "Result",
        variants: &[
            VariantSchema {
                name: "Ok",
                key: 0, // TODO: generate
                payload: Some(T::SCHEMA),
            },
            VariantSchema {
                name: "Err",
                key: 1, // TODO: generate
                payload: Some(E::SCHEMA),
            },
        ] as &[VariantSchema<_>],
    });
}

impl<T: Schema> Schema for Option<T> {
    const SCHEMA: &'static StaticSchema = &TypeSchema::Enum(&EnumSchema {
        name: "Option",
        variants: &[
            VariantSchema {
                name: "Some",
                key: 0, // TODO: generate
                payload: Some(T::SCHEMA),
            },
            VariantSchema {
                name: "None",
                key: 1, // TODO: generate
                payload: None,
            },
        ] as &[VariantSchema<_>],
    });
}
