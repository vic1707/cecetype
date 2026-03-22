use crate::{Schema, StaticSchema, TypeSchema, VariantSchema};

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
primitive_schema!(&str, Str);
primitive_schema!(char, Char);
#[cfg(feature = "std")]
primitive_schema!(::std::string::String, Str);

impl<T: Schema, const N: usize> Schema for [T; N] {
    const SCHEMA: &'static StaticSchema = &TypeSchema::Array {
        element: T::SCHEMA,
        len: N,
    };
}

impl<T: Schema> Schema for [T] {
    const SCHEMA: &'static StaticSchema = &TypeSchema::Slice { element: T::SCHEMA };
}

impl<T: Schema> Schema for &[T] {
    const SCHEMA: &'static StaticSchema = &TypeSchema::Slice { element: T::SCHEMA };
}

impl<T: Schema> Schema for &T {
    const SCHEMA: &'static StaticSchema = T::SCHEMA;
}

impl<T: Schema> Schema for &mut T {
    const SCHEMA: &'static StaticSchema = T::SCHEMA;
}

impl<T: Schema, E: Schema> Schema for Result<T, E> {
    const SCHEMA: &'static StaticSchema = &TypeSchema::Enum {
        name: "Result",
        variants: &[
            &VariantSchema::NewType {
                name: "Ok",
                discriminant: 0, // TODO: generate
                field: T::SCHEMA,
            },
            &VariantSchema::NewType {
                name: "Err",
                discriminant: 1, // TODO: generate
                field: E::SCHEMA,
            },
        ],
    };
}

impl<T: Schema> Schema for Option<T> {
    const SCHEMA: &'static StaticSchema = &TypeSchema::Option(T::SCHEMA);
}

macro_rules! impl_tuple_schema {
    () => {};
    ($head:ident $(, $tail:ident)*) => {
        impl<$head: Schema $(, $tail: Schema)*> Schema for ($head, $($tail,)*) {
            const SCHEMA: &'static StaticSchema = &TypeSchema::Tuple {
                elements: &[
                    $head::SCHEMA,
                    $($tail::SCHEMA),*
                ],
            };
        }

        impl_tuple_schema!($($tail),*);
    };
}

impl_tuple_schema!(L, K, J, I, H, G, F, E, D, C, B, A);
