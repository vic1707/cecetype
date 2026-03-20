#[proc_macro_derive(Schema, attributes(serde))]
pub fn derive_schema(input: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
    expand(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

fn expand(input: ::proc_macro::TokenStream) -> ::syn::Result<::proc_macro2::TokenStream> {
    let input: ::syn::DeriveInput = ::syn::parse(input)?;

    match input.data {
        ::syn::Data::Struct(ref data) => todo!(),
        ::syn::Data::Enum(ref data) => todo!(),
        _ => Err(syn::Error::new_spanned(
            input.ident,
            "Schema derive only supports struct and enum",
        )),
    }
}
