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
