mod borrowed;
mod owned;

pub use self::{borrowed::*, owned::*};

pub trait SchemaFlavor<'s>
where
    Self: 's,
{
    type Ptr<T: 's>: core::ops::Deref<Target = T>;
    type List<T: 's>: core::ops::Deref<Target = [Self::Ptr<T>]>;
    type Str: core::ops::Deref<Target = str>;

    fn serialize_ptr<S: ::serde::Serializer, T: ::serde::Serialize>(
        ptr: &Self::Ptr<T>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        use core::ops::Deref as _;

        ptr.deref().serialize(serializer)
    }

    fn serialize_list<S: ::serde::Serializer, T: ::serde::Serialize>(
        list: &Self::List<T>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        use ::{core::ops::Deref as _, serde::ser::SerializeSeq as _};

        let slice = list.deref();
        let mut seq = serializer.serialize_seq(Some(slice.len()))?;
        for p in slice {
            seq.serialize_element(p.deref())?;
        }
        seq.end()
    }
}

pub trait OwnedSchemaFlavor<'s>: SchemaFlavor<'s> {
    fn deserialize_ptr<'de, D, T>(deserializer: D) -> Result<Self::Ptr<T>, D::Error>
    where
        D: ::serde::Deserializer<'de>,
        T: ::serde::Deserialize<'de>;

    fn deserialize_list<'de, D, T>(deserializer: D) -> Result<Self::List<T>, D::Error>
    where
        D: ::serde::Deserializer<'de>,
        T: ::serde::Deserialize<'de>;
}
pub trait ValueFlavor<'s>
where
    Self: 's,
{
    type Ptr<T: 's>: core::ops::Deref<Target = T>;
    type List<T: 's>: core::ops::Deref<Target = [Self::Ptr<T>]>;
    type Str: core::ops::Deref<Target = str>;
}
