//! Tokenizer for CLI input.

use super::word::Word;
use ::core::{fmt, ops::Range};

#[derive(Debug, PartialEq, Eq, Clone)]
pub(super) struct Spanned<T> {
    pub value: T,
    pub span: Range<usize>,
}

/// Tokens produced by the lexer.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token<'w> {
    Word(Word<'w>),

    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Comma,
    Colon,
    Eq,
}

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Word(word) => word.fmt(f),
            Self::LParen => '('.fmt(f),
            Self::RParen => ')'.fmt(f),
            Self::LBracket => '['.fmt(f),
            Self::RBracket => ']'.fmt(f),
            Self::LBrace => '{'.fmt(f),
            Self::RBrace => '}'.fmt(f),
            Self::Comma => ','.fmt(f),
            Self::Colon => ':'.fmt(f),
            Self::Eq => '='.fmt(f),
        }
    }
}

#[derive(Debug, ::thiserror::Error, PartialEq, Eq)]
pub enum LexError {
    #[error("unexpected end of input")]
    UnexpectedEOF,
}

#[derive(Debug)]
pub(super) struct Tokens<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Tokens<'a> {
    pub const fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }
}

impl<'a> Iterator for Tokens<'a> {
    type Item = Spanned<Result<Token<'a>, LexError>>;

    fn next(&mut self) -> Option<Self::Item> {
        let bytes = self.input.as_bytes();

        while bytes.get(self.pos).is_some_and(u8::is_ascii_whitespace) {
            self.pos += 1;
        }

        let token_start = self.pos;
        let tok = match *bytes.get(self.pos)? {
            b'(' => Token::LParen,
            b')' => Token::RParen,
            b'[' => Token::LBracket,
            b']' => Token::RBracket,
            b'{' => Token::LBrace,
            b'}' => Token::RBrace,
            b',' => Token::Comma,
            b':' => Token::Colon,
            b'=' => Token::Eq,
            quote @ (b'"' | b'\'') => {
                self.pos += 1;
                let start = self.pos;

                loop {
                    match bytes.get(self.pos) {
                        None => {
                            return Some(Spanned {
                                value: Err(LexError::UnexpectedEOF),
                                span: token_start..self.input.len(),
                            });
                        }
                        Some(b'\\') => self.pos += 2,
                        Some(&ch) if ch == quote => break,
                        Some(_) => self.pos += 1,
                    }
                }
                #[expect(clippy::string_slice, reason = "bound checked")]
                Token::Word(Word::Quoted(&self.input[start..self.pos]))
            }
            _ => {
                let start = self.pos;
                while bytes.get(self.pos + 1).is_some_and(|&ch| is_bare_char(ch)) {
                    self.pos += 1;
                }
                #[expect(clippy::string_slice, reason = "bound checked")]
                Token::Word(Word::Bare(&self.input[start..=self.pos]))
            }
        };

        self.pos += 1;
        Some(Spanned {
            value: Ok(tok),
            span: token_start..self.pos,
        })
    }
}

#[inline]
const fn is_bare_char(ch: u8) -> bool {
    !ch.is_ascii_whitespace()
        && !matches!(
            ch,
            b'(' | b')' | b'[' | b']' | b'{' | b'}' | b',' | b':' | b'=' | b'"' | b'\''
        )
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "tests")]
mod tests {
    use super::*;
    use ::core::array;

    #[rstest::rstest]
    #[case::simple_words("hello world", [Token::Word(Word::Bare("hello")), Token::Word(Word::Bare("world"))])]
    #[case::ignores_whitespace("  a\tb\nc ", [Token::Word(Word::Bare("a")), Token::Word(Word::Bare("b")), Token::Word(Word::Bare("c"))])]
    #[case::adjacent_delims("a(b)c", [Token::Word(Word::Bare("a")), Token::LParen, Token::Word(Word::Bare("b")), Token::RParen, Token::Word(Word::Bare("c"))])]
    #[case::quotes(r#" "hello world" 'abc' 'toto \n tata' "#, [Token::Word(Word::Quoted("hello world")), Token::Word(Word::Quoted("abc")), Token::Word(Word::Quoted("toto \\n tata"))])]
    #[case::quote_adjacent_to_word("a'b c'd", [Token::Word(Word::Bare("a")), Token::Word(Word::Quoted("b c")), Token::Word(Word::Bare("d"))])]
    #[case::map_like("{hello:1,world:2}", [Token::LBrace, Token::Word(Word::Bare("hello")), Token::Colon, Token::Word(Word::Bare("1")), Token::Comma, Token::Word(Word::Bare("world")), Token::Colon, Token::Word(Word::Bare("2")), Token::RBrace])]
    #[case::float("1.234", [Token::Word(Word::Bare("1.234"))])]
    #[case::float_quoted("'1.234'", [Token::Word(Word::Quoted("1.234"))])]
    fn lex<const N: usize>(#[case] input: &str, #[case] expected_tokens: [Token; N]) {
        let mut toks = Tokens::new(input);
        assert_eq!(
            array::from_fn(|_| toks.next().unwrap().value.unwrap()),
            expected_tokens
        );
        assert!(toks.next().is_none());
    }

    #[rstest::rstest]
    #[case::unterminated_quotes(" 'hello ", LexError::UnexpectedEOF)]
    #[case::escape_at_end_quote("'a\\", LexError::UnexpectedEOF)]
    #[case::escape_endquote("'a\\'", LexError::UnexpectedEOF)]
    fn lex_error(#[case] input: &str, #[case] expected_error: LexError) {
        assert_eq!(
            Tokens::new(input).next().unwrap().value.unwrap_err(),
            expected_error
        );
    }
}
