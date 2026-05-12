#![cfg_attr(doc, doc = include_str!("../README.md"))]
#![no_std]

extern crate self as cecetype;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

pub mod flavors;
pub mod parse;
pub mod schema;
mod utils;
pub mod value;

#[cfg(feature = "derive")]
pub use ::cecetype_macros::Schema;

/// Schema definition for a type.
///
/// Implement via `#[derive(Schema)]` or manually:
/// ```
/// use cecetype::{Schema, StaticSchema, schema::Schema as S};
///
/// struct Point { x: f32, y: f32 }
///
/// impl Schema for Point {
///     const SCHEMA: &'static StaticSchema = &S::Struct {
///         name: "Point",
///         data: cecetype::schema::Data::Struct {
///             fields: &[
///                 &cecetype::schema::FieldSchema { name: "x", ty: &S::F32 },
///                 &cecetype::schema::FieldSchema { name: "y", ty: &S::F32 },
///             ],
///         },
///     };
/// }
/// ```
pub trait Schema {
    const SCHEMA: &'static StaticSchema;
}

/// Schema with `Box<T>` / `Vec<T>` / `String` (requires `alloc`).
pub type OwnedSchema<'s> = schema::Schema<'s, flavors::Owned>;
/// Value with owned storage (requires `alloc`).
pub type OwnedValue = value::Value<flavors::Owned>;

/// Schema with `&'s T` / `&'s [&'s T]` borrowing from input.
pub type BorrowedSchema<'s> = schema::Schema<'s, flavors::Borrowed>;
/// Schema with `&'static T` / `&'static [&'static T]` (zero-copy).
pub type StaticSchema = schema::Schema<'static, flavors::Static>;

#[cfg(test)]
mod tests {
    use super::*;
    use ::{
        core::fmt,
        serde::{Deserialize, Serialize, de::DeserializeOwned},
    };

    macro_rules! implements {
        ( $(
            $(#[$meta:meta])*
            $ty:ty : ($($bounds:tt)+) ;
        )+ ) => {
            $(
                $(#[$meta])*
                const _: fn() = || {
                    fn assert_impl<T: $($bounds)+>() {}
                    assert_impl::<$ty>();
                };
            )+
        };
    }

    implements! {
        StaticSchema: (Schema + Clone + fmt::Display + fmt::Debug + PartialEq + Serialize);
        BorrowedSchema: (Schema + Clone + fmt::Display + fmt::Debug + PartialEq + Serialize);
    }

    #[cfg(feature = "alloc")]
    implements! {
        OwnedSchema: (Schema + Clone + fmt::Display + fmt::Debug + PartialEq + Serialize + for <'de> Deserialize<'de> + DeserializeOwned);
        OwnedValue: (fmt::Display + fmt::Debug + PartialEq + Serialize);
    }
}
