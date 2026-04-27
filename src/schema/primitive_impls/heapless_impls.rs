use crate::{Schema, StaticSchema, schema};
use ::{
    hash32::BuildHasherDefault,
    heapless::{
        BinaryHeap, Deque, HistoryBuf, IndexMap, IndexSet, LinearMap, String, Vec,
        binary_heap::Kind as BinaryHeapKind,
    },
};

impl<const N: usize> Schema for String<N> {
    const SCHEMA: &'static StaticSchema = &schema::Schema::Str;
}

impl<T: Schema, const N: usize> Schema for Vec<T, N> {
    const SCHEMA: &'static StaticSchema = &schema::Schema::Slice { element: T::SCHEMA };
}

impl<T: Schema, const N: usize> Schema for Deque<T, N> {
    const SCHEMA: &'static StaticSchema = &schema::Schema::Slice { element: T::SCHEMA };
}

impl<T: Schema, KIND: BinaryHeapKind, const N: usize> Schema for BinaryHeap<T, KIND, N> {
    const SCHEMA: &'static StaticSchema = &schema::Schema::Slice { element: T::SCHEMA };
}

impl<T: Schema, const N: usize> Schema for HistoryBuf<T, N> {
    const SCHEMA: &'static StaticSchema = &schema::Schema::Slice { element: T::SCHEMA };
}

impl<K: Schema, V: Schema, S, const N: usize> Schema for IndexMap<K, V, BuildHasherDefault<S>, N> {
    const SCHEMA: &'static StaticSchema = &schema::Schema::Map {
        key: K::SCHEMA,
        value: V::SCHEMA,
    };
}

impl<T: Schema, S, const N: usize> Schema for IndexSet<T, BuildHasherDefault<S>, N> {
    const SCHEMA: &'static StaticSchema = &schema::Schema::Slice { element: T::SCHEMA };
}

impl<K: Schema, V: Schema, const N: usize> Schema for LinearMap<K, V, N> {
    const SCHEMA: &'static StaticSchema = &schema::Schema::Map {
        key: K::SCHEMA,
        value: V::SCHEMA,
    };
}
