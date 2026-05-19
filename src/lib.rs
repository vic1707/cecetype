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

/// Compile-time schema for a Rust type.
///
/// Schemas are plain data. Derived schemas are `&'static` and can be
/// used in `no_std` code without allocation.
///
/// Implement this trait with `#[derive(Schema)]` or manually:
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
    /// Static schema describing `Self`.
    const SCHEMA: &'static StaticSchema;
}

/// Schema storage using `Box<T>`, `Vec<T>`, and `String`.
#[cfg(feature = "alloc")]
pub type OwnedSchema<'s> = schema::Schema<'s, flavors::Owned>;
/// Dynamic value storage using `Box<T>`, `Vec<T>`, and `String`.
#[cfg(feature = "alloc")]
pub type OwnedValue = value::Value<flavors::Owned>;

/// Schema storage borrowing from an input lifetime.
pub type BorrowedSchema<'s> = schema::Schema<'s, flavors::Borrowed>;
/// Schema storage for static, zero-allocation schemas.
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

    #[cfg(feature = "cli")]
    implements! {
        self::parse::cli::spec::Spec<flavors::Static>: (Schema + Serialize);
        self::parse::cli::spec::Spec<flavors::Owned>: (Schema + Serialize + for <'de> Deserialize<'de> + DeserializeOwned);
    }
}
