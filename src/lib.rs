#![no_std]

#[cfg(feature = "std")]
extern crate std;

mod type_schema;

use self::type_schema::*;

pub trait Schema {
    const SCHEMA: &'static StaticSchema;
}

#[cfg(feature = "std")]
pub type OwnedSchema<'s> = TypeSchema<'s, Owned>;

pub type BorrowedSchema<'s> = TypeSchema<'s, Borrowed>;
pub type StaticSchema = TypeSchema<'static, Static>;

#[cfg(test)]
mod tests {
    use super::*;
    use ::serde::{Deserialize, Serialize, de::DeserializeOwned};

    const _: fn() = || {
        fn assert_impl<T: Serialize>() {}
        assert_impl::<StaticSchema>();
        assert_impl::<BorrowedSchema>();
    };

    #[cfg(feature = "std")]
    const _: fn() = || {
        fn assert_impl<T: for <'de> Deserialize<'de> + DeserializeOwned + Serialize>() {}
        assert_impl::<OwnedSchema>();
    };
}
