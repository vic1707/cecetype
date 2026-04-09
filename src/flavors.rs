mod borrowed;
mod owned;

use ::core::{
    fmt,
    ops::{Deref, DerefMut},
};

pub use self::{
    borrowed::{Borrowed, Static},
    owned::Owned,
};

pub trait SchemaFlavor<'s>
where
    Self: 's,
{
    type Ptr<T: 's + Clone + PartialEq + fmt::Debug>: Deref<Target = T>
        + Clone
        + PartialEq
        + fmt::Debug;
    type List<T: 's + Clone + PartialEq + fmt::Debug>: Deref<Target = [Self::Ptr<T>]>
        + Clone
        + PartialEq
        + fmt::Debug;
    type Str: AsRef<str> + Clone + PartialEq + fmt::Debug;
}

pub trait OwnedSchemaFlavor<'s>: SchemaFlavor<'s> {
    fn deserialize_ptr<'de, D, T>(deserializer: D) -> Result<Self::Ptr<T>, D::Error>
    where
        D: ::serde::Deserializer<'de>,
        T: ::serde::Deserialize<'de> + Clone + PartialEq + fmt::Debug;

    fn deserialize_list<'de, D, T>(deserializer: D) -> Result<Self::List<T>, D::Error>
    where
        D: ::serde::Deserializer<'de>,
        T: ::serde::Deserialize<'de> + Clone + PartialEq + fmt::Debug;
}

pub trait ValueFlavor {
    type Ptr<T: PartialEq + fmt::Debug>: Deref<Target = T> + PartialEq + fmt::Debug;
    type List<T: PartialEq + fmt::Debug>: DerefMut<Target = [T]> + PartialEq + fmt::Debug;
    type Str: AsRef<str> + PartialEq + fmt::Debug;
}

pub trait ValueBuilder: ValueFlavor {
    fn make_ptr<T: PartialEq + fmt::Debug>(value: T) -> Self::Ptr<T>;

    fn make_str(str: impl AsRef<str>) -> Self::Str;

    fn list<T: PartialEq + fmt::Debug>() -> Self::List<T>;
    fn list_from_iter<T: PartialEq + fmt::Debug>(iter: impl Iterator<Item = T>) -> Self::List<T>;
    fn list_with_capacity<T: PartialEq + fmt::Debug>(capacity: usize) -> Self::List<T>;
    fn list_push<T: PartialEq + fmt::Debug>(builder: &mut Self::List<T>, value: T);
}

pub mod ser {
    use ::{
        core::ops::Deref,
        serde::{ser::SerializeSeq as _, Serialize, Serializer},
    };

    #[inline]
    pub fn serialize_list_ptr<S: Serializer, T: Serialize>(
        list: &impl Deref<Target = [impl Deref<Target = T>]>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let slice = &**list;
        let mut seq = serializer.serialize_seq(Some(slice.len()))?;
        for el in slice {
            seq.serialize_element(&**el)?;
        }
        seq.end()
    }

    #[inline]
    pub fn serialize_ptr<S: Serializer, T: Serialize>(
        ptr: &impl Deref<Target = T>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        ptr.deref().serialize(serializer)
    }
}
