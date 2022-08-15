use std::error::Error;
use std::fs::File;
use std::io::Write;

use itertools::Itertools;

fn main() -> Result<(), Box<dyn Error>> {
    let font = bdf::read(File::open("../resources/fonts/shinonome/shnm8x16a.bdf")?)?;

    let glyphs = font
        .glyphs()
        .iter()
        .filter(|(char, _)| char.is_ascii() && !char.is_ascii_control())
        .sorted_by(|(a, _), (b, _)| Ord::cmp(a, b))
        .collect::<Vec<_>>();

    let mut file = File::create("./src/graphics/fonts/shinonome.rs")?;

    writeln!(file, "#![allow(dead_code)]")?;

    glyphs.iter().try_for_each(|(_, glyph)| {
        writeln!(
            file,
            "const GLYPH_{}: &[u8] = &[{}];",
            glyph.name().to_ascii_uppercase(),
            glyph
                .pixels()
                .chunks(8)
                .into_iter()
                .map(|chunk| {
                    chunk
                        .enumerate()
                        .map(|(i, (_, value))| (if value { 1 } else { 0 }) << (7 - i))
                        .fold(0u8, |mut acc, v| {
                            acc |= v;
                            acc
                        })
                })
                .map(|byte| format!("{:#010b}", byte))
                .join(",")
        )
    })?;

    writeln!(
        file,
        "pub(crate) fn glyph(c: char) -> Option<&'static [u8]> {{ Some(match c {{"
    )?;

    glyphs.iter().try_for_each(|(char, glyph)| {
        writeln!(
            file,
            "'{}' => GLYPH_{},",
            match char {
                '\'' => "\\'".to_owned(),
                '\\' => "\\\\".to_owned(),
                _ => char.to_string(),
            },
            glyph.name().to_ascii_uppercase()
        )
    })?;

    writeln!(file, "_ => return None,")?;
    writeln!(file, "}}) }}")?;

    Ok(())
}
