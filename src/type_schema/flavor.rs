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

    fn deserialize_str<'de, D>(deserializer: D) -> Result<Self::Str, D::Error>
    where
        D: ::serde::Deserializer<'de>;
}

pub struct Static;

impl SchemaFlavor<'static> for Static {
    type Ptr<T: 'static> = &'static T;
    type List<T: 'static> = &'static [&'static T];
    type Str = &'static str;
}

pub struct Borrowed;

impl<'s> SchemaFlavor<'s> for Borrowed {
    type Ptr<T: 's> = &'s T;
    type List<T: 's> = &'s [&'s T];
    type Str = &'s str;
}

#[cfg(feature = "std")]
pub struct Owned;

#[cfg(feature = "std")]
impl<'s> SchemaFlavor<'s> for Owned {
    type Ptr<T: 's> = ::std::boxed::Box<T>;
    type List<T: 's> = ::std::vec::Vec<::std::boxed::Box<T>>;
    type Str = ::std::string::String;
}

#[cfg(feature = "std")]
impl<'s> OwnedSchemaFlavor<'s> for Owned {
    fn deserialize_ptr<'de, D, T>(deserializer: D) -> Result<Self::Ptr<T>, D::Error>
    where
        D: ::serde::Deserializer<'de>,
        T: ::serde::Deserialize<'de> + 's,
    {
        let value = T::deserialize(deserializer)?;
        Ok(::std::boxed::Box::new(value))
    }

    fn deserialize_list<'de, D, T>(deserializer: D) -> Result<Self::List<T>, D::Error>
    where
        D: ::serde::Deserializer<'de>,
        T: ::serde::Deserialize<'de> + 's,
    {
        use ::serde::Deserialize as _;

        let values: ::std::vec::Vec<T> = ::std::vec::Vec::deserialize(deserializer)?;
        Ok(values.into_iter().map(::std::boxed::Box::new).collect())
    }

    fn deserialize_str<'de, D>(deserializer: D) -> Result<Self::Str, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        use ::serde::Deserialize as _;

        ::std::string::String::deserialize(deserializer)
    }
}
