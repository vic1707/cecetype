#[cfg(feature = "alloc")]
use ::{
    alloc::{boxed::Box, string::String, vec::Vec},
    core::fmt,
};

pub struct Owned;

#[cfg(feature = "alloc")]
impl<'s> super::SchemaFlavor<'s> for Owned {
    type Ptr<T: 's + Clone + PartialEq + fmt::Debug> = Box<T>;
    type List<T: 's + Clone + PartialEq + fmt::Debug> = Vec<Box<T>>;
    type Str = String;
}

#[cfg(feature = "alloc")]
impl super::ValueFlavor for Owned {
    type Ptr<T: PartialEq + fmt::Debug> = Box<T>;
    type List<T: PartialEq + fmt::Debug> = Vec<T>;
    type Str = String;
}

#[cfg(feature = "alloc")]
impl<'s> super::OwnedSchemaFlavor<'s> for Owned {
    #[inline]
    fn deserialize_ptr<'de, D, T>(deserializer: D) -> Result<Self::Ptr<T>, D::Error>
    where
        D: ::serde::Deserializer<'de>,
        T: ::serde::Deserialize<'de> + 's + Clone + PartialEq + fmt::Debug,
    {
        let value = T::deserialize(deserializer)?;
        Ok(Box::new(value))
    }

    #[inline]
    fn deserialize_list<'de, D, T>(deserializer: D) -> Result<Self::List<T>, D::Error>
    where
        D: ::serde::Deserializer<'de>,
        T: ::serde::Deserialize<'de> + 's + Clone + PartialEq + fmt::Debug,
    {
        use ::serde::Deserialize as _;

        let values: Vec<T> = Vec::deserialize(deserializer)?;
        Ok(values.into_iter().map(Box::new).collect())
    }
}

#[cfg(feature = "alloc")]
impl super::ValueBuilder for Owned {
    #[inline]
    fn make_ptr<T: PartialEq + fmt::Debug>(value: T) -> Self::Ptr<T> {
        Box::new(value)
    }

    #[inline]
    fn make_str(str: &str) -> Self::Str {
        String::from(str)
    }

    #[inline]
    fn list<T: PartialEq + fmt::Debug>() -> Self::List<T> {
        Self::List::new()
    }
    #[inline]
    fn list_from_iter<T: PartialEq + fmt::Debug>(iter: impl Iterator<Item = T>) -> Self::List<T> {
        iter.collect()
    }
    #[inline]
    fn list_with_capacity<T: PartialEq + fmt::Debug>(capacity: usize) -> Self::List<T> {
        Self::List::with_capacity(capacity)
    }
    #[inline]
    fn list_push<T: PartialEq + fmt::Debug>(builder: &mut Self::List<T>, value: T) {
        builder.push(value);
    }
}
