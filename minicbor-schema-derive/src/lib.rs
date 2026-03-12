mod expand;
mod parse;

#[proc_macro_derive(Schema, attributes(cbor, n, b))]
pub fn derive_schema(input: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
    expand::expand(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
