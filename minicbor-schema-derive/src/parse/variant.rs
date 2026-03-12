#[derive(Default)]
pub struct VariantSettings {
    pub index: Option<u32>,
    pub skip: bool,
}

pub fn parse_variant_settings(attrs: &[syn::Attribute]) -> syn::Result<VariantSettings> {
    let mut out = VariantSettings::default();

    for attr in attrs {
        if attr.path().is_ident("n") {
            let lit: syn::LitInt = attr.parse_args()?;
            out.index = Some(lit.base10_parse()?);
            continue;
        }

        if attr.path().is_ident("cbor") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("skip") {
                    out.skip = true;
                }

                Ok(())
            })?;
        }
    }

    Ok(out)
}
