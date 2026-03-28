use ::syn::punctuated;

// Won't support
// - `rename(...)` / `rename_all(...)` / `rename_all_fields(...)` variations
// - `default` as we can't (won't) send the default value
#[derive(Default)]
pub struct ContainerAttrs {
    pub rename: Option<::syn::LitStr>,
    pub repr_via: Option<::syn::ExprPath>, // `into` + `from`/`try_from`
    pub transparent: bool,
} // rename_all, rename_all_fields, tag, content, untagged

// Won't support
// - `rename(...)` / `rename_all(...)` variations
// - `alias`
// - `with` / `serialize_with` / `deserialize_with` as we can't send the functions used for serialization
// - `other` as it can't really be represented
// - `skip_serializing` / `skip_deserializing` as it would cause a desync
#[derive(Default)]
pub struct VariantAttrs {
    pub rename: Option<::syn::LitStr>,
    pub skip: bool,
} // rename_all, untagged

// Won't support
// - `rename(...)` variations
// - `alias`
// - `with` / `serialize_with` / `deserialize_with` as we can't send the functions used for serialization
// - `default` as we can't (won't) send the default value
// - `skip_serializing` / `skip_deserializing` / `skip_serializing_if` as it would cause a desync
// Dunno
// - `flatten` is probably too hard
#[derive(Default)]
pub struct FieldAttrs {
    pub rename: Option<::syn::LitStr>,
    pub skip: bool,
}

impl ContainerAttrs {
    pub fn parse(attrs: &[::syn::Attribute]) -> ::syn::Result<Self> {
        let mut out = Self::default();

        let mut from_ty = None;
        let mut into_ty = None;

        for attr in attrs {
            if !attr.path().is_ident("serde") {
                continue;
            }

            let nested = attr.parse_args_with(
                punctuated::Punctuated::<::syn::Meta, ::syn::Token![,]>::parse_terminated,
            )?;

            for meta in nested {
                match meta {
                    ::syn::Meta::NameValue(::syn::MetaNameValue {
                        path,
                        value:
                            ::syn::Expr::Lit(::syn::ExprLit {
                                lit: ::syn::Lit::Str(rename),
                                ..
                            }),
                        ..
                    }) if path.is_ident("rename") => out.rename = Some(rename),

                    ::syn::Meta::NameValue(::syn::MetaNameValue {
                        path,
                        value:
                            ::syn::Expr::Lit(::syn::ExprLit {
                                lit: ::syn::Lit::Str(ty),
                                ..
                            }),
                        ..
                    }) if path.is_ident("from") || path.is_ident("try_from") => {
                        if from_ty.is_some() {
                            return Err(::syn::Error::new_spanned(
                                &ty,
                                "Schema: only one of `from` / `try_from` can be used at a time",
                            ));
                        }

                        from_ty = Some(ty);
                    }

                    ::syn::Meta::NameValue(::syn::MetaNameValue {
                        path,
                        value:
                            ::syn::Expr::Lit(::syn::ExprLit {
                                lit: ::syn::Lit::Str(ty),
                                ..
                            }),
                        ..
                    }) if path.is_ident("into") => {
                        into_ty = Some(ty);
                    }

                    ::syn::Meta::Path(path) if path.is_ident("transparent") => {
                        out.transparent = true;
                    }

                    // -------- explicitly reject rename(...)
                    ::syn::Meta::List(list) if list.path.is_ident("rename") => {
                        return Err(::syn::Error::new_spanned(
                            &list.path,
                            "Schema: `rename(...)` not supported, use `rename = \"...\"`",
                        ));
                    }

                    _ if meta.path().is_ident("bound")
                        || meta.path().is_ident("remote")
                        || meta.path().is_ident("crate")
                        || meta.path().is_ident("expecting")
                        || meta.path().is_ident("deny_unknown_fields") => {}

                    other @ (::syn::Meta::Path(_)
                    | ::syn::Meta::List(_)
                    | ::syn::Meta::NameValue(_)) => {
                        return Err(::syn::Error::new_spanned(
                            other,
                            "Schema: unsupported serde attribute",
                        ));
                    }
                }
            }
        }

        match (into_ty, from_ty) {
            (None, None) => {}
            (Some(ity), Some(fty)) => {
                if ity != fty {
                    return Err(::syn::Error::new(
                        ::proc_macro2::Span::call_site(),
                        "Schema: `into` and `from`/`try_from` must use the same type",
                    ));
                }
                out.repr_via = Some(ity.parse()?);
            }
            _ => {
                return Err(::syn::Error::new(
                    ::proc_macro2::Span::call_site(),
                    "Schema: You must use `into` + `from`/`try_from` together",
                ));
            }
        }

        Ok(out)
    }
}

impl VariantAttrs {
    pub fn parse(attrs: &[::syn::Attribute]) -> ::syn::Result<Self> {
        let mut out = Self::default();

        for attr in attrs {
            if !attr.path().is_ident("serde") {
                continue;
            }

            let nested = attr.parse_args_with(
                punctuated::Punctuated::<::syn::Meta, ::syn::Token![,]>::parse_terminated,
            )?;

            for meta in nested {
                match meta {
                    ::syn::Meta::NameValue(::syn::MetaNameValue {
                        path,
                        value:
                            ::syn::Expr::Lit(::syn::ExprLit {
                                lit: ::syn::Lit::Str(rename),
                                ..
                            }),
                        ..
                    }) if path.is_ident("rename") => {
                        out.rename = Some(rename);
                    }

                    ::syn::Meta::Path(path) if path.is_ident("skip") => {
                        out.skip = true;
                    }

                    ::syn::Meta::List(list) if list.path.is_ident("rename") => {
                        return Err(::syn::Error::new_spanned(
                            &list.path,
                            "Schema: `rename(...)` not supported, use `rename = \"...\"`",
                        ));
                    }

                    _ if meta.path().is_ident("bound") || meta.path().is_ident("borrow") => {}

                    other @ (::syn::Meta::Path(_)
                    | ::syn::Meta::List(_)
                    | ::syn::Meta::NameValue(_)) => {
                        return Err(::syn::Error::new_spanned(
                            other,
                            "Schema: unsupported serde attribute",
                        ));
                    }
                }
            }
        }

        Ok(out)
    }
}

// ----------------------------
// Field
// ----------------------------

impl FieldAttrs {
    pub fn parse(attrs: &[::syn::Attribute]) -> ::syn::Result<Self> {
        let mut out = Self::default();

        for attr in attrs {
            if !attr.path().is_ident("serde") {
                continue;
            }

            let nested = attr.parse_args_with(
                punctuated::Punctuated::<::syn::Meta, ::syn::Token![,]>::parse_terminated,
            )?;

            for meta in nested {
                match meta {
                    ::syn::Meta::NameValue(::syn::MetaNameValue {
                        path,
                        value:
                            ::syn::Expr::Lit(::syn::ExprLit {
                                lit: ::syn::Lit::Str(rename),
                                ..
                            }),
                        ..
                    }) if path.is_ident("rename") => {
                        out.rename = Some(rename);
                    }

                    ::syn::Meta::Path(path) if path.is_ident("skip") => {
                        out.skip = true;
                    }

                    ::syn::Meta::Path(path)
                        if path.is_ident("borrow")
                            || path.is_ident("bound")
                            || path.is_ident("getter") => {}

                    ::syn::Meta::List(list)
                        if list.path.is_ident("borrow")
                            || list.path.is_ident("bound")
                            || list.path.is_ident("getter") => {}

                    ::syn::Meta::NameValue(nv)
                        if nv.path.is_ident("borrow")
                            || nv.path.is_ident("bound")
                            || nv.path.is_ident("getter") => {}

                    other @ (::syn::Meta::Path(_)
                    | ::syn::Meta::List(_)
                    | ::syn::Meta::NameValue(_)) => {
                        return Err(::syn::Error::new_spanned(
                            other,
                            "Schema: unsupported serde attribute",
                        ));
                    }
                }
            }
        }

        Ok(out)
    }
}
