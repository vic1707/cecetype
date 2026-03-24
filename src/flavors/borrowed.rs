use ::core::fmt;

pub struct Static;

impl super::SchemaFlavor<'static> for Static {
    type Ptr<T: 'static + PartialEq + fmt::Debug> = &'static T;
    type List<T: 'static + PartialEq + fmt::Debug> = &'static [&'static T];
    type Str = &'static str;
}

pub struct Borrowed;

impl<'s> super::SchemaFlavor<'s> for Borrowed {
    type Ptr<T: 's + PartialEq + fmt::Debug> = &'s T;
    type List<T: 's + PartialEq + fmt::Debug> = &'s [&'s T];
    type Str = &'s str;
}
