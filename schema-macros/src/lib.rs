#![expect(clippy::unwrap_used, reason = "wip")]
use ::{quote::quote, syn::Fields};

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
        ..
    } = ::syn::parse(input)?;

    let schema = match &data {
        ::syn::Data::Struct(data_struct) => struct_schema(&ident, data_struct),
        ::syn::Data::Enum(data_enum) => enum_schema(&ident, data_enum),
        ::syn::Data::Union(_) => {
            return Err(::syn::Error::new_spanned(
                ident,
                "Schema derive only supports struct and enum",
            ));
        }
    };

    let generics_ident = generics
        .type_params()
        .cloned()
        .map(|syn::TypeParam { ident: i, .. }| i)
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
    name: &::proc_macro2::Ident,
    data: &::syn::DataStruct,
) -> ::proc_macro2::TokenStream {
    let struct_name = name.to_string();

    match &data.fields {
        Fields::Named(fields) => {
            let field_defs = fields.named.iter().map(|::syn::Field { ident, ty, .. }| {
                let fname = ident.as_ref().unwrap().to_string();

                quote! {
                    &schema::FieldSchema {
                        name: #fname,
                        ty: <#ty as schema::Schema>::SCHEMA,
                    }
                }
            });

            quote! {
                schema::TypeSchema::Struct {
                    name: #struct_name,
                    fields: &[
                        #( #field_defs ),*
                    ],
                }
            }
        }

        Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
            let ::syn::Field { ty, .. } = fields.unnamed.first().unwrap();

            quote! {
                schema::TypeSchema::NewTypeStruct {
                    name: #struct_name,
                    field: { <#ty as schema::Schema>::SCHEMA },
                }
            }
        }

        Fields::Unnamed(fields) => {
            let field_tys = fields.unnamed.iter().map(|::syn::Field { ty, .. }| ty);

            quote! {
                schema::TypeSchema::TupleStruct {
                    name: #struct_name,
                    fields: &[
                        #( <#field_tys as schema::Schema>::SCHEMA ),*
                    ],
                }
            }
        }

        Fields::Unit => {
            quote! {
                schema::TypeSchema::UnitStruct {
                    name: #struct_name,
                }
            }
        }
    }
}

fn enum_schema(name: &::proc_macro2::Ident, data: &::syn::DataEnum) -> ::proc_macro2::TokenStream {
    let enum_name = name.to_string();

    let variants = data
        .variants
        .iter()
        .enumerate()
        .map(|(i, ::syn::Variant { ident: vident, fields: vfields, .. })| {
            let vname = vident.to_string();
            let discriminant = u32::try_from(i).unwrap();

            match vfields {
                Fields::Unit => {
                    quote! {
                        &schema::VariantSchema::Unit {
                            name: #vname,
                            discriminant: #discriminant,
                        }
                    }
                }

                Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                    let ::syn::Field { ty, .. } = fields.unnamed.first().unwrap();

                    quote! {
                        &schema::VariantSchema::NewType {
                            name: #vname,
                            discriminant: #discriminant,
                            field: { <#ty as schema::Schema>::SCHEMA },
                        }
                    }
                }

                Fields::Unnamed(fields) => {
                    let field_tys = fields.unnamed.iter().map(|::syn::Field { ty, .. }| ty);

                    quote! {
                        &schema::VariantSchema::Tuple {
                            name: #vname,
                            discriminant: #discriminant,
                            fields: &[
                                #( <#field_tys as schema::Schema>::SCHEMA ),*
                            ],
                        }
                    }
                }

                Fields::Named(fields) => {
                    let field_defs =
                        fields.named.iter().map(|::syn::Field { ident, ty, .. }| {
                            let fname = ident.as_ref().unwrap().to_string();

                            quote! {
                                &schema::FieldSchema {
                                    name: #fname,
                                    ty: <#ty as schema::Schema>::SCHEMA,
                                }
                            }
                        });

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
            }
        });

    quote! {
        schema::TypeSchema::Enum {
            name: #enum_name,
            variants: &[
                #( #variants ),*
            ],
        }
    }
}
