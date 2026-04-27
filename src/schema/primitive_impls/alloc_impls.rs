use super::{map_like, passthrough_schemas, primitive_schema, slice_like};
use crate::{Schema, StaticSchema, schema};
use ::alloc::{
    borrow::{Cow, ToOwned},
    boxed::Box,
    collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque},
    string::String,
    vec::Vec,
};

primitive_schema!(String, Str);
primitive_schema!(Cow<'_, str>, Str);

passthrough_schemas!(Box<T>);

slice_like!(
    Vec<T>,
    VecDeque<T>,
    LinkedList<T>,
    BinaryHeap<T>,
    BTreeSet<T>,
);

map_like!(BTreeMap<K, V>);

impl<T: Schema + ToOwned> Schema for Cow<'_, [T]>
where
    [T]: ToOwned,
{
    const SCHEMA: &'static StaticSchema = &schema::Schema::Slice { element: T::SCHEMA };
}
