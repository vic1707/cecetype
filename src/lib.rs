#![no_std]

mod type_schema;

use self::type_schema::*;

pub trait Schema {
    const SCHEMA: &'static TypeSchema;
}
