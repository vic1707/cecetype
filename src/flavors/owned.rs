#![cfg(feature = "alloc")]
use ::{
    alloc::{boxed::Box, string::String, vec::Vec},
    core::{fmt, marker::PhantomData},
};

/// Owned flavor: `Box<T>`, `Vec<T>`, `String`. Requires `alloc`.
pub struct Owned;

impl<'s> super::SchemaFlavor<'s> for Owned {
    type Ptr<T: 's> = Box<T>;
    type List<T: 's> = Vec<Box<T>>;
    type Str = String;
}

impl super::ValueFlavor for Owned {
    type Ptr<T> = Box<T>;
    type List<T> = Vec<T>;
    type Str = String;
}

impl<'s> super::OwnedSchemaFlavor<'s> for Owned {
    #[inline]
    fn deserialize_ptr<'de, D, T>(deserializer: D) -> Result<Self::Ptr<T>, D::Error>
    where
        D: ::serde::Deserializer<'de>,
        T: ::serde::Deserialize<'de> + 's,
    {
        let value = T::deserialize(deserializer)?;
        Ok(Box::new(value))
    }

    #[inline]
    fn deserialize_list<'de, D, T>(deserializer: D) -> Result<Self::List<T>, D::Error>
    where
        D: ::serde::Deserializer<'de>,
        T: ::serde::Deserialize<'de> + 's,
    {
        use ::serde::de::{SeqAccess, Visitor};

        struct BoxSeqVisitor<T>(PhantomData<T>);

        impl<'de, T> Visitor<'de> for BoxSeqVisitor<T>
        where
            T: ::serde::Deserialize<'de>,
        {
            type Value = Vec<Box<T>>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a sequence")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut values = Vec::with_capacity(seq.size_hint().unwrap_or(0_usize));
                while let Some(value) = seq.next_element::<T>()? {
                    values.push(Box::new(value));
                }
                Ok(values)
            }
        }

        deserializer.deserialize_seq(BoxSeqVisitor(PhantomData))
    }
}

impl super::ValueBuilder for Owned {
    #[inline]
    fn make_ptr<T>(value: T) -> Self::Ptr<T> {
        Box::new(value)
    }

    #[inline]
    fn make_str(str: impl AsRef<str>) -> Self::Str {
        String::from(str.as_ref())
    }
    #[inline]
    fn make_str_from_display(disp: &impl fmt::Display) -> Self::Str {
        ::alloc::format!("{disp}")
    }

    #[inline]
    fn list<T>() -> Self::List<T> {
        Self::List::new()
    }
    #[inline]
    fn list_from_iter<T>(iter: impl Iterator<Item = T>) -> Self::List<T> {
        iter.collect()
    }
    #[inline]
    fn list_with_capacity<T>(capacity: usize) -> Self::List<T> {
        Self::List::with_capacity(capacity)
    }
    #[inline]
    fn list_push<T>(builder: &mut Self::List<T>, value: T) {
        builder.push(value);
    }
}
