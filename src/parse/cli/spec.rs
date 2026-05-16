//! Usage help generator from schemas.
//!
//! ```
//! use cecetype::{Schema, flavors::Static, parse::cli::spec::Spec};
//!
//! #[derive(Schema)]
//! struct Request { id: u64, msg: String }
//!
//! #[derive(Schema)]
//! struct Response { ok: bool }
//!
//! let help = Spec::<Static>::new("cmd", "Does something", Request::SCHEMA, Response::SCHEMA).unwrap().to_string();
//! assert_eq!(help, "\
//!cmd -- Does something
//!
//!USAGE:
//!\tcmd <`Request`>
//!
//!TYPES:
//!\tRequest\t<id: <u64>> <msg: <str>>
//!
//!EXAMPLE:
//!\tcmd 0 'example'
//!
//!RESPONSE:
//!\tResponse { ok: bool }
//!
//!");
//! ```
#![expect(
    clippy::cognitive_complexity,
    clippy::shadow_unrelated,
    clippy::shadow_reuse,
    reason = "_"
)]
// TODO: how to support refs? worried about infinite help print
use crate::{
    flavors::SchemaFlavor,
    schema::{Data, Schema, VariantSchema},
};
use ::{
    core::{cell::RefCell, convert::Infallible, error, fmt, iter},
    serde::{Deserialize, Serialize},
};

/// Generates usage help, examples, and type listings from request/response schemas.
#[::derive_where::derive_where(Debug;)]
#[derive(Serialize, Deserialize)]
#[serde(bound(
    serialize = "
        SF::Str: Serialize,
        SF::Ptr<Schema<'s, SF>>: Serialize,
    ",
    deserialize = "
        SF::Str: Deserialize<'de>,
        SF::Ptr<Schema<'s, SF>>: Deserialize<'de>,
        SF: ::cecetype::flavors::OwnedSchemaFlavor<'s>,
    "
))]
#[derive(::cecetype_macros::Schema)]
#[schema(bounds(
    SF::Str: crate::Schema,
    SF::Ptr<Schema<'s, SF>>: crate::Schema,
))]
pub struct Spec<'s, SF: SchemaFlavor<'s>> {
    name: SF::Str,
    description: SF::Str,
    request: SF::Ptr<Schema<'s, SF>>,
    response: SF::Ptr<Schema<'s, SF>>,
}

#[::derive_where::derive_where(Debug;)]
#[derive(::thiserror::Error)]
#[error("Found Ref: '{}'", self.0.as_ref())]
pub struct FoundRef<'s, SF: SchemaFlavor<'s>>(pub SF::Str);

impl<'s, SF: SchemaFlavor<'s>> Spec<'s, SF> {
    /// Create help for request/response schemas.
    ///
    /// Returns `Err(FoundRef)` if the request schema contains `Ref` nodes.
    #[inline]
    pub fn new(
        name: SF::Str,
        description: SF::Str,
        request: SF::Ptr<Schema<'s, SF>>,
        response: SF::Ptr<Schema<'s, SF>>,
    ) -> Result<Self, FoundRef<'s, SF>> {
        find_ref(&request)
            .map(|name| FoundRef(name.clone()))
            .map_or(Ok(()), Err)?;

        Ok(Self {
            name,
            description,
            request,
            response,
        })
    }

    #[inline]
    pub const fn name(&self) -> &SF::Str {
        &self.name
    }

    #[inline]
    pub const fn description(&self) -> &SF::Str {
        &self.description
    }

    #[inline]
    pub fn request(&self) -> &Schema<'s, SF> {
        &self.request
    }

    #[inline]
    pub fn response(&self) -> &Schema<'s, SF> {
        &self.response
    }

    #[inline]
    #[doc(hidden)]
    pub const fn new_unchecked(
        name: SF::Str,
        description: SF::Str,
        request: SF::Ptr<Schema<'s, SF>>,
        response: SF::Ptr<Schema<'s, SF>>,
    ) -> Self {
        Self {
            name,
            description,
            request,
            response,
        }
    }

    #[inline]
    #[must_use]
    #[doc(hidden)]
    pub fn usage(&self) -> impl fmt::Display + '_ {
        ReprMode::Usage.fmt(&self.request, 0)
    }

    #[inline]
    #[must_use]
    #[doc(hidden)]
    pub fn example(&self) -> impl fmt::Display + '_ {
        ReprMode::Example.fmt(&self.request, 0)
    }

    #[inline]
    #[must_use]
    #[doc(hidden)]
    pub fn types(&self) -> Option<impl fmt::Display + '_> {
        let col = max_type_name_length(&self.request)?;
        let mut first = true;

        Some(fmt::from_fn(move |fmt| {
            visit_types(&*self.request, None, &mut move |ns| {
                if !first {
                    write!(fmt, "\n\n")?;
                }
                first = false;

                match ns {
                    NamedSchema::Enum { variants, .. } => {
                        let variants_repr = joined(
                            variants.iter().map(|va| {
                                let variant_col = col - 1;
                                fmt::from_fn(move |fmt| {
                                    write!(
                                        fmt,
                                        "\t\t{:<variant_col$}\t{}",
                                        va.name.as_ref(),
                                        ReprMode::Usage.fmt_data(&va.data, 0)
                                    )
                                })
                            }),
                            '\n',
                        );
                        writeln!(fmt, "\t{:<col$}", ns.name())?;
                        write!(fmt, "{variants_repr}")
                    }
                    NamedSchema::Struct { data, .. } => write!(
                        fmt,
                        "\t{:<col$}\t{}",
                        ns.name(),
                        ReprMode::Usage.fmt_data(data, 0)
                    ),
                }
            })
        }))
    }
}

impl<'s, SF: SchemaFlavor<'s>> fmt::Display for Spec<'s, SF> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} -- {}", self.name.as_ref(), self.description.as_ref())?;
        writeln!(f)?;
        writeln!(f, "USAGE:")?;
        writeln!(f, "\t{} {}", self.name.as_ref(), self.usage())?;
        writeln!(f)?;

        if let Some(types) = self.types() {
            writeln!(f, "TYPES:")?;
            writeln!(f, "{types}")?;
            writeln!(f)?;
        }

        writeln!(f, "EXAMPLE:")?;
        writeln!(f, "\t{} {}", self.name.as_ref(), self.example())?;
        writeln!(f)?;
        writeln!(f, "RESPONSE:")?;
        writeln!(f, "\t{}", &*self.response)?;
        writeln!(f)
    }
}

fn find_ref<'a, 's, SF: SchemaFlavor<'s>>(schema: &'a Schema<'s, SF>) -> Option<&'a SF::Str> {
    let find_ref_data = |data: &'a Data<'s, SF>| match data {
        Data::Unit => None,
        Data::NewType { field, .. } => find_ref(field),
        Data::Tuple { fields, .. } => fields.iter().find_map(|field| find_ref(field)),
        Data::Struct { fields, .. } => fields.iter().find_map(|field| find_ref(&field.ty)),
    };

    match schema {
        Schema::Ref { name, .. } => Some(name),
        Schema::Array { element, .. } | Schema::Slice { element } | Schema::Option(element) => {
            find_ref(element)
        }
        Schema::Map { key, value } => find_ref(key).or_else(|| find_ref(value)),
        Schema::Tuple { elements } => elements.iter().find_map(|el| find_ref(el)),
        Schema::Struct { data, .. } => find_ref_data(data),
        Schema::Enum { variants, .. } => variants.iter().find_map(|va| find_ref_data(&va.data)),
        #[rustfmt::skip]
        Schema::Unit | Schema::Bool | Schema::Str | Schema::Char | Schema::U8 | Schema::U16 | Schema::U32 | Schema::U64 | Schema::I8 | Schema::I16 | Schema::I32 | Schema::I64 | Schema::F32 | Schema::F64 | Schema::U128 | Schema::I128 => None,
    }
}

fn joined(
    iter: impl Iterator<Item = impl fmt::Display>,
    sep: impl fmt::Display,
) -> impl fmt::Display {
    let rc_iter = RefCell::new(iter);
    fmt::from_fn(move |fmt| {
        let mut iter = rc_iter.borrow_mut();
        if let Some(first) = iter.next() {
            write!(fmt, "{first}")?;
            for item in iter.by_ref() {
                write!(fmt, "{sep}{item}")?;
            }
        }
        Ok(())
    })
}

fn maybe_grouped(grouped: bool, thing: impl fmt::Display) -> impl fmt::Display {
    fmt::from_fn(move |fmt| {
        if grouped {
            write!(fmt, "({thing})")
        } else {
            write!(fmt, "{thing}")
        }
    })
}

#[derive(Clone, Copy)]
enum ReprMode {
    Usage,
    Example,
}

macro_rules! repr {
    (
        $self:ident, $fmt:expr,
        usage($u_fmt:expr $(, $u_args:expr)*) ;
        example($e_fmt:expr $(, $e_args:expr)*) $(,)?
    ) => {
        match $self {
            Self::Usage => write!($fmt, $u_fmt $(, $u_args)*),
            Self::Example => write!($fmt, $e_fmt $(, $e_args)*),
        }
    };
}

impl ReprMode {
    fn fmt<'a, 's, SF: SchemaFlavor<'s>>(
        self,
        schema: &'a Schema<'s, SF>,
        depth: usize,
    ) -> impl fmt::Display + 'a {
        fmt::from_fn(move |fmt| match schema {
            Schema::Ref { .. } => unreachable!(),
            Schema::Unit => repr!(self, fmt, usage("<void>"); example("")),
            Schema::Bool => repr!(self, fmt, usage("<bool>"); example("true")),
            Schema::Char => repr!(self, fmt, usage("<char>"); example("'x'")),
            Schema::Str => repr!(self, fmt, usage("<str>"); example("'example'")),
            Schema::U8 => repr!(self, fmt, usage("<u8>"); example("0")),
            Schema::U16 => repr!(self, fmt, usage("<u16>"); example("0")),
            Schema::U32 => repr!(self, fmt, usage("<u32>"); example("0")),
            Schema::U64 => repr!(self, fmt, usage("<u64>"); example("0")),
            Schema::U128 => repr!(self, fmt, usage("<u128>"); example("0")),
            Schema::I8 => repr!(self, fmt, usage("<i8>"); example("0")),
            Schema::I16 => repr!(self, fmt, usage("<i16>"); example("0")),
            Schema::I32 => repr!(self, fmt, usage("<i32>"); example("0")),
            Schema::I64 => repr!(self, fmt, usage("<i64>"); example("0")),
            Schema::I128 => repr!(self, fmt, usage("<i128>"); example("0")),
            Schema::F32 => repr!(self, fmt, usage("<f32>"); example("1.0")),
            Schema::F64 => repr!(self, fmt, usage("<f64>"); example("1.0")),
            Schema::Slice { element } => {
                repr!(self, fmt, usage("[{}...]", self.fmt(element, 0)); example("[{}]", self.fmt(element, 0)))
            }
            Schema::Map { key, value } => repr!(
                self, fmt,
                usage("{{ {}: {} }}", self.fmt(key, 0), self.fmt(value, 0));
                example("{{ {}: {} }}", self.fmt(key, 0), self.fmt(value, 0)),
            ),
            Schema::Tuple { elements } => {
                let next_depth = if elements.len() > 1 { depth + 1 } else { depth };
                repr!(
                    self, fmt,
                    usage("({})", joined(elements.iter().map(|el| self.fmt(el, next_depth)), " "));
                    example("{}", maybe_grouped(depth > 0 && elements.len() > 1, joined(elements.iter().filter(|el| !matches!(***el, Schema::Unit | Schema::Struct { data: Data::Unit, .. })).map(|el| self.fmt(el, next_depth)), " "))),
                )
            }
            Schema::Array { element, len } => repr!(
                self, fmt,
                usage("[{}]", joined(iter::repeat_n(element, *len).map(|el| self.fmt(el, 0)), ", "));
                example("[{}]", joined(iter::repeat_n(element, *len).map(|el| self.fmt(el, 0)), ", ")),
            ),
            Schema::Option(element) => {
                repr!(self, fmt, usage("{}?", self.fmt(element, 0)); example("some({})", self.fmt(element, 0)))
            }
            Schema::Enum { name, variants } => {
                let Some(example_variant) = variants
                    .iter()
                    .find(|va| !matches!(va.data, Data::Unit))
                    .or_else(|| variants.first())
                else {
                    // TODO: dunno, don't like
                    return repr!(self, fmt, usage("<`{}`>", name.as_ref()); example(""));
                };
                repr!(
                    self, fmt,
                    usage("<`{}`>", name.as_ref());
                    example("{} {}", example_variant.name.as_ref(), self.fmt_data(&example_variant.data, 0))
                )
            }
            Schema::Struct { name, data } => {
                let grouped =
                    depth > 0 && matches!(data, Data::Struct { fields, .. } if fields.len() > 1);
                repr!(
                    self, fmt,
                    usage("<`{}`>", name.as_ref());
                    example("{}", maybe_grouped(grouped, self.fmt_data(data, depth))),
                )
            }
        })
    }

    fn fmt_data<'a, 's, SF: SchemaFlavor<'s>>(
        self,
        data: &'a Data<'s, SF>,
        depth: usize,
    ) -> impl fmt::Display + 'a {
        fmt::from_fn(move |fmt| match data {
            Data::Unit => Ok(()),
            Data::NewType { field } => write!(fmt, "{}", self.fmt(field, 0)),
            Data::Tuple { fields, .. } => repr!(
                self, fmt,
                usage("{}", joined(fields.iter().map(|fi| self.fmt(fi, 0)), " "));
                example("{}", joined(fields.iter().filter(|fi| !matches!(***fi, Schema::Unit | Schema::Struct { data: Data::Unit, .. })).map(|fi| self.fmt(fi, depth + usize::from(fields.len() > 1))), " ")),
            ),
            Data::Struct { fields, .. } => repr!(
                self, fmt,
                usage("{}", joined(fields.iter().map(|fi| fmt::from_fn(|fmt| write!(fmt, "<{}: {}>", fi.name.as_ref(), self.fmt(&fi.ty, 0)))), " "));
                example("{}", joined(fields.iter().filter(|fi| !matches!(&*fi.ty, Schema::Unit | Schema::Struct { data: Data::Unit, .. })).map(|fi| self.fmt(&fi.ty, depth + usize::from(fields.len() > 1))), " ")),
            ),
        })
    }
}

#[::derive_where::derive_where(PartialEq; )]
enum NamedSchema<'a, 's, SF: SchemaFlavor<'s>> {
    Enum {
        name: &'a SF::Str,
        variants: &'a SF::List<VariantSchema<'s, SF>>,
    },
    Struct {
        name: &'a SF::Str,
        data: &'a Data<'s, SF>,
    },
}

impl<'s, SF: SchemaFlavor<'s>> NamedSchema<'_, 's, SF> {
    fn name(&self) -> &str {
        match self {
            Self::Enum { name, .. } | Self::Struct { name, .. } => name.as_ref(),
        }
    }
}

struct Seen<'node, 'a, 's, SF: SchemaFlavor<'s>> {
    value: NamedSchema<'a, 's, SF>,
    prev: Option<&'node Self>,
}

impl<'s, SF: SchemaFlavor<'s>> Seen<'_, '_, 's, SF> {
    fn contains(&self, ns: &NamedSchema<'_, 's, SF>) -> bool {
        self.value == *ns || self.prev.is_some_and(|prev| prev.contains(ns))
    }
}

fn visit_types<'node, 'a, 's, SF: SchemaFlavor<'s>, E: error::Error>(
    schema: &'a Schema<'s, SF>,
    seen: Option<&'node Seen<'node, 'a, 's, SF>>,
    visitor: &mut impl FnMut(&NamedSchema<'a, 's, SF>) -> Result<(), E>,
) -> Result<(), E> {
    let visit_data = &mut |data: &'a Data<'s, SF>, seen, visitor: &mut _| match data {
        Data::Unit => Ok(()),
        Data::NewType { field } => visit_types(field, seen, visitor),
        Data::Tuple { fields } => {
            for fi in fields.iter() {
                visit_types(fi, seen, visitor)?;
            }
            Ok(())
        }
        Data::Struct { fields } => {
            for fi in fields.iter() {
                visit_types(&fi.ty, seen, visitor)?;
            }
            Ok(())
        }
    };

    match schema {
        Schema::Struct { name, data } => {
            let ns = NamedSchema::Struct { name, data };
            if seen.is_some_and(|seen| seen.contains(&ns)) {
                return Ok(());
            }
            visitor(&ns)?;
            let seen = Seen {
                value: ns,
                prev: seen,
            };
            visit_data(data, Some(&seen), visitor)
        }
        Schema::Enum { name, variants } => {
            let ns = NamedSchema::Enum { name, variants };
            if seen.is_some_and(|seen| seen.contains(&ns)) {
                return Ok(());
            }
            visitor(&ns)?;
            let seen = Seen {
                value: ns,
                prev: seen,
            };
            for va in variants.iter() {
                visit_data(&va.data, Some(&seen), visitor)?;
            }
            Ok(())
        }
        Schema::Array { element, .. } | Schema::Slice { element } | Schema::Option(element) => {
            visit_types(element, seen, visitor)
        }
        Schema::Map { key, value } => {
            visit_types(key, seen, visitor)?;
            visit_types(value, seen, visitor)
        }
        Schema::Tuple { elements } => {
            for el in elements.iter() {
                visit_types(el, seen, visitor)?;
            }
            Ok(())
        }
        #[rustfmt::skip]
        Schema::Ref { .. } | Schema::Unit | Schema::Bool | Schema::Str | Schema::Char | Schema::U8 | Schema::U16 | Schema::U32 | Schema::U64 | Schema::I8 | Schema::I16 | Schema::I32 | Schema::I64 | Schema::F32 | Schema::F64 | Schema::U128 | Schema::I128 => Ok(()),
    }
}

fn max_type_name_length<'a, 's, SF: SchemaFlavor<'s>>(schema: &'a Schema<'s, SF>) -> Option<usize> {
    let mut max = 0;
    let mut saw_any = false;
    let Ok(()) = visit_types(schema, None, &mut |ns| {
        saw_any = true;
        max = max.max(ns.name().len());
        if let NamedSchema::Enum { variants, .. } = ns {
            for va in variants.iter() {
                max = max.max(va.name.as_ref().len() + '\t'.len_utf16());
            }
        }
        Result::<(), Infallible>::Ok(())
    });
    saw_any.then_some(max)
}
