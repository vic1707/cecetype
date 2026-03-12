#[derive(Default)]
pub struct ContainerSettings {
    pub array: bool,
    pub map: bool,
    pub index_only: bool,
    pub transparent: bool,
}

pub fn parse_container_settings(attrs: &[syn::Attribute]) -> syn::Result<ContainerSettings> {
    let mut out = ContainerSettings::default();

    for attr in attrs {
        if !attr.path().is_ident("cbor") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("array") {
                out.array = true;
            }

            if meta.path.is_ident("map") {
                out.map = true;
            }

            if meta.path.is_ident("index_only") {
                out.index_only = true;
            }

            if meta.path.is_ident("transparent") {
                out.transparent = true;
            }

            Ok(())
        })?;
    }

    Ok(out)
}
