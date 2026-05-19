//! Parser abstraction for building [`Value`](crate::value::Value) from schemas.
//!
//! A parser provides typed atoms and container boundaries. The built-in CLI
//! parser is one implementation, but the trait can also drive other dynamic
//! inputs.
use crate::{flavors, schema, value};
use ::core::error;

#[cfg(feature = "cli")]
pub mod cli;

/// Error produced while building a value from a schema and parser.
#[derive(Debug, ::thiserror::Error)]
pub enum BuildError<'schema, E: error::Error> {
    #[error("unresolved schema ref: '{0}'")]
    UnresolvedRef(&'schema str),
    #[error("parser error: {0}")]
    Parser(E),
}

#[cfg(feature = "miette")]
impl<E: error::Error + ::miette::Diagnostic> ::miette::Diagnostic for BuildError<'_, E> {
    #[inline]
    fn code<'a>(&'a self) -> Option<::std::boxed::Box<dyn ::core::fmt::Display + 'a>> {
        match self {
            Self::UnresolvedRef(_) => Some(::std::boxed::Box::new(
                "cecetype::parse::unresolved_schema_ref",
            )),
            Self::Parser(err) => ::miette::Diagnostic::code(err),
        }
    }

    #[inline]
    fn help<'a>(&'a self) -> Option<::std::boxed::Box<dyn ::core::fmt::Display + 'a>> {
        match self {
            Self::UnresolvedRef(_) => Some(::std::boxed::Box::new(
                "make sure referenced schemas are registered before parsing",
            )),
            Self::Parser(err) => ::miette::Diagnostic::help(err),
        }
    }

    #[inline]
    fn source_code(&self) -> Option<&dyn ::miette::SourceCode> {
        match self {
            Self::UnresolvedRef(_) => None,
            Self::Parser(err) => ::miette::Diagnostic::source_code(err),
        }
    }

    #[inline]
    fn labels(&self) -> Option<::std::boxed::Box<dyn Iterator<Item = ::miette::LabeledSpan> + '_>> {
        match self {
            Self::UnresolvedRef(_) => None,
            Self::Parser(err) => ::miette::Diagnostic::labels(err),
        }
    }

    #[inline]
    fn diagnostic_source(&self) -> Option<&dyn ::miette::Diagnostic> {
        // Parser diagnostics are flattened onto this wrapper via code/help/source/labels.
        // Returning them here as a cause makes miette render duplicate snippets.
        None
    }
}

/// Source of typed values for schema-driven parsing.
///
/// Implement this trait when input is not already a serde format but can still
/// be walked according to a schema.
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
        variants: &'s SF::List<schema::VariantSchema<'s, SF>>,
        builder: impl FnMut(
            &mut Self,
            &'s schema::Schema<'s, SF>,
        ) -> Result<value::Value<VB>, BuildError<'s, Self::Error>>,
    ) -> Result<(u32, VB::Str, value::Data<VB>), BuildError<'s, Self::Error>>;
}
