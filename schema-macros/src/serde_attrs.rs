// Won't support
// - `rename(...)` / `rename_all(...)` / `rename_all_fields(...)` variations
// - `default` as we can't (won't) send the default value
pub struct ContainerAttrs; // rename, rename_all, rename_all_fields, tag, content, untagged, transparent, from, try_from, into

// Won't support
// - `rename(...)` / `rename_all(...)` variations
// - `alias`
// - `with` / `serialize_with` / `deserialize_with` as we can't send the functions used for serialization
// - `other` as it can't really be represented
// Dunno
// - `skip_serializing` / `skip_deserializing`
pub struct VariantAttrs; // rename, rename_all, skip, untagged

// Won't support
// - `rename(...)` variations
// - `alias`
// - `with` / `serialize_with` / `deserialize_with` as we can't send the functions used for serialization
// - `default` as we can't (won't) send the default value
// Dunno
// - `flatten` is probably too hard
// - `skip_serializing` / `skip_deserializing` / `skip_serializing_if`
pub struct FieldAttrs; // rename, skip

impl ContainerAttrs {
    pub fn parse(attrs: &[::syn::Attribute]) -> ::syn::Result<Self> {
        let out = Self;

        for attr in attrs {
            if !attr.path().is_ident("serde") {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("bound")
                    || meta.path.is_ident("remote")
                    || meta.path.is_ident("crate")
                    || meta.path.is_ident("expecting")
                    || meta.path.is_ident("deny_unknown_fields")
                {
                    return Ok(());
                }

                Err(::syn::Error::new_spanned(
                    &meta.path,
                    "Schema: unsupported serde attribute",
                ))
            })?;
        }

        Ok(out)
    }
}

impl VariantAttrs {
    pub fn parse(attrs: &[::syn::Attribute]) -> ::syn::Result<Self> {
        let out = Self;

        for attr in attrs {
            if !attr.path().is_ident("serde") {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("bound") || meta.path.is_ident("borrow") {
                    return Ok(());
                }

                Err(::syn::Error::new_spanned(
                    &meta.path,
                    "Schema: unsupported serde attribute",
                ))
            })?;
        }

        Ok(out)
    }
}

impl FieldAttrs {
    pub fn parse(attrs: &[::syn::Attribute]) -> ::syn::Result<Self> {
        let out = Self;

        for attr in attrs {
            if !attr.path().is_ident("serde") {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("borrow")
                    || meta.path.is_ident("bound")
                    || meta.path.is_ident("getter")
                {
                    return Ok(());
                }

                Err(::syn::Error::new_spanned(
                    &meta.path,
                    "Schema: unsupported serde attribute",
                ))
            })?;
        }

        Ok(out)
    }
}
