use ::core::fmt;

/// Zero-copy flavor: `&'static T`, `&'static [&'static T]`, `&'static str`.
#[derive(Debug)]
pub struct Static;

impl super::SchemaFlavor<'static> for Static {
    type Ptr<T: 'static + Clone + PartialEq + fmt::Debug> = &'static T;
    type List<T: 'static + Clone + PartialEq + fmt::Debug> = &'static [&'static T];
    type Str = &'static str;
}

/// Borrowed flavor: `&'s T`, `&'s [&'s T]`, `&'s str`.
#[derive(Debug)]
pub struct Borrowed;

impl<'s> super::SchemaFlavor<'s> for Borrowed {
    type Ptr<T: 's + Clone + PartialEq + fmt::Debug> = &'s T;
    type List<T: 's + Clone + PartialEq + fmt::Debug> = &'s [&'s T];
    type Str = &'s str;
}
