/// Zero-copy flavor: `&'static T`, `&'static [&'static T]`, `&'static str`.
pub struct Static;

impl super::SchemaFlavor<'static> for Static {
    type Ptr<T: 'static> = &'static T;
    type List<T: 'static> = &'static [&'static T];
    type Str = &'static str;
}

/// Borrowed flavor: `&'s T`, `&'s [&'s T]`, `&'s str`.
pub struct Borrowed;

impl<'s> super::SchemaFlavor<'s> for Borrowed {
    type Ptr<T: 's> = &'s T;
    type List<T: 's> = &'s [&'s T];
    type Str = &'s str;
}
