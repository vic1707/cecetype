use ::core::mem;

#[inline]
#[expect(
    clippy::undocumented_unsafe_blocks,
    reason = "private use within serde ser/de"
)]
pub fn as_static_str(val: &(impl AsRef<str> + ?Sized)) -> &'static str {
    unsafe { mem::transmute::<&str, &'static str>(val.as_ref()) }
}
