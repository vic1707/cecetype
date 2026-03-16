#![no_std]

#[cfg(feature = "std")]
extern crate std;

mod flavors;
mod type_schema;
mod value;

use self::{flavors::*, type_schema::*, value::*};

pub trait Schema {
    const SCHEMA: &'static StaticSchema;
}

pub type OwnedSchema<'s> = TypeSchema<'s, Owned>;

pub type BorrowedSchema<'s> = TypeSchema<'s, Borrowed>;
pub type StaticSchema = TypeSchema<'static, Static>;

#[cfg(test)]
mod tests {
    use super::*;
    use ::serde::{Deserialize, Serialize, de::DeserializeOwned};

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
        StaticSchema: (Serialize);

        BorrowedSchema: (Serialize);

        #[cfg(feature = "std")]
        OwnedSchema: (Serialize + for <'de> Deserialize<'de> + DeserializeOwned);
    }
}
