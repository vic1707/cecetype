pub trait SchemaFlavor<'s>
where
    Self: 's,
{
    type Ptr<T: 's>: core::ops::Deref<Target = T>;
    type List<T: 's>: core::ops::Deref<Target = [T]>;
    type Str: core::ops::Deref<Target = str>;
}

pub struct Static;

impl SchemaFlavor<'static> for Static {
    type Ptr<T: 'static> = &'static T;
    type List<T: 'static> = &'static [T];
    type Str = &'static str;
}

pub struct Borrowed;

impl<'s> SchemaFlavor<'s> for Borrowed {
    type Ptr<T: 's> = &'s T;
    type List<T: 's> = &'s [T];
    type Str = &'s str;
}

#[cfg(feature = "std")]
pub struct Owned;

#[cfg(feature = "std")]
impl<'s> SchemaFlavor<'s> for Owned {
    type Ptr<T: 's> = ::std::boxed::Box<T>;
    type List<T: 's> = ::std::vec::Vec<T>;
    type Str = ::std::string::String;
}
