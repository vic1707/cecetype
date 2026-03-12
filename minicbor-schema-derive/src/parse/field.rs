#[derive(Default)]
pub struct FieldSettings {
    pub index: Option<u32>,
    pub skip: bool,
    pub default: bool,
}

pub fn parse_field_settings(attrs: &[syn::Attribute]) -> syn::Result<FieldSettings> {
    let mut out = FieldSettings::default();

    for attr in attrs {
        if attr.path().is_ident("n") || attr.path().is_ident("b") {
            let lit: syn::LitInt = attr.parse_args()?;
            out.index = Some(lit.base10_parse()?);
            continue;
        }

        if attr.path().is_ident("cbor") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("skip") {
                    out.skip = true;
                }

                if meta.path.is_ident("default") {
                    out.default = true;
                }

                Ok(())
            })?;
        }
    }

    Ok(out)
}
