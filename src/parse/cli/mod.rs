//! Positional CLI parser for building values from user input.
//!
//! The parser is designed to pair with [`spec`] output. Struct fields and tuple
//! fields are read in schema order, while nested multi-field values are grouped
//! with parentheses.
//!
//! ```
//! use cecetype::{OwnedValue, Schema, parse::cli::Parser, flavors::Owned};
//!
//! #[derive(Schema)]
//! struct Request { id: u32, name: Option<String>, msg: String }
//!
//! let mut parser = Parser::new(r#"42 some('toto') 'hello'"#);
//! let value: OwnedValue = Request::SCHEMA.build_value::<Owned, _>(&mut parser).unwrap();
//! ```

mod lexer;
pub mod spec;
mod word;

use self::{
    lexer::{LexError, Spanned, Token, Tokens},
    word::{ParseError, Word},
};
use crate::{flavors, schema, value};
use ::core::{fmt, iter::Peekable, mem, ops::Range};

/// CLI parser error with location info.
#[derive(Debug, ::thiserror::Error)]
#[expect(
    clippy::error_impl_error,
    reason = "public CLI error type is intentionally named Error"
)]
#[error("at {path}: {kind}")]
pub struct Error<'input, S> {
    pub path: S,
    pub kind: ErrorKind<'input>,
    pub input: &'input str,
    pub span: Range<usize>,
}

/// CLI parser error kinds.
#[derive(Debug, ::thiserror::Error)]
pub enum ErrorKind<'input> {
    #[error("lex error: {0}")]
    Lex(#[from] LexError),
    #[error("parse error: {0}")]
    Parse(#[from] ParseError),
    #[error("expected bare word, got quoted")]
    UnexpectedQuotedWord,
    #[error("expected quoted word, got bare")]
    ExpectedQuotedWord,
    #[error("unexpected end of input")]
    EOF,
    #[error("unexpected token '{0}'")]
    UnexpectedToken(Token<'input>),
    #[error("unexpected token '{0}', expected '{1}'")]
    UnexpectedExpectedToken(Token<'input>, Token<'input>),
    #[error("variant '{0}' not found")]
    VariantNotFound(&'input str),
}

impl<'input> Token<'input> {
    const fn word(self) -> Result<Word<'input>, ErrorKind<'input>> {
        if let Self::Word(word) = self {
            return Ok(word);
        }
        Err(ErrorKind::UnexpectedToken(self))
    }
}

#[cfg(feature = "miette")]
impl<S> ::miette::Diagnostic for Error<'_, S>
where
    S: AsRef<str> + fmt::Debug + fmt::Display,
{
    #[inline]
    fn code<'a>(&'a self) -> Option<::std::boxed::Box<dyn fmt::Display + 'a>> {
        let code = match &self.kind {
            ErrorKind::Lex(_) => "cecetype::cli::lex",
            ErrorKind::Parse(_) => "cecetype::cli::parse_atom",
            ErrorKind::UnexpectedQuotedWord => "cecetype::cli::unexpected_quoted_word",
            ErrorKind::ExpectedQuotedWord => "cecetype::cli::expected_quoted_word",
            ErrorKind::EOF => "cecetype::cli::unexpected_eof",
            ErrorKind::UnexpectedToken(_) => "cecetype::cli::unexpected_token",
            ErrorKind::UnexpectedExpectedToken(_, _) => "cecetype::cli::unexpected_token",
            ErrorKind::VariantNotFound(_) => "cecetype::cli::variant_not_found",
        };
        Some(::std::boxed::Box::new(code))
    }

    #[inline]
    fn help<'a>(&'a self) -> Option<::std::boxed::Box<dyn fmt::Display + 'a>> {
        let help = match &self.kind {
            ErrorKind::UnexpectedQuotedWord => "remove quotes around non-string values",
            ErrorKind::ExpectedQuotedWord => "wrap string and character values in quotes",
            ErrorKind::UnexpectedExpectedToken(_, _) => {
                "check the command usage for expected separators and grouping"
            }
            ErrorKind::VariantNotFound(_) => {
                "choose one of the enum variants from the command usage"
            }
            _ => return None,
        };
        Some(::std::boxed::Box::new(help))
    }

    #[inline]
    fn source_code(&self) -> Option<&dyn ::miette::SourceCode> {
        Some(&self.input as &dyn ::miette::SourceCode)
    }

    #[inline]
    fn labels(&self) -> Option<::std::boxed::Box<dyn Iterator<Item = ::miette::LabeledSpan> + '_>> {
        let label = ::std::format!("while parsing {}", self.path.as_ref());
        Some(::std::boxed::Box::new(::core::iter::once(
            ::miette::LabeledSpan::new_primary_with_span(Some(label), self.span.clone()),
        )))
    }
}

#[derive(Debug)]
pub struct Parser<'input, VB: flavors::ValueBuilder> {
    input: &'input str,
    tokens: Peekable<Tokens<'input>>,
    path: VB::Str,
    last_span: Range<usize>,
    depth: u32,
}

impl<'input, VB: flavors::ValueBuilder> Parser<'input, VB> {
    #[inline]
    #[must_use]
    pub fn new(input: &'input str) -> Self {
        Self {
            input,
            tokens: Tokens::new(input).peekable(),
            path: VB::make_str("<root>"),
            last_span: input.len()..input.len(),
            depth: 0,
        }
    }

    fn lift_err(&self, err: ErrorKind<'input>) -> Error<'input, VB::Str> {
        Error {
            path: VB::make_str(self.path.as_ref()),
            kind: err,
            input: self.input,
            span: self.last_span.clone(),
        }
    }

    fn lift_eof_err(&mut self) -> Error<'input, VB::Str> {
        let end = self.input.len();
        self.last_span = end..end;
        self.lift_err(ErrorKind::EOF)
    }
}

impl<'input, 's, VB: flavors::ValueBuilder> super::Parser<'s, VB> for Parser<'input, VB> {
    type Atom = Token<'input>;
    type Error = Error<'input, VB::Str>;

    #[inline]
    fn next_atom(&mut self) -> Result<Self::Atom, Self::Error> {
        let Some(Spanned { value, span }) = self.tokens.next() else {
            return Err(self.lift_eof_err());
        };
        self.last_span = span;

        value.map_err(|err| self.lift_err(ErrorKind::Lex(err)))
    }

    #[inline]
    fn parse_unit(&mut self) -> Result<(), Self::Error> {
        if self.consume_if(&Token::LParen) {
            self.expect(Token::RParen)?;
        }
        Ok(())
    }

    #[inline]
    fn finish(&mut self) -> Result<(), Self::Error> {
        let Some(Spanned { value, span }) = self.tokens.next() else {
            return Ok(());
        };
        self.last_span = span;

        match value {
            Ok(tok) => Err(self.lift_err(ErrorKind::UnexpectedToken(tok))),
            Err(err) => Err(self.lift_err(err.into())),
        }
    }

    #[inline]
    fn parse_bool(&mut self) -> Result<bool, Self::Error> {
        <Self as super::Parser<'s, VB>>::next_atom(self)?
            .word()
            .and_then(Word::parse_bool)
            .map_err(|err| self.lift_err(err))
    }

    #[inline]
    fn parse_string(&mut self) -> Result<impl AsRef<str>, Self::Error> {
        <Self as super::Parser<'s, VB>>::next_atom(self)?
            .word()
            .and_then(Word::parse_string)
            .map_err(|err| self.lift_err(err))
    }
    #[inline]
    fn parse_char(&mut self) -> Result<char, Self::Error> {
        <Self as super::Parser<'s, VB>>::next_atom(self)?
            .word()
            .and_then(Word::parse_char)
            .map_err(|err| self.lift_err(err))
    }

    #[inline]
    fn parse_u8(&mut self) -> Result<u8, Self::Error> {
        <Self as super::Parser<'s, VB>>::next_atom(self)?
            .word()
            .and_then(Word::parse_u8)
            .map_err(|err| self.lift_err(err))
    }
    #[inline]
    fn parse_u16(&mut self) -> Result<u16, Self::Error> {
        <Self as super::Parser<'s, VB>>::next_atom(self)?
            .word()
            .and_then(Word::parse_u16)
            .map_err(|err| self.lift_err(err))
    }
    #[inline]
    fn parse_u32(&mut self) -> Result<u32, Self::Error> {
        <Self as super::Parser<'s, VB>>::next_atom(self)?
            .word()
            .and_then(Word::parse_u32)
            .map_err(|err| self.lift_err(err))
    }
    #[inline]
    fn parse_u64(&mut self) -> Result<u64, Self::Error> {
        <Self as super::Parser<'s, VB>>::next_atom(self)?
            .word()
            .and_then(Word::parse_u64)
            .map_err(|err| self.lift_err(err))
    }
    #[inline]
    fn parse_u128(&mut self) -> Result<u128, Self::Error> {
        <Self as super::Parser<'s, VB>>::next_atom(self)?
            .word()
            .and_then(Word::parse_u128)
            .map_err(|err| self.lift_err(err))
    }

    #[inline]
    fn parse_i8(&mut self) -> Result<i8, Self::Error> {
        <Self as super::Parser<'s, VB>>::next_atom(self)?
            .word()
            .and_then(Word::parse_i8)
            .map_err(|err| self.lift_err(err))
    }
    #[inline]
    fn parse_i16(&mut self) -> Result<i16, Self::Error> {
        <Self as super::Parser<'s, VB>>::next_atom(self)?
            .word()
            .and_then(Word::parse_i16)
            .map_err(|err| self.lift_err(err))
    }
    #[inline]
    fn parse_i32(&mut self) -> Result<i32, Self::Error> {
        <Self as super::Parser<'s, VB>>::next_atom(self)?
            .word()
            .and_then(Word::parse_i32)
            .map_err(|err| self.lift_err(err))
    }
    #[inline]
    fn parse_i64(&mut self) -> Result<i64, Self::Error> {
        <Self as super::Parser<'s, VB>>::next_atom(self)?
            .word()
            .and_then(Word::parse_i64)
            .map_err(|err| self.lift_err(err))
    }
    #[inline]
    fn parse_i128(&mut self) -> Result<i128, Self::Error> {
        <Self as super::Parser<'s, VB>>::next_atom(self)?
            .word()
            .and_then(Word::parse_i128)
            .map_err(|err| self.lift_err(err))
    }

    #[inline]
    fn parse_f32(&mut self) -> Result<f32, Self::Error> {
        <Self as super::Parser<'s, VB>>::next_atom(self)?
            .word()
            .and_then(Word::parse_f32)
            .map_err(|err| self.lift_err(err))
    }
    #[inline]
    fn parse_f64(&mut self) -> Result<f64, Self::Error> {
        <Self as super::Parser<'s, VB>>::next_atom(self)?
            .word()
            .and_then(Word::parse_f64)
            .map_err(|err| self.lift_err(err))
    }

    #[inline]
    fn parse_map<SF: flavors::SchemaFlavor<'s>>(
        &mut self,
        (key_schema, value_schema): (&'s schema::Schema<'s, SF>, &'s schema::Schema<'s, SF>),
        mut builder: impl FnMut(
            &mut Self,
            &'s schema::Schema<'s, SF>,
        ) -> Result<value::Value<VB>, super::BuildError<'s, Self::Error>>,
    ) -> Result<VB::List<(value::Value<VB>, value::Value<VB>)>, super::BuildError<'s, Self::Error>>
    {
        self.expect(Token::LBrace)
            .map_err(super::BuildError::Parser)?;

        let mut fields = VB::list();

        while !self.consume_if(&Token::RBrace) {
            let key = builder(self, key_schema)?;

            let new_path =
                VB::make_str_from_display(&format_args!("{}{{{}}}.$", self.path.as_ref(), key));
            let saved = mem::replace(&mut self.path, new_path);

            let result = (|| {
                self.expect(Token::Colon)
                    .map_err(super::BuildError::Parser)?;
                builder(self, value_schema)
            })();

            self.path = saved;
            let value = result?;

            VB::list_push(&mut fields, (key, value));
            self.consume_if(&Token::Comma);
        }

        Ok(fields)
    }

    #[inline]
    fn parse_array<SF: flavors::SchemaFlavor<'s>>(
        &mut self,
        schemas: impl ExactSizeIterator<Item = &'s schema::Schema<'s, SF>>,
        mut builder: impl FnMut(
            &mut Self,
            &'s schema::Schema<'s, SF>,
        ) -> Result<value::Value<VB>, super::BuildError<'s, Self::Error>>,
    ) -> Result<VB::List<value::Value<VB>>, super::BuildError<'s, Self::Error>> {
        self.expect(Token::LBracket)
            .map_err(super::BuildError::Parser)?;

        let values = self.parse_repeated::<_, _>(schemas, Some(Token::Comma), |this, schema| {
            builder(this, schema)
        })?;

        self.expect(Token::RBracket)
            .map_err(super::BuildError::Parser)?;

        Ok(values)
    }

    #[inline]
    fn parse_seq<SF: flavors::SchemaFlavor<'s>>(
        &mut self,
        element: &'s schema::Schema<'s, SF>,
        mut builder: impl FnMut(
            &mut Self,
            &'s schema::Schema<'s, SF>,
        ) -> Result<value::Value<VB>, super::BuildError<'s, Self::Error>>,
    ) -> Result<VB::List<value::Value<VB>>, super::BuildError<'s, Self::Error>> {
        self.expect(Token::LBracket)
            .map_err(super::BuildError::Parser)?;

        let mut values = VB::list();

        while !self.consume_if(&Token::RBracket) {
            let val = builder(self, element)?;
            VB::list_push(&mut values, val);
            self.consume_if(&Token::Comma);
        }

        Ok(values)
    }

    #[inline]
    fn parse_tuple<SF: flavors::SchemaFlavor<'s>>(
        &mut self,
        schemas: impl ExactSizeIterator<Item = &'s schema::Schema<'s, SF>>,
        mut builder: impl FnMut(
            &mut Self,
            &'s schema::Schema<'s, SF>,
        ) -> Result<value::Value<VB>, super::BuildError<'s, Self::Error>>,
    ) -> Result<VB::List<value::Value<VB>>, super::BuildError<'s, Self::Error>> {
        let count = schemas.len();
        let grouped = count > 1 && self.depth > 0;
        if grouped {
            self.expect(Token::LParen)
                .map_err(super::BuildError::Parser)?;
        }
        if count > 1 {
            self.depth += 1;
        }
        let result =
            self.parse_repeated::<_, _>(schemas, None, |this, schema| builder(this, schema));
        if count > 1 {
            self.depth -= 1;
        }
        let values = result?;
        if grouped {
            self.expect(Token::RParen)
                .map_err(super::BuildError::Parser)?;
        }
        Ok(values)
    }

    #[inline]
    fn parse_option<SF: flavors::SchemaFlavor<'s>>(
        &mut self,
        inner: &'s schema::Schema<'s, SF>,
        mut builder: impl FnMut(
            &mut Self,
            &'s schema::Schema<'s, SF>,
        ) -> Result<value::Value<VB>, super::BuildError<'s, Self::Error>>,
    ) -> Result<Option<VB::Ptr<value::Value<VB>>>, super::BuildError<'s, Self::Error>> {
        if self.consume_if(&Token::Word(Word::Bare("none")))
            || self.consume_if(&Token::Word(Word::Bare("null")))
            || self.tokens.peek().is_none()
        {
            return Ok(None);
        }

        self.expect(Token::Word(Word::Bare("some")))
            .map_err(super::BuildError::Parser)?;
        self.expect(Token::LParen)
            .map_err(super::BuildError::Parser)?;
        let saved_depth = self.depth;
        self.depth = 0;
        let result = builder(self, inner);
        self.depth = saved_depth;
        let val = result?;
        self.expect(Token::RParen)
            .map_err(super::BuildError::Parser)?;
        Ok(Some(VB::make_ptr(val)))
    }

    #[inline]
    fn parse_struct<SF: flavors::SchemaFlavor<'s>>(
        &mut self,
        data: &'s schema::Data<'s, SF>,
        builder: impl FnMut(
            &mut Self,
            &'s schema::Schema<'s, SF>,
        ) -> Result<value::Value<VB>, super::BuildError<'s, Self::Error>>,
    ) -> Result<value::Data<VB>, super::BuildError<'s, Self::Error>> {
        let grouped = self.depth > 0
            && matches!(data, schema::Data::Struct { fields, .. } if fields.len() > 1);

        if grouped {
            self.expect(Token::LParen)
                .map_err(super::BuildError::Parser)?;
        }

        let parsed_data = self.parse_data(data, builder)?;

        if grouped {
            self.expect(Token::RParen)
                .map_err(super::BuildError::Parser)?;
        }

        Ok(parsed_data)
    }

    #[inline]
    fn parse_enum<SF: flavors::SchemaFlavor<'s>>(
        &mut self,
        variants: &'s SF::List<schema::VariantSchema<'s, SF>>,
        builder: impl FnMut(
            &mut Self,
            &'s schema::Schema<'s, SF>,
        ) -> Result<value::Value<VB>, super::BuildError<'s, Self::Error>>,
    ) -> Result<(u32, VB::Str, value::Data<VB>), super::BuildError<'s, Self::Error>> {
        let input_name = <Self as super::Parser<'s, VB>>::next_atom(self)
            .and_then(|atom| {
                atom.word()
                    .and_then(Word::bare)
                    .map_err(|err| self.lift_err(err))
            })
            .map_err(super::BuildError::Parser)?;

        let Some(variant) = (**variants)
            .iter()
            .find(|va| va.name.as_ref().eq_ignore_ascii_case(input_name))
        else {
            return Err(super::BuildError::Parser(
                self.lift_err(ErrorKind::VariantNotFound(input_name)),
            ));
        };
        let schema::VariantSchema {
            discriminant,
            name,
            data,
        } = &**variant;

        let new_path =
            VB::make_str_from_display(&format_args!("{}.{}", self.path.as_ref(), name.as_ref()));
        let saved = mem::replace(&mut self.path, new_path);
        let saved_depth = self.depth;
        self.depth = 0;

        let parsed_data = self.parse_data(data, builder)?;

        self.path = saved;
        self.depth = saved_depth;

        Ok((*discriminant, VB::make_str(name.as_ref()), parsed_data))
    }
}

impl<'i, VB: flavors::ValueBuilder> Parser<'i, VB> {
    fn parse_data<'s, SF: flavors::SchemaFlavor<'s>>(
        &mut self,
        data: &'s schema::Data<'s, SF>,
        mut builder: impl FnMut(
            &mut Self,
            &'s schema::Schema<'s, SF>,
        )
            -> Result<value::Value<VB>, super::BuildError<'s, Error<'i, VB::Str>>>,
    ) -> Result<value::Data<VB>, super::BuildError<'s, Error<'i, VB::Str>>> {
        use super::Parser as _;

        Ok(match data {
            schema::Data::Unit { .. } => value::Data::Unit,
            schema::Data::NewType { field, .. } => value::Data::NewType {
                field: VB::make_ptr(builder(self, field)?),
            },
            schema::Data::Tuple { fields, .. } => {
                let schemas = fields.iter().map(|el| &**el);

                value::Data::Tuple {
                    fields: self.parse_tuple(schemas, &mut builder)?,
                }
            }
            schema::Data::Struct { fields, .. } => {
                if fields.len() > 1 {
                    self.depth += 1;
                }

                let parsed_fields =
                    self.parse_repeated::<_, _>(fields.iter(), None, |this, field_schema| {
                        this.parse_field(field_schema, &mut builder)
                    })?;

                if fields.len() > 1 {
                    self.depth -= 1;
                }

                value::Data::Struct {
                    fields: parsed_fields,
                }
            }
        })
    }

    fn expect(&mut self, expected: Token<'i>) -> Result<(), Error<'i, VB::Str>> {
        let token = <Self as super::Parser<'i, VB>>::next_atom(self)?;

        if token != expected {
            return Err(self.lift_err(ErrorKind::UnexpectedExpectedToken(token, expected)));
        }
        Ok(())
    }

    fn consume_if(&mut self, token: &Token<'i>) -> bool {
        self.tokens
            .next_if(|spanned| matches!(&spanned.value, Ok(value) if value == token))
            .map(|Spanned { span, .. }| self.last_span = span)
            .is_some()
    }

    fn parse_field<'s, SF: flavors::SchemaFlavor<'s>>(
        &mut self,
        field_schema: &'s schema::FieldSchema<'s, SF>,
        builder: &mut impl FnMut(
            &mut Self,
            &'s schema::Schema<'s, SF>,
        )
            -> Result<value::Value<VB>, super::BuildError<'s, Error<'i, VB::Str>>>,
    ) -> Result<(VB::Str, value::Value<VB>), super::BuildError<'s, Error<'i, VB::Str>>> {
        let fname_str = field_schema.name.as_ref();
        let fname = VB::make_str(fname_str);

        let new_path =
            VB::make_str_from_display(&format_args!("{}.{}", self.path.as_ref(), fname_str));
        let saved = mem::replace(&mut self.path, new_path);
        let result = builder(self, &*field_schema.ty);
        self.path = saved;

        Ok((fname, result?))
    }

    fn parse_repeated<'s, T, R: PartialEq + fmt::Debug + Clone>(
        &mut self,
        schemas: impl ExactSizeIterator<Item = T>,
        separator: Option<Token<'i>>,
        mut on_item: impl FnMut(&mut Self, T) -> Result<R, super::BuildError<'s, Error<'i, VB::Str>>>,
    ) -> Result<VB::List<R>, super::BuildError<'s, Error<'i, VB::Str>>> {
        let mut values = VB::list_with_capacity(schemas.len());

        let mut iter = schemas.peekable();
        while let Some(schema) = iter.next() {
            let val = on_item(self, schema)?;
            VB::list_push(&mut values, val);

            if iter.peek().is_some()
                && let Some(sep) = &separator
            {
                self.expect(sep.clone())
                    .map_err(super::BuildError::Parser)?;
            }
        }

        if let Some(sep) = separator {
            self.consume_if(&sep);
        }

        Ok(values)
    }
}
