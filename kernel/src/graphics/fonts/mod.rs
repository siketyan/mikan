#[rustfmt::skip]
pub(crate) mod shinonome;

pub(crate) use shinonome::Shinonome;

pub(crate) trait Font {
    fn glyph(c: char) -> Option<&'static [u8]>;
}
