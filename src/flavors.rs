mod borrowed;
mod owned;

pub use self::{borrowed::*, owned::*};

pub trait SchemaFlavor<'s>
where
    Self: 's,
{
    type Ptr<T: 's>: ::core::ops::Deref<Target = T>;
    type List<T: 's>: ::core::ops::Deref<Target = [Self::Ptr<T>]>;
    type Str: ::core::ops::Deref<Target = str>;
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
    type Ptr<T: 's>: ::core::ops::Deref<Target = T>;
    type List<T: 's>: ::core::ops::Deref<Target = [Self::Ptr<T>]>;
    type Str: ::core::ops::Deref<Target = str>;
}

pub(crate) mod ser {
    use ::{
        core::ops::Deref,
        serde::{Serialize, Serializer, ser::SerializeSeq as _},
    };

    pub fn serialize_list<S: Serializer, T: Serialize>(
        list: &impl Deref<Target = [impl Deref<Target = T>]>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let slice = list.deref();
        let mut seq = serializer.serialize_seq(Some(slice.len()))?;
        for p in slice {
            seq.serialize_element(p.deref())?;
        }
        seq.end()
    }

    pub fn serialize_opt_ptr<S: Serializer, T: Serialize>(
        ptr: &Option<impl Deref<Target = T>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match ptr {
            None => serializer.serialize_none(),
            Some(p) => serializer.serialize_some(p.deref()),
        }
    }

    pub fn serialize_ptr<S: Serializer, T: Serialize>(
        ptr: &impl Deref<Target = T>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        ptr.deref().serialize(serializer)
    }
}
