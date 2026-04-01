use crate::{Schema, StaticSchema, TypeSchema, VariantSchema};
#[cfg(feature = "alloc")]
use ::alloc::string::String;

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
primitive_schema!(u128, U128);
primitive_schema!(usize, U64); // serde always encodes usize as u64, with safe try_from overflow checking on decode
primitive_schema!(i8, I8);
primitive_schema!(i16, I16);
primitive_schema!(i32, I32);
primitive_schema!(i64, I64);
primitive_schema!(i128, I128);
primitive_schema!(isize, I64); // serde always encodes isize as i64, with safe try_from overflow checking on decode
primitive_schema!(f32, F32);
primitive_schema!(f64, F64);
primitive_schema!(&str, Str);
primitive_schema!(char, Char);
#[cfg(feature = "alloc")]
primitive_schema!(String, Str);

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
    const SCHEMA: &'static StaticSchema = <[T] as Schema>::SCHEMA;
}

impl<T: Schema> Schema for &T {
    const SCHEMA: &'static StaticSchema = T::SCHEMA;
}

impl<T: Schema> Schema for &mut T {
    const SCHEMA: &'static StaticSchema = T::SCHEMA;
}

const OK_DISCRIMINANT: u32 = 0;
const ERR_DISCRIMINANT: u32 = 1;
impl<T: Schema, E: Schema> Schema for Result<T, E> {
    const SCHEMA: &'static StaticSchema = &TypeSchema::Enum {
        name: "Result",
        variants: &[
            &VariantSchema::NewType {
                name: "Ok",
                discriminant: OK_DISCRIMINANT,
                field: T::SCHEMA,
            },
            &VariantSchema::NewType {
                name: "Err",
                discriminant: ERR_DISCRIMINANT,
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

#[cfg(test)]
mod tests {
    #![expect(
        clippy::as_conversions,
        clippy::undocumented_unsafe_blocks,
        clippy::ptr_as_ptr,
        reason = "testing"
    )]
    use super::*;
    use ::core::ptr;

    const _: () = {
        assert! {
            unsafe { *(ptr::from_ref::<Result<(), ()>>(&Result::Ok(())) as *const u8) } as u32  == OK_DISCRIMINANT
        };
        assert! {
            unsafe { *(ptr::from_ref::<Result<(), ()>>(&Result::Err(())) as *const u8) } as u32  == ERR_DISCRIMINANT
        };
    };
}
