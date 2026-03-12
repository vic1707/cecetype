use quote::quote;
use syn::{Data, DeriveInput};

use crate::parse::container::parse_container_settings;

pub fn expand(input: proc_macro::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let input: DeriveInput = syn::parse(input)?;

    match input.data {
        Data::Struct(ref data) => expand_struct(&input, data),
        Data::Enum(ref data) => expand_enum(&input, data),
        _ => Err(syn::Error::new_spanned(
            input.ident,
            "Schema derive only supports struct and enum",
        )),
    }
}

fn expand_struct(
    input: &DeriveInput,
    data: &syn::DataStruct,
) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let name_str = name.to_string();

    let container = parse_container_settings(&input.attrs)?;

    let fields = match data.fields {
        syn::Fields::Named(ref f) => &f.named,
        _ => {
            return Err(syn::Error::new_spanned(
                name,
                "Schema derive requires named fields",
            ));
        }
    };

    let mut field_tokens = Vec::new();
    let mut index = 0usize;

    for field in fields {
        let settings = crate::parse::field::parse_field_settings(&field.attrs)?;

        if settings.skip {
            continue;
        }

        let ty = &field.ty;
        let ident = field.ident.as_ref().unwrap();
        let name_str = ident.to_string();

        let idx = settings.index.unwrap_or_else(|| {
            let i = index as u32;
            index += 1;
            i
        });

        if container.array {
            field_tokens.push(quote! {
                ::schema_core::ArrayFieldSchema {
                    name: #name_str,
                    position: #idx as usize,
                    ty: <#ty as ::schema_core::Schema>::SCHEMA,
                }
            });
        } else {
            field_tokens.push(quote! {
                ::schema_core::MapFieldSchema {
                    name: #name_str,
                    key: #idx,
                    ty: <#ty as ::schema_core::Schema>::SCHEMA,
                }
            });
        }
    }

    let field_ty = if container.array {
        quote!(::schema_core::ArrayFieldSchema)
    } else {
        quote!(::schema_core::MapFieldSchema)
    };

    let struct_ty = if container.array {
        quote!(::schema_core::ArrayStructSchema)
    } else {
        quote!(::schema_core::MapStructSchema)
    };

    let schema_variant = if container.array {
        quote!(ArrayStruct)
    } else {
        quote!(MapStruct)
    };

    Ok(quote! {

        const _: () = {

            static FIELDS: &[#field_ty] = &[
                #(#field_tokens),*
            ];

            static STRUCT_SCHEMA: #struct_ty =
                ::schema_core::StructSchema {
                    name: #name_str,
                    fields: FIELDS,
                };

            impl ::schema_core::Schema for #name {

                const SCHEMA: &'static ::schema_core::TypeSchema =
                    &::schema_core::TypeSchema::#schema_variant(
                        &STRUCT_SCHEMA
                    );

            }
        };

    })
}

fn expand_enum(input: &DeriveInput, data: &syn::DataEnum) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let name_str = name.to_string();

    let container = parse_container_settings(&input.attrs)?;

    let mut variant_tokens = Vec::new();
    let mut index = 0usize;

    for variant in &data.variants {
        let settings = crate::parse::variant::parse_variant_settings(&variant.attrs)?;

        if settings.skip {
            continue;
        }

        let vname = &variant.ident;
        let vname_str = vname.to_string();

        let idx = settings.index.unwrap_or_else(|| {
            let i = index as u32;
            index += 1;
            i
        });

        let payload = match variant.fields {
            syn::Fields::Unit => quote!(None),

            syn::Fields::Unnamed(ref f) if f.unnamed.len() == 1 => {
                let ty = &f.unnamed.first().unwrap().ty;

                quote!(
                    Some(<#ty as ::schema_core::Schema>::SCHEMA)
                )
            }

            _ => {
                return Err(syn::Error::new_spanned(
                    vname,
                    "Enum variants must be unit or single-field",
                ));
            }
        };

        if container.array {
            variant_tokens.push(quote! {
                ::schema_core::ArrayVariantSchema {
                    name: #vname_str,
                    position: #idx as usize,
                    payload: #payload,
                }
            });
        } else {
            variant_tokens.push(quote! {
                ::schema_core::MapVariantSchema {
                    name: #vname_str,
                    key: #idx,
                    payload: #payload,
                }
            });
        }
    }

    let variant_ty = if container.array {
        quote!(::schema_core::ArrayVariantSchema)
    } else {
        quote!(::schema_core::MapVariantSchema)
    };

    let enum_ty = if container.array {
        quote!(::schema_core::ArrayEnumSchema)
    } else {
        quote!(::schema_core::MapEnumSchema)
    };

    let schema_variant = if container.array {
        quote!(ArrayEnum)
    } else {
        quote!(MapEnum)
    };

    Ok(quote! {

        const _: () = {

            static VARIANTS: &[#variant_ty] = &[
                #(#variant_tokens),*
            ];

            static ENUM_SCHEMA: #enum_ty =
                ::schema_core::EnumSchema {
                    name: #name_str,
                    variants: VARIANTS,
                };

            impl ::schema_core::Schema for #name {

                const SCHEMA: &'static ::schema_core::TypeSchema =
                    &::schema_core::TypeSchema::#schema_variant(
                        &ENUM_SCHEMA
                    );

            }
        };

    })
}
