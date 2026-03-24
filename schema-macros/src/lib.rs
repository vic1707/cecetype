#![expect(clippy::unwrap_used, reason = "wip")]

mod serde_attrs;
use self::serde_attrs::{ContainerAttrs, FieldAttrs, VariantAttrs};
use ::{
    quote::quote,
    syn::{self, Fields},
};

#[proc_macro_derive(Schema, attributes(serde))]
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

    let schema = match &data {
        ::syn::Data::Struct(data_struct) => struct_schema(&name, data_struct, container_attrs)?,
        ::syn::Data::Enum(data_enum) => enum_schema(&name, data_enum, container_attrs)?,
        ::syn::Data::Union(_) => {
            return Err(::syn::Error::new_spanned(
                ident,
                "Schema derive only supports struct and enum",
            ));
        }
    };

    let generics_ident = generics
        .type_params()
        .map(|syn::TypeParam { ident: i, .. }| i)
        .cloned()
        .collect::<Vec<_>>();

    let where_clause_ref = generics.make_where_clause();
    for ty in generics_ident {
        where_clause_ref
            .predicates
            .push(::syn::parse_quote! { #ty: ::schema::Schema });
    }

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    Ok(quote! {
        impl #impl_generics ::schema::Schema for #ident #ty_generics #where_clause {
            const SCHEMA: &'static ::schema::StaticSchema = &#schema;
        }
    })
}

fn struct_schema(
    struct_name: &str,
    data: &::syn::DataStruct,
    _container_attrs: ContainerAttrs,
) -> ::syn::Result<::syn::ExprStruct> {
    let field_defs = data
        .fields
        .iter()
        .map(|field| Ok((field, FieldAttrs::parse(&field.attrs)?)))
        .collect::<::syn::Result<Vec<_>>>()?
        .into_iter()
        .filter(|(_, attrs)| !attrs.skip)
        .map(field_schema)
        .collect::<Vec<::syn::Expr>>();

    let schema = match &data.fields {
        Fields::Named(_) => {
            ::syn::parse_quote! {
                schema::TypeSchema::Struct {
                    name: #struct_name,
                    fields: &[
                        #( #field_defs ),*
                    ],
                }
            }
        }

        Fields::Unnamed(_) if field_defs.len() == 1 => {
            let field = field_defs.first().unwrap();
            ::syn::parse_quote! {
                schema::TypeSchema::NewTypeStruct {
                    name: #struct_name,
                    field: #field,
                }
            }
        }

        Fields::Unnamed(_) => {
            ::syn::parse_quote! {
                schema::TypeSchema::TupleStruct {
                    name: #struct_name,
                    fields: &[
                        #( #field_defs ),*
                    ],
                }
            }
        }

        Fields::Unit => {
            ::syn::parse_quote! {
                schema::TypeSchema::UnitStruct {
                    name: #struct_name,
                }
            }
        }
    };
    Ok(schema)
}

fn enum_schema(
    enum_name: &str,
    data: &::syn::DataEnum,
    _container_attrs: ContainerAttrs,
) -> ::syn::Result<::syn::ExprStruct> {
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
                            &schema::VariantSchema::Unit {
                                name: #vname,
                                discriminant: #discriminant,
                            }
                        }
                    }

                    Fields::Unnamed(_) if field_defs.len() == 1 => {
                        let fschema = field_defs.first().unwrap();

                        quote! {
                            &schema::VariantSchema::NewType {
                                name: #vname,
                                discriminant: #discriminant,
                                field: #fschema,
                            }
                        }
                    }

                    Fields::Unnamed(_) => {
                        quote! {
                            &schema::VariantSchema::Tuple {
                                name: #vname,
                                discriminant: #discriminant,
                                fields: &[
                                    #( #field_defs ),*
                                ],
                            }
                        }
                    }

                    Fields::Named(_) => {
                        quote! {
                            &schema::VariantSchema::Struct {
                                name: #vname,
                                discriminant: #discriminant,
                                fields: &[
                                    #( #field_defs ),*
                                ],
                            }
                        }
                    }
                };
                Ok(schema)
            },
        )
        .collect::<::syn::Result<Vec<_>>>()?;

    Ok(::syn::parse_quote! {
        schema::TypeSchema::Enum {
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
    let fname = field_attrs
        .rename
        .as_ref()
        .map(::syn::LitStr::value)
        .or_else(|| ident.as_ref().map(ToString::to_string));

    fname.map_or_else(
        || ::syn::parse_quote! { <#ty as schema::Schema>::SCHEMA },
        |name| {
            ::syn::parse_quote! {
                &schema::FieldSchema {
                    name: #name,
                    ty: <#ty as schema::Schema>::SCHEMA,
                }
            }
        },
    )
}
