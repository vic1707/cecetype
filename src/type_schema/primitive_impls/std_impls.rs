use crate::{Schema, StaticSchema, TypeSchema};
use ::std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::{Mutex, RwLock},
};

impl<K: Schema, V: Schema, S> Schema for HashMap<K, V, S> {
    const SCHEMA: &'static StaticSchema = &TypeSchema::Map {
        key: K::SCHEMA,
        value: V::SCHEMA,
    };
}

impl<T: Schema, S> Schema for HashSet<T, S> {
    const SCHEMA: &'static StaticSchema = &TypeSchema::Slice { element: T::SCHEMA };
}

impl Schema for PathBuf {
    const SCHEMA: &'static StaticSchema = &TypeSchema::Str;
}

impl Schema for &Path {
    const SCHEMA: &'static StaticSchema = &TypeSchema::Str;
}

impl<T: Schema> Schema for Mutex<T> {
    const SCHEMA: &'static StaticSchema = T::SCHEMA;
}

impl<T: Schema> Schema for RwLock<T> {
    const SCHEMA: &'static StaticSchema = T::SCHEMA;
}
