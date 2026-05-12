use crate::{flavors, schema, value};
use ::core::error;

#[cfg(feature = "cli")]
pub mod cli;

#[derive(Debug, ::thiserror::Error)]
pub enum BuildError<'schema, E: error::Error> {
    #[error("unresolved schema ref: '{0}'")]
    UnresolvedRef(&'schema str),
    #[error("parser error: {0}")]
    Parser(E),
}

pub trait Parser<'s, VB: flavors::ValueBuilder>: Sized {
    type Error: error::Error;
    type Atom;

    fn next_atom(&mut self) -> Result<Self::Atom, Self::Error>;

    #[inline]
    fn parse_unit(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn parse_bool(&mut self) -> Result<bool, Self::Error>;

    #[inline]
    fn finish(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn parse_char(&mut self) -> Result<char, Self::Error>;
    fn parse_string(&mut self) -> Result<impl AsRef<str>, Self::Error>;

    fn parse_u8(&mut self) -> Result<u8, Self::Error>;
    fn parse_u16(&mut self) -> Result<u16, Self::Error>;
    fn parse_u32(&mut self) -> Result<u32, Self::Error>;
    fn parse_u64(&mut self) -> Result<u64, Self::Error>;
    fn parse_u128(&mut self) -> Result<u128, Self::Error>;

    fn parse_i8(&mut self) -> Result<i8, Self::Error>;
    fn parse_i16(&mut self) -> Result<i16, Self::Error>;
    fn parse_i32(&mut self) -> Result<i32, Self::Error>;
    fn parse_i64(&mut self) -> Result<i64, Self::Error>;
    fn parse_i128(&mut self) -> Result<i128, Self::Error>;

    fn parse_f32(&mut self) -> Result<f32, Self::Error>;
    fn parse_f64(&mut self) -> Result<f64, Self::Error>;

    fn parse_map<SF: flavors::SchemaFlavor<'s>>(
        &mut self,
        kv_schemas: (&'s schema::Schema<'s, SF>, &'s schema::Schema<'s, SF>),
        builder: impl FnMut(
            &mut Self,
            &'s schema::Schema<'s, SF>,
        ) -> Result<value::Value<VB>, BuildError<'s, Self::Error>>,
    ) -> Result<VB::List<(value::Value<VB>, value::Value<VB>)>, BuildError<'s, Self::Error>>;

    fn parse_array<SF: flavors::SchemaFlavor<'s>>(
        &mut self,
        schemas: impl ExactSizeIterator<Item = &'s schema::Schema<'s, SF>>,
        builder: impl FnMut(
            &mut Self,
            &'s schema::Schema<'s, SF>,
        ) -> Result<value::Value<VB>, BuildError<'s, Self::Error>>,
    ) -> Result<VB::List<value::Value<VB>>, BuildError<'s, Self::Error>>;

    fn parse_seq<SF: flavors::SchemaFlavor<'s>>(
        &mut self,
        element: &'s schema::Schema<'s, SF>,
        builder: impl FnMut(
            &mut Self,
            &'s schema::Schema<'s, SF>,
        ) -> Result<value::Value<VB>, BuildError<'s, Self::Error>>,
    ) -> Result<VB::List<value::Value<VB>>, BuildError<'s, Self::Error>>;

    fn parse_tuple<SF: flavors::SchemaFlavor<'s>>(
        &mut self,
        schemas: impl ExactSizeIterator<Item = &'s schema::Schema<'s, SF>>,
        builder: impl FnMut(
            &mut Self,
            &'s schema::Schema<'s, SF>,
        ) -> Result<value::Value<VB>, BuildError<'s, Self::Error>>,
    ) -> Result<VB::List<value::Value<VB>>, BuildError<'s, Self::Error>>;

    fn parse_option<SF: flavors::SchemaFlavor<'s>>(
        &mut self,
        inner: &'s schema::Schema<'s, SF>,
        builder: impl FnMut(
            &mut Self,
            &'s schema::Schema<'s, SF>,
        ) -> Result<value::Value<VB>, BuildError<'s, Self::Error>>,
    ) -> Result<Option<VB::Ptr<value::Value<VB>>>, BuildError<'s, Self::Error>>;

    fn parse_struct<SF: flavors::SchemaFlavor<'s>>(
        &mut self,
        data: &'s schema::Data<'s, SF>,
        builder: impl FnMut(
            &mut Self,
            &'s schema::Schema<'s, SF>,
        ) -> Result<value::Value<VB>, BuildError<'s, Self::Error>>,
    ) -> Result<value::Data<VB>, BuildError<'s, Self::Error>>;

    fn parse_enum<SF: flavors::SchemaFlavor<'s>>(
        &mut self,
        variants: &'s SF::List<(u32, SF::Str, schema::Data<'s, SF>)>,
        builder: impl FnMut(
            &mut Self,
            &'s schema::Schema<'s, SF>,
        ) -> Result<value::Value<VB>, BuildError<'s, Self::Error>>,
    ) -> Result<(u32, VB::Str, value::Data<VB>), BuildError<'s, Self::Error>>;
}
