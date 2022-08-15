pub(crate) struct Colors;

macro_rules! colors {
    ($(($name: ident, $r: literal, $g: literal, $b: literal) $(,)?)*) => {
        #[allow(dead_code)]
        impl Colors {
            $(
            #[inline]
            pub(crate) fn $name() -> super::Color {
                super::Color::new($r, $g, $b)
            }
            )*
        }
    };
}

colors!(
    (black, 0x00, 0x00, 0x00),
    (white, 0xFF, 0xFF, 0xFF),
    (red, 0xFF, 0x00, 0x00),
    (green, 0x00, 0xFF, 0x00),
    (blue, 0x00, 0x00, 0xFF),
);
