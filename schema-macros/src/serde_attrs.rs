pub struct ContainerAttrs;

pub struct VariantAttrs;

pub struct FieldAttrs;

impl ContainerAttrs {
    pub fn parse(attrs: &[syn::Attribute]) -> syn::Result<Self> {
        let out = Self {};

        for attr in attrs {
            if !attr.path().is_ident("serde") {
                continue;
            }

            attr.meta.require_list()?;
            attr.parse_nested_meta(|meta| {
                Err(syn::Error::new_spanned(
                    &meta.path,
                    "Schema: unsupported serde attribute",
                ))
            })?;
        }

        Ok(out)
    }
}

impl VariantAttrs {
    pub fn parse(attrs: &[syn::Attribute]) -> syn::Result<Self> {
        let out = Self {};

        for attr in attrs {
            if !attr.path().is_ident("serde") {
                continue;
            }

            attr.meta.require_list()?;
            attr.parse_nested_meta(|meta| {
                Err(syn::Error::new_spanned(
                    &meta.path,
                    "Schema: unsupported serde attribute",
                ))
            })?;
        }

        Ok(out)
    }
}

impl FieldAttrs {
    pub fn parse(attrs: &[syn::Attribute]) -> syn::Result<Self> {
        let out = Self {};

        for attr in attrs {
            if !attr.path().is_ident("serde") {
                continue;
            }

            attr.meta.require_list()?;
            attr.parse_nested_meta(|meta| {
                Err(syn::Error::new_spanned(
                    &meta.path,
                    "Schema: unsupported serde attribute",
                ))
            })?;
        }

        Ok(out)
    }
}