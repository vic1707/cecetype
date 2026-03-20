pub struct Owned;

#[cfg(feature = "std")]
impl<'s> super::SchemaFlavor<'s> for Owned {
    type Ptr<T: 's> = ::std::boxed::Box<T>;
    type List<T: 's> = ::std::vec::Vec<::std::boxed::Box<T>>;
    type Str = ::std::string::String;
}

#[cfg(feature = "std")]
impl super::ValueFlavor for Owned {
    type Ptr<T: PartialEq + ::core::fmt::Debug> = ::std::boxed::Box<T>;
    type List<T: PartialEq + ::core::fmt::Debug> = ::std::vec::Vec<T>;
    type Str = ::std::string::String;
}

#[cfg(feature = "std")]
impl<'s> super::OwnedSchemaFlavor<'s> for Owned {
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
}

#[cfg(feature = "std")]
impl super::ValueBuilder for Owned {
    fn make_ptr<T: PartialEq + ::core::fmt::Debug>(value: T) -> Self::Ptr<T> {
        ::std::boxed::Box::new(value)
    }

    fn make_str(str: &str) -> Self::Str {
        ::std::string::String::from(str)
    }

    fn list<T: PartialEq + ::core::fmt::Debug>() -> Self::List<T> {
        Self::List::new()
    }
    fn list_from_iter<T: PartialEq + ::core::fmt::Debug>(
        iter: impl Iterator<Item = T>,
    ) -> Self::List<T> {
        iter.collect()
    }
    fn list_with_capacity<T: PartialEq + ::core::fmt::Debug>(capacity: usize) -> Self::List<T> {
        Self::List::with_capacity(capacity)
    }
    fn list_push<T: PartialEq + ::core::fmt::Debug>(builder: &mut Self::List<T>, value: T) {
        builder.push(value);
    }
}
