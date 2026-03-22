use ::{quote::quote, syn::Fields};

#[proc_macro_derive(Schema, attributes(serde))]
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

    let schema = match data {
        ::syn::Data::Struct(ref data) => struct_schema(&ident, data),
        ::syn::Data::Enum(ref data) => enum_schema(&ident, data),
        _ => {
            return Err(::syn::Error::new_spanned(
                ident,
                "Schema derive only supports struct and enum",
            ));
        }
    };

    let generics_ident = generics
        .type_params()
        .map(|syn::TypeParam { ident, .. }| ident.clone())
        .collect::<Vec<_>>();
    let where_clause = generics.make_where_clause();
    for ty in generics_ident {
        where_clause
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
    let name = name.to_string();

    match &data.fields {
        Fields::Named(fields) => {
            let field_defs = fields.named.iter().map(|f| {
                let fname = f.ident.as_ref().unwrap().to_string();
                let ty = &f.ty;

                quote! {
                    &schema::FieldSchema {
                        name: #fname,
                        ty: <#ty as schema::Schema>::SCHEMA,
                    }
                }
            });

            quote! {
                schema::TypeSchema::Struct {
                    name: #name,
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
                    name: #name,
                    field: { <#ty as schema::Schema>::SCHEMA },
                }
            }
        }

        Fields::Unnamed(fields) => {
            let field_tys = fields.unnamed.iter().map(|::syn::Field { ty, .. }| ty);

            quote! {
                schema::TypeSchema::TupleStruct {
                    name: #name,
                    fields: &[
                        #( <#field_tys as schema::Schema>::SCHEMA ),*
                    ],
                }
            }
        }

        Fields::Unit => {
            quote! {
                schema::TypeSchema::UnitStruct {
                    name: #name,
                }
            }
        }
    }
}

fn enum_schema(name: &::proc_macro2::Ident, data: &::syn::DataEnum) -> ::proc_macro2::TokenStream {
    let name = name.to_string();

    let variants = data.variants.iter().enumerate().map(|(i, v)| {
        let vname = v.ident.to_string();
        let discriminant = i as u32;

        match &v.fields {
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
                let field_defs = fields.named.iter().map(|f| {
                    let fname = f.ident.as_ref().unwrap().to_string();
                    let ty = &f.ty;

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
            name: #name,
            variants: &[
                #( #variants ),*
            ],
        }
    }
}
