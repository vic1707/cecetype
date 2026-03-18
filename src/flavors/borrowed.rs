pub struct Static;

impl super::SchemaFlavor<'static> for Static {
    type Ptr<T: 'static> = &'static T;
    type List<T: 'static> = &'static [&'static T];
    type Str = &'static str;
}


pub struct Borrowed;

impl<'s> super::SchemaFlavor<'s> for Borrowed {
    type Ptr<T: 's> = &'s T;
    type List<T: 's> = &'s [&'s T];
    type Str = &'s str;
}

