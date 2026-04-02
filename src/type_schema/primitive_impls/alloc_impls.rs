use crate::{Schema, StaticSchema, TypeSchema};
use ::alloc::{
    borrow::Cow,
    boxed::Box,
    collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque},
    string::String,
    vec::Vec,
};

impl Schema for String {
    const SCHEMA: &'static StaticSchema = &TypeSchema::Str;
}

impl<T: Schema> Schema for Box<T> {
    const SCHEMA: &'static StaticSchema = T::SCHEMA;
}

impl<T: Schema> Schema for Vec<T> {
    const SCHEMA: &'static StaticSchema = &TypeSchema::Slice { element: T::SCHEMA };
}

impl<K: Schema, V: Schema> Schema for BTreeMap<K, V> {
    const SCHEMA: &'static StaticSchema = &TypeSchema::Map {
        key: K::SCHEMA,
        value: V::SCHEMA,
    };
}

impl<T: Schema> Schema for BTreeSet<T> {
    const SCHEMA: &'static StaticSchema = &TypeSchema::Slice { element: T::SCHEMA };
}

impl<T: Schema> Schema for BinaryHeap<T> {
    const SCHEMA: &'static StaticSchema = &TypeSchema::Slice { element: T::SCHEMA };
}

impl<T: Schema> Schema for LinkedList<T> {
    const SCHEMA: &'static StaticSchema = &TypeSchema::Slice { element: T::SCHEMA };
}

impl<T: Schema> Schema for VecDeque<T> {
    const SCHEMA: &'static StaticSchema = &TypeSchema::Slice { element: T::SCHEMA };
}

impl Schema for Cow<'_, str> {
    const SCHEMA: &'static StaticSchema = &TypeSchema::Str;
}

impl<T: Schema + Clone> Schema for Cow<'_, [T]> {
    const SCHEMA: &'static StaticSchema = &TypeSchema::Slice { element: T::SCHEMA };
}
