#[cfg(feature = "alloc")]
use ::{
    alloc::{boxed::Box, string::String, vec::Vec},
    core::{fmt, marker::PhantomData},
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
    type Ptr<T: PartialEq + fmt::Debug + Clone> = Box<T>;
    type List<T: PartialEq + fmt::Debug + Clone> = Vec<T>;
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

#[cfg(feature = "alloc")]
impl super::ValueBuilder for Owned {
    #[inline]
    fn make_ptr<T: PartialEq + fmt::Debug + Clone>(value: T) -> Self::Ptr<T> {
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
    fn list<T: PartialEq + fmt::Debug + Clone>() -> Self::List<T> {
        Self::List::new()
    }
    #[inline]
    fn list_from_iter<T: PartialEq + fmt::Debug + Clone>(
        iter: impl Iterator<Item = T>,
    ) -> Self::List<T> {
        iter.collect()
    }
    #[inline]
    fn list_with_capacity<T: PartialEq + fmt::Debug + Clone>(capacity: usize) -> Self::List<T> {
        Self::List::with_capacity(capacity)
    }
    #[inline]
    fn list_push<T: PartialEq + fmt::Debug + Clone>(builder: &mut Self::List<T>, value: T) {
        builder.push(value);
    }
}
