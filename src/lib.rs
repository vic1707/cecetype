#![no_std]

extern crate self as schema;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod flavors;
mod type_schema;
mod value;

pub use self::{
    flavors::{
        ser, Borrowed, Owned, OwnedSchemaFlavor, SchemaFlavor, Static, ValueBuilder, ValueFlavor,
    },
    type_schema::{Data, FieldSchema, RefKind, TypeSchema},
    value::{Data as ValueData, Value},
};
pub use ::schema_macros::Schema;

pub trait Schema {
    const SCHEMA: &'static StaticSchema;
}

pub type OwnedSchema<'s> = TypeSchema<'s, Owned>;
pub type OwnedValue = Value<Owned>;

pub type BorrowedSchema<'s> = TypeSchema<'s, Borrowed>;
pub type StaticSchema = TypeSchema<'static, Static>;

#[cfg(test)]
mod tests {
    use super::*;
    use ::{
        core::fmt,
        serde::{de::DeserializeOwned, Deserialize, Serialize},
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
