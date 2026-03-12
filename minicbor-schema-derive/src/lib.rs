use ::quote::ToTokens as _;

#[proc_macro_derive(Schema, attributes(cbor, n, b))]
pub fn derive_schema(input: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
    expand(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

fn expand(input: ::proc_macro::TokenStream) -> ::syn::Result<::proc_macro2::TokenStream> {
    let input: ::syn::DeriveInput = ::syn::parse(input)?;

    match input.data {
        ::syn::Data::Struct(ref data) => {}
        ::syn::Data::Enum(ref data) => {}
        _ => panic!("Schema derive only supports struct and enum"),
    };

    Ok(input.to_token_stream())
}
