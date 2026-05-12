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

pub use ::cecetype_macros::Schema;

pub trait Schema {
    const SCHEMA: &'static StaticSchema;
}

pub type OwnedSchema<'s> = schema::Schema<'s, flavors::Owned>;
pub type OwnedValue = value::Value<flavors::Owned>;

pub type BorrowedSchema<'s> = schema::Schema<'s, flavors::Borrowed>;
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
        StaticSchema: (Clone + fmt::Display + fmt::Debug + PartialEq + Serialize);
        BorrowedSchema: (Clone + fmt::Display + fmt::Debug + PartialEq + Serialize);
    }

    #[cfg(feature = "alloc")]
    implements! {
        OwnedSchema: (Clone + fmt::Display + fmt::Debug + PartialEq + Serialize + for <'de> Deserialize<'de> + DeserializeOwned);
        OwnedValue: (fmt::Display + fmt::Debug + PartialEq + Serialize);
    }

    implements! {
        StaticSchema: (Schema);
        BorrowedSchema: (Schema);
    }

    #[cfg(feature = "alloc")]
    implements! {
        OwnedSchema: (Schema);
    }
}
