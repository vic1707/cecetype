#![cfg_attr(doc, doc = include_str!("../README.md"))]
#![expect(clippy::unwrap_used, reason = "wip")]

mod attrs;

use self::attrs::{ContainerAttrs, FieldAttrs, RefAttrKind, VariantAttrs};
use ::{
    quote::quote,
    syn::{self, Fields},
};

/// Derive macro for [`::cecetype::Schema`].
///
/// See [cecetype-macros](https://docs.rs/cecetype-macros) for usage.
#[proc_macro_derive(Schema, attributes(serde, schema))]
#[inline]
pub fn derive_schema(input: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
    expand(input)
        // .inspect(|o| eprintln!("{o}"))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

fn expand(input: ::proc_macro::TokenStream) -> ::syn::Result<::proc_macro2::TokenStream> {
    let ::syn::DeriveInput {
        data,
        ident,
        mut generics,
        attrs,
        ..
    } = ::syn::parse(input)?;

    let container_attrs = ContainerAttrs::parse(&attrs)?;

    let name = container_attrs
        .rename
        .as_ref()
        .map_or_else(|| ident.to_string(), ::syn::LitStr::value);

    let schema = container_attrs.repr_via.as_ref().map_or(
        match &data {
            ::syn::Data::Struct(data_struct) => struct_schema(&name, data_struct, &container_attrs),
            ::syn::Data::Enum(data_enum) => enum_schema(&name, data_enum, &container_attrs),
            ::syn::Data::Union(_) => {
                return Err(::syn::Error::new_spanned(
                    ident,
                    "Schema derive only supports struct and enum",
                ));
            }
        },
        |ty| Ok(::syn::parse_quote! { <#ty as ::cecetype::Schema>::SCHEMA }),
    )?;

    let schema_bounds = container_attrs.bounds.map_or_else(
        || {
            generics
                .type_params()
                .map(
                    |syn::TypeParam { ident: ident2, .. }| ::syn::parse_quote! { #ident2: ::cecetype::Schema },
                )
                .collect::<Vec<::syn::WherePredicate>>()
        },
        |bounds| bounds.into_iter().collect::<Vec<_>>(),
    );

    let where_clause_ref = generics.make_where_clause();
    for bound in schema_bounds {
        where_clause_ref.predicates.push(bound);
    }

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    if let Some(ref_attr) = container_attrs.references {
        let ref_name = ref_attr.name.to_string();
        let kind = match ref_attr.kind {
            RefAttrKind::Direct => quote! { ::cecetype::schema::RefKind::Direct },
            RefAttrKind::List => quote! { ::cecetype::schema::RefKind::Slice },
        };
        return Ok(quote! {
            impl #impl_generics ::cecetype::Schema for #ident #ty_generics #where_clause {
                const SCHEMA: &'static ::cecetype::StaticSchema = &::cecetype::schema::Schema::Ref { name: #ref_name, kind: #kind };
            }
        });
    }

    Ok(quote! {
        impl #impl_generics ::cecetype::Schema for #ident #ty_generics #where_clause {
            const SCHEMA: &'static ::cecetype::StaticSchema = &#schema;
        }
    })
}

fn struct_schema(
    struct_name: &str,
    data: &::syn::DataStruct,
    container_attrs: &ContainerAttrs,
) -> ::syn::Result<::syn::Expr> {
    let fields = data
        .fields
        .iter()
        .map(|field| Ok((field, FieldAttrs::parse(&field.attrs)?)))
        .filter(|res| res.as_ref().is_ok_and(|(_, attrs)| !attrs.skip))
        .collect::<::syn::Result<Vec<_>>>()?;

    if container_attrs.transparent {
        let (field, field_attrs) = fields.first().unwrap();
        let ty = field_attrs.repr_via.as_ref().unwrap_or(&field.ty);
        return Ok(::syn::parse_quote! { <#ty as ::cecetype::Schema>::SCHEMA });
    }

    let mut field_defs = fields.into_iter().map(field_schema);

    let schema = match &data.fields {
        Fields::Named(_) => {
            ::syn::parse_quote! {
                ::cecetype::schema::Schema::Struct{
                    name: #struct_name,
                    data: ::cecetype::schema::Data::Struct {
                        fields: &[
                            #( #field_defs ),*
                        ],
                    }
                }
            }
        }

        Fields::Unnamed(_) if field_defs.len() == 1 => {
            let field = field_defs.next().unwrap();
            ::syn::parse_quote! {
                ::cecetype::schema::Schema::Struct{
                    name: #struct_name,
                    data: ::cecetype::schema::Data::NewType {
                        field: #field,
                    }
                }
            }
        }

        Fields::Unnamed(_) => {
            ::syn::parse_quote! {
                ::cecetype::schema::Schema::Struct {
                    name: #struct_name,
                    data: ::cecetype::schema::Data::Tuple {
                        fields: &[
                            #( #field_defs ),*
                        ],
                    }
                }
            }
        }

        Fields::Unit => {
            ::syn::parse_quote! {
                ::cecetype::schema::Schema::Struct{
                    name: #struct_name,
                    data: ::cecetype::schema::Data::Unit,
                }
            }
        }
    };

    Ok(schema)
}

fn enum_schema(
    enum_name: &str,
    data: &::syn::DataEnum,
    _container_attrs: &ContainerAttrs,
) -> ::syn::Result<::syn::Expr> {
    let variants = data
        .variants
        .iter()
        .map(|variant| Ok((variant, VariantAttrs::parse(&variant.attrs)?)))
        .collect::<::syn::Result<Vec<_>>>()?
        .iter()
        .enumerate() // serde keeps original discriminants
        .filter(|(_, (_, attrs))| !attrs.skip)
        .map(
            |(
                i,
                (
                    ::syn::Variant {
                        ident: vident,
                        fields: vfields,
                        ..
                    },
                    variant_attrs,
                ),
            )| {
                let vname = variant_attrs
                    .rename
                    .as_ref()
                    .map_or_else(|| vident.to_string(), ::syn::LitStr::value);

                let discriminant = u32::try_from(i).unwrap();

                if let Some(schema) = variant_attrs
                    .references
                    .as_ref()
                    .map::<::syn::Expr, _>(|ref_attr| {
                        let ref_name = ref_attr.name.to_string();
                        let kind = match ref_attr.kind {
                            RefAttrKind::Direct => quote! { ::cecetype::schema::RefKind::Direct },
                            RefAttrKind::List => quote! { ::cecetype::schema::RefKind::Slice },
                        };

                        ::syn::parse_quote! { &::cecetype::schema::Schema::Ref { name: #ref_name, kind: #kind } }
                    })
                    .or_else(|| variant_attrs.repr_via.as_ref().map(|repr_ty| ::syn::parse_quote! { <#repr_ty as ::cecetype::Schema>::SCHEMA })
                    )
                {
                    return Ok(quote! {
                        &::cecetype::schema::VariantSchema {
                            discriminant: #discriminant,
                            name: #vname,
                            data: ::cecetype::schema::Data::NewType {
                                field: #schema,
                            },
                        }
                    });
                }

                let field_defs = vfields
                    .iter()
                    .map(|field| Ok((field, FieldAttrs::parse(&field.attrs)?)))
                    .collect::<::syn::Result<Vec<_>>>()?
                    .into_iter()
                    .filter(|(_, attrs)| !attrs.skip)
                    .map(field_schema)
                    .collect::<Vec<::syn::Expr>>();

                let schema = match vfields {
                    Fields::Unit => {
                        quote! {
                            &::cecetype::schema::VariantSchema {
                                discriminant: #discriminant,
                                name: #vname,
                                data: ::cecetype::schema::Data::Unit,
                            }
                        }
                    }

                    Fields::Unnamed(_) if field_defs.len() == 1 => {
                        let fschema = field_defs.first().unwrap();

                        quote! {
                            &::cecetype::schema::VariantSchema {
                                discriminant: #discriminant,
                                name: #vname,
                                data: ::cecetype::schema::Data::NewType {
                                    field: #fschema,
                                },
                            }
                        }
                    }

                    Fields::Unnamed(_) => {
                        quote! {
                            &::cecetype::schema::VariantSchema {
                                discriminant: #discriminant,
                                name: #vname,
                                data: ::cecetype::schema::Data::Tuple {
                                    fields: &[
                                        #( #field_defs ),*
                                    ],
                                },
                            }
                        }
                    }

                    Fields::Named(_) => {
                        quote! {
                            &::cecetype::schema::VariantSchema {
                                discriminant: #discriminant,
                                name: #vname,
                                data: ::cecetype::schema::Data::Struct {
                                    fields: &[
                                        #( #field_defs ),*
                                    ],
                                },
                            }
                        }
                    }
                };
                Ok(schema)
            },
        )
        .collect::<::syn::Result<Vec<_>>>()?;

    Ok(::syn::parse_quote! {
        ::cecetype::schema::Schema::Enum {
            name: #enum_name,
            variants: &[
                #( #variants ),*
            ],
        }
    })
}

fn field_schema(
    (::syn::Field { ident, ty, .. }, field_attrs): (&::syn::Field, FieldAttrs),
) -> ::syn::Expr {
    let ty_schema: ::syn::Expr = field_attrs.references.map_or_else(
        || {
            let repr_ty = field_attrs.repr_via.as_ref().unwrap_or(ty);
            ::syn::parse_quote! { <#repr_ty as ::cecetype::Schema>::SCHEMA }
        },
        |ref_attr| {
            let ref_name = ref_attr.name.to_string();
            let kind = match ref_attr.kind {
                RefAttrKind::Direct => quote! { ::cecetype::schema::RefKind::Direct },
                RefAttrKind::List => quote! { ::cecetype::schema::RefKind::Slice },
            };
            ::syn::parse_quote! { &::cecetype::schema::Schema::Ref { name: #ref_name, kind: #kind } }
        },
    );
    field_attrs
        .rename
        .as_ref()
        .map(::syn::LitStr::value)
        .or_else(|| ident.as_ref().map(ToString::to_string))
        .map_or_else(
            || ::syn::parse_quote! { #ty_schema },
            |name| {
                ::syn::parse_quote! {
                    &::cecetype::schema::FieldSchema {
                        name: #name,
                        ty: #ty_schema,
                    }
                }
            },
        )
}
