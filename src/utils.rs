use ::core::{mem, ops::Deref};

#[inline]
#[expect(
    clippy::undocumented_unsafe_blocks,
    reason = "private use within serde ser/de"
)]
pub fn as_static_str(val: &(impl Deref<Target = str> + ?Sized)) -> &'static str {
    unsafe { mem::transmute::<&str, &'static str>(&**val) }
}
