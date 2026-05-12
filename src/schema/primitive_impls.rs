#![expect(clippy::needless_pub_self, reason = "false positive")]

#[cfg(feature = "alloc")]
mod alloc_impls;
#[cfg(feature = "heapless")]
mod heapless_impls;
#[cfg(feature = "std")]
mod std_impls;

use crate::{Schema, StaticSchema, schema};
use ::core::{
    cell::{Cell, RefCell},
    convert::Infallible,
    marker::PhantomData,
    num::{
        NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128, NonZeroIsize, NonZeroU8,
        NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128, NonZeroUsize, Saturating, Wrapping,
    },
    time::Duration,
};

primitive_schema!((), Unit);
primitive_schema!(bool, Bool);
primitive_schema!(u8, U8);
primitive_schema!(NonZeroU8, U8);
primitive_schema!(u16, U16);
primitive_schema!(NonZeroU16, U16);
primitive_schema!(u32, U32);
primitive_schema!(NonZeroU32, U32);
primitive_schema!(u64, U64);
primitive_schema!(NonZeroU64, U64);
primitive_schema!(u128, U128);
primitive_schema!(NonZeroU128, U128);
primitive_schema!(usize, U64); // serde always encodes usize as u64, with safe try_from overflow checking on decode
primitive_schema!(NonZeroUsize, U64);
primitive_schema!(i8, I8);
primitive_schema!(NonZeroI8, I8);
primitive_schema!(i16, I16);
primitive_schema!(NonZeroI16, I16);
primitive_schema!(i32, I32);
primitive_schema!(NonZeroI32, I32);
primitive_schema!(i64, I64);
primitive_schema!(NonZeroI64, I64);
primitive_schema!(i128, I128);
primitive_schema!(NonZeroI128, I128);
primitive_schema!(isize, I64); // serde always encodes isize as i64, with safe try_from overflow checking on decode
primitive_schema!(NonZeroIsize, I64);
primitive_schema!(f32, F32);
primitive_schema!(f64, F64);
primitive_schema!(&str, Str);
primitive_schema!(char, Char);

impl_tuple_schema!(L, K, J, I, H, G, F, E, D, C, B, A);

passthrough_schemas!(Wrapping<T>, Saturating<T>, Cell<T>, RefCell<T>, &T, &mut T);

impl<T: Schema> Schema for [T] {
    const SCHEMA: &'static StaticSchema = &schema::Schema::Slice { element: T::SCHEMA };
}

impl<T: Schema> Schema for &[T] {
    const SCHEMA: &'static StaticSchema = <[T] as Schema>::SCHEMA;
}

impl<T: ?Sized> Schema for PhantomData<T> {
    const SCHEMA: &'static StaticSchema = &schema::Schema::Struct {
        name: "PhantomData",
        data: schema::Data::Unit,
    };
}

impl Schema for Duration {
    const SCHEMA: &'static StaticSchema = &schema::Schema::Struct {
        name: "Duration",
        data: schema::Data::Struct {
            fields: &[
                &schema::FieldSchema {
                    name: "secs",
                    ty: &schema::Schema::U64,
                },
                &schema::FieldSchema {
                    name: "nanos",
                    ty: &schema::Schema::U32,
                },
            ],
        },
    };
}

impl<T: Schema, const N: usize> Schema for [T; N] {
    const SCHEMA: &'static StaticSchema = &schema::Schema::Array {
        element: T::SCHEMA,
        len: N,
    };
}

const OK_DISCRIMINANT: u32 = 0;
const ERR_DISCRIMINANT: u32 = 1;
impl<T: Schema, E: Schema> Schema for Result<T, E> {
    const SCHEMA: &'static StaticSchema = &schema::Schema::Enum {
        name: "Result",
        variants: &[
            &schema::VariantSchema {
                discriminant: OK_DISCRIMINANT,
                name: "Ok",
                data: schema::Data::NewType { field: T::SCHEMA },
            },
            &schema::VariantSchema {
                discriminant: ERR_DISCRIMINANT,
                name: "Err",
                data: schema::Data::NewType { field: E::SCHEMA },
            },
        ],
    };
}

impl Schema for Infallible {
    const SCHEMA: &'static StaticSchema = &schema::Schema::Enum {
        name: "Infallible",
        variants: &[],
    };
}

impl<T: Schema> Schema for Option<T> {
    const SCHEMA: &'static StaticSchema = &schema::Schema::Option(T::SCHEMA);
}

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
            unsafe { *(ptr::from_ref::<Result<(), ()>>(&Result::Ok(())) as *const u8) } as u32 == OK_DISCRIMINANT
        };
        assert! {
            unsafe { *(ptr::from_ref::<Result<(), ()>>(&Result::Err(())) as *const u8) } as u32 == ERR_DISCRIMINANT
        };
    };
}

mod macros {
    macro_rules! primitive_schema {
        ($ty:ty, $variant:ident) => {
            impl Schema for $ty {
                const SCHEMA: &'static StaticSchema = &schema::Schema::$variant;
            }
        };
    }

    macro_rules! passthrough_schemas {
        ($($wrapper:ty),* $(,)?) => {
            $(
                impl<T: Schema> Schema for $wrapper {
                    const SCHEMA: &'static StaticSchema = T::SCHEMA;
                }
            )*
        };
    }

    macro_rules! impl_tuple_schema {
        () => {};
        ($head:ident $(, $tail:ident)*) => {
            impl<$head: Schema $(, $tail: Schema)*> Schema for ($head, $($tail,)*) {
                const SCHEMA: &'static StaticSchema = &schema::Schema::Tuple {
                    elements: &[
                        $head::SCHEMA,
                        $($tail::SCHEMA),*
                    ],
                };
            }

            impl_tuple_schema!($($tail),*);
        };
    }

    macro_rules! slice_like {
        ($($ty:ty),* $(,)?) => {
            $(
                impl<T: Schema> Schema for $ty {
                    const SCHEMA: &'static StaticSchema =
                        &schema::Schema::Slice { element: T::SCHEMA };
                }
            )*
        };
    }

    macro_rules! map_like {
        ($($ty:ty),* $(,)?) => {
            $(
                impl<K: Schema, V: Schema> Schema for $ty {
                    const SCHEMA: &'static StaticSchema = &schema::Schema::Map {
                        key: K::SCHEMA,
                        value: V::SCHEMA,
                    };
                }
            )*
        };
    }

    pub(super) use {
        impl_tuple_schema, map_like, passthrough_schemas, primitive_schema, slice_like,
    };
}

pub(self) use macros::{
    impl_tuple_schema, map_like, passthrough_schemas, primitive_schema, slice_like,
};
