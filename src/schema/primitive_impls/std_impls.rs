use super::{passthrough_schemas, primitive_schema};
use crate::{Schema, StaticSchema, schema};
use ::std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::{Mutex, RwLock},
};

primitive_schema!(PathBuf, Str);
primitive_schema!(&Path, Str);

passthrough_schemas!(Mutex<T>, RwLock<T>);

impl<K: Schema, V: Schema, S> Schema for HashMap<K, V, S> {
    const SCHEMA: &'static StaticSchema = &schema::Schema::Map {
        key: K::SCHEMA,
        value: V::SCHEMA,
    };
}

impl<T: Schema, S> Schema for HashSet<T, S> {
    const SCHEMA: &'static StaticSchema = &schema::Schema::Slice { element: T::SCHEMA };
}
