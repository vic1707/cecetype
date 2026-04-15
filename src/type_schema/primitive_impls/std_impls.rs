use super::{map_like, passthrough_schemas, primitive_schema, slice_like};
use crate::{Schema, StaticSchema, TypeSchema};
use ::std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::{Mutex, RwLock},
};

primitive_schema!(PathBuf, Str);
primitive_schema!(&Path, Str);

passthrough_schemas!(Mutex<T>, RwLock<T>);

impl<K: Schema, V: Schema, S> Schema for HashMap<K, V, S> {
    const SCHEMA: &'static StaticSchema = &TypeSchema::Map {
        key: K::SCHEMA,
        value: V::SCHEMA,
    };
}

impl<T: Schema, S> Schema for HashSet<T, S> {
    const SCHEMA: &'static StaticSchema = &TypeSchema::Slice { element: T::SCHEMA };
}
