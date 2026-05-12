pub mod help;
mod lexer;
mod word;

use self::{
    lexer::{LexError, Token, Tokens},
    word::{ParseError, Word},
};
use crate::{flavors, schema, value};
use ::core::{fmt, iter::Peekable, mem};

#[derive(Debug, ::thiserror::Error)]
#[expect(
    clippy::error_impl_error,
    reason = "public CLI error type is intentionally named Error"
)]
#[error("at {path}: {kind}")]
pub struct Error<'input, S> {
    pub path: S,
    pub kind: ErrorKind<'input>,
}

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

#[derive(Debug)]
pub struct Parser<'input, VB: flavors::ValueBuilder> {
    tokens: Peekable<Tokens<'input>>,
    path: VB::Str,
    depth: u32,
}

impl<'input, VB: flavors::ValueBuilder> Parser<'input, VB> {
    #[inline]
    #[must_use]
    pub fn new(input: &'input str) -> Self {
        Self {
            tokens: Tokens::new(input).peekable(),
            path: VB::make_str("<root>"),
            depth: 0,
        }
    }

    fn lift_err(&self, err: ErrorKind<'input>) -> Error<'input, VB::Str> {
        Error {
            path: VB::make_str(self.path.as_ref()),
            kind: err,
        }
    }
}

impl<'input, 's, VB: flavors::ValueBuilder> super::Parser<'s, VB> for Parser<'input, VB> {
    type Atom = Word<'input>;
    type Error = Error<'input, VB::Str>;

    #[inline]
    fn next_atom(&mut self) -> Result<Self::Atom, Self::Error> {
        let Some(tok) = self.tokens.next() else {
            return Err(self.lift_err(ErrorKind::EOF));
        };

        match tok
            .map_err(ErrorKind::Lex)
            .map_err(|err| self.lift_err(err))?
        {
            Token::Word(word) => Ok(word),
            utok @ (Token::LParen
            | Token::RParen
            | Token::LBracket
            | Token::RBracket
            | Token::LBrace
            | Token::RBrace
            | Token::Comma
            | Token::Colon
            | Token::Eq) => Err(self.lift_err(ErrorKind::UnexpectedToken(utok))),
        }
    }

    #[inline]
    fn parse_unit(&mut self) -> Result<(), Self::Error> {
        if self.consume_if(Token::LParen) {
            self.expect(Token::RParen)?;
        }
        Ok(())
    }

    #[inline]
    fn finish(&mut self) -> Result<(), Self::Error> {
        match self.tokens.next() {
            None => Ok(()),
            Some(Ok(tok)) => Err(self.lift_err(ErrorKind::UnexpectedToken(tok))),
            Some(Err(err)) => Err(self.lift_err(err.into())),
        }
    }

    #[inline]
    fn parse_bool(&mut self) -> Result<bool, Self::Error> {
        <Self as super::Parser<'s, VB>>::next_atom(self)?
            .parse_bool()
            .map_err(|err| self.lift_err(err))
    }

    #[inline]
    fn parse_string(&mut self) -> Result<impl AsRef<str>, Self::Error> {
        <Self as super::Parser<'s, VB>>::next_atom(self)?
            .parse_string()
            .map_err(|err| self.lift_err(err))
    }
    #[inline]
    fn parse_char(&mut self) -> Result<char, Self::Error> {
        <Self as super::Parser<'s, VB>>::next_atom(self)?
            .parse_char()
            .map_err(|err| self.lift_err(err))
    }

    #[inline]
    fn parse_u8(&mut self) -> Result<u8, Self::Error> {
        <Self as super::Parser<'s, VB>>::next_atom(self)?
            .parse_u8()
            .map_err(|err| self.lift_err(err))
    }
    #[inline]
    fn parse_u16(&mut self) -> Result<u16, Self::Error> {
        <Self as super::Parser<'s, VB>>::next_atom(self)?
            .parse_u16()
            .map_err(|err| self.lift_err(err))
    }
    #[inline]
    fn parse_u32(&mut self) -> Result<u32, Self::Error> {
        <Self as super::Parser<'s, VB>>::next_atom(self)?
            .parse_u32()
            .map_err(|err| self.lift_err(err))
    }
    #[inline]
    fn parse_u64(&mut self) -> Result<u64, Self::Error> {
        <Self as super::Parser<'s, VB>>::next_atom(self)?
            .parse_u64()
            .map_err(|err| self.lift_err(err))
    }
    #[inline]
    fn parse_u128(&mut self) -> Result<u128, Self::Error> {
        <Self as super::Parser<'s, VB>>::next_atom(self)?
            .parse_u128()
            .map_err(|err| self.lift_err(err))
    }

    #[inline]
    fn parse_i8(&mut self) -> Result<i8, Self::Error> {
        <Self as super::Parser<'s, VB>>::next_atom(self)?
            .parse_i8()
            .map_err(|err| self.lift_err(err))
    }
    #[inline]
    fn parse_i16(&mut self) -> Result<i16, Self::Error> {
        <Self as super::Parser<'s, VB>>::next_atom(self)?
            .parse_i16()
            .map_err(|err| self.lift_err(err))
    }
    #[inline]
    fn parse_i32(&mut self) -> Result<i32, Self::Error> {
        <Self as super::Parser<'s, VB>>::next_atom(self)?
            .parse_i32()
            .map_err(|err| self.lift_err(err))
    }
    #[inline]
    fn parse_i64(&mut self) -> Result<i64, Self::Error> {
        <Self as super::Parser<'s, VB>>::next_atom(self)?
            .parse_i64()
            .map_err(|err| self.lift_err(err))
    }
    #[inline]
    fn parse_i128(&mut self) -> Result<i128, Self::Error> {
        <Self as super::Parser<'s, VB>>::next_atom(self)?
            .parse_i128()
            .map_err(|err| self.lift_err(err))
    }

    #[inline]
    fn parse_f32(&mut self) -> Result<f32, Self::Error> {
        <Self as super::Parser<'s, VB>>::next_atom(self)?
            .parse_f32()
            .map_err(|err| self.lift_err(err))
    }
    #[inline]
    fn parse_f64(&mut self) -> Result<f64, Self::Error> {
        <Self as super::Parser<'s, VB>>::next_atom(self)?
            .parse_f64()
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

        while !self.consume_if(Token::RBrace) {
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
            self.consume_if(Token::Comma);
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

        while !self.consume_if(Token::RBracket) {
            let val = builder(self, element)?;
            VB::list_push(&mut values, val);
            self.consume_if(Token::Comma);
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
        if self.consume_if(Token::Word(Word::Bare("none")))
            || self.consume_if(Token::Word(Word::Bare("null")))
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
        variants: &'s SF::List<(u32, SF::Str, schema::Data<'s, SF>)>,
        builder: impl FnMut(
            &mut Self,
            &'s schema::Schema<'s, SF>,
        ) -> Result<value::Value<VB>, super::BuildError<'s, Self::Error>>,
    ) -> Result<(u32, VB::Str, value::Data<VB>), super::BuildError<'s, Self::Error>> {
        let input_name = <Self as super::Parser<'s, VB>>::next_atom(self)
            .and_then(|atom| atom.bare().map_err(|err| self.lift_err(err)))
            .map_err(super::BuildError::Parser)?;

        let Some((discriminant, variant_name, variant_data)) = variants
            .iter()
            .map(|variant_ptr| (&variant_ptr.0, variant_ptr.1.as_ref(), &variant_ptr.2))
            .find(|(_, name, _)| name.eq_ignore_ascii_case(input_name))
        else {
            return Err(super::BuildError::Parser(
                self.lift_err(ErrorKind::VariantNotFound(input_name)),
            ));
        };

        let new_path =
            VB::make_str_from_display(&format_args!("{}.{}", self.path.as_ref(), variant_name));
        let saved = mem::replace(&mut self.path, new_path);
        let saved_depth = self.depth;
        self.depth = 0;

        let data = self.parse_data(variant_data, builder)?;

        self.path = saved;
        self.depth = saved_depth;

        Ok((*discriminant, VB::make_str(variant_name), data))
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

    fn expect(&mut self, token: Token<'i>) -> Result<(), Error<'i, VB::Str>> {
        let tok = self
            .tokens
            .next()
            .ok_or_else(|| self.lift_err(ErrorKind::EOF))?
            .map_err(|err| self.lift_err(err.into()))?;

        if tok != token {
            return Err(self.lift_err(ErrorKind::UnexpectedExpectedToken(tok, token)));
        }
        Ok(())
    }

    fn consume_if(&mut self, token: Token<'i>) -> bool {
        self.tokens.next_if_eq(&Ok(token)).is_some()
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
            self.consume_if(sep);
        }

        Ok(values)
    }
}
