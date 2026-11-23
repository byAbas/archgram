//! The still preview: a drawing as PNG, rendered inside archgram from the
//! SVG it just wrote, so a reader's view can be checked without a browser
//! (docs/features/reader-check.md).
//!
//! resvg renders the SVG. Two things are done first, since its parser
//! leaves them out: each CSS custom property (`var(--card)`) is replaced by
//! the value the theme gives it on `:root`, and the embedded font's family
//! is named as the font files name it. Only archgram's own fonts are
//! loaded, never the system's, and no image or other file is fetched: the
//! SVG is archgram's own and refers to none.

use std::sync::Arc;

use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg::{self, ImageHrefResolver};

/// The most pixels a preview draws: about 128 MB at four bytes each. A
/// drawing of the most nodes a spec may have, in a row, is about 12.5
/// million (47,196 × 264 px); one larger is too large to read, and only
/// its labels' length, which nothing else bounds, could make it so.
const MAX_PIXELS: u64 = 32_000_000;

/// The PNG of `svg`, an SVG archgram drew in one theme (`--theme light` or
/// `dark`), at its own size, when it is no larger than [`MAX_PIXELS`].
pub fn png(svg: &str) -> Result<Vec<u8>, String> {
    let svg = with_fonts_named(&with_values(svg)?);

    let mut fonts = usvg::fontdb::Database::new();
    for data in [
        archgram_core::font::REGULAR,
        archgram_core::font::MEDIUM,
        archgram_core::font::SEMIBOLD,
        archgram_core::font::MONO_REGULAR,
        archgram_core::font::MONO_MEDIUM,
    ] {
        fonts.load_font_data(data.to_vec());
    }
    let options = usvg::Options {
        font_family: "Geist".into(),
        fontdb: Arc::new(fonts),
        resources_dir: None,
        // archgram's SVG holds no image; none is read, from a file or a
        // data URL, should one ever appear.
        image_href_resolver: ImageHrefResolver {
            resolve_data: Box::new(|_, _, _| None),
            resolve_string: Box::new(|_, _| None),
        },
        ..usvg::Options::default()
    };
    let tree = usvg::Tree::from_str(&svg, &options).map_err(|e| e.to_string())?;
    // Rounded up, as a browser shows the last row a drawing reaches into:
    // a drawing 309.4 px tall is 310 rows, none of it cut off.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let (width, height) = (
        tree.size().width().ceil() as u32,
        tree.size().height().ceil() as u32,
    );
    if u64::from(width) * u64::from(height) > MAX_PIXELS {
        return Err(format!(
            "the drawing is {width}×{height} px, more than the {} million pixels a preview draws; shorten its labels, or split it into diagrams a reader can follow",
            MAX_PIXELS / 1_000_000
        ));
    }
    let mut pixmap = Pixmap::new(width, height)
        .ok_or_else(|| format!("cannot make a {width}×{height} image"))?;
    resvg::render(&tree, Transform::default(), &mut pixmap.as_mut());
    pixmap.encode_png().map_err(|e| e.to_string())
}

/// `svg` with each `var(--name)` replaced by the value its style gives
/// `--name`. A drawing in one theme gives each property one value, its
/// colours on `:root` and the rest on the rules that use them; a property
/// given two values, or none, is an error, not a guess or a black shape.
fn with_values(svg: &str) -> Result<String, String> {
    if !svg.contains(":root {") {
        return Err("the drawing has no `:root` rule to take its colours from".into());
    }
    let mut values: Vec<(&str, &str)> = Vec::new();
    for (at, _) in svg.match_indices("--") {
        let before = svg[..at].trim_end();
        if !(before.ends_with('{') || before.ends_with(';')) {
            continue; // a use, `var(--name)`, not a declaration
        }
        let declaration = &svg[at..];
        let Some(colon) = declaration.find(':') else {
            continue;
        };
        let name = declaration[..colon].trim();
        if !name.chars().all(|c| c == '-' || c.is_ascii_alphanumeric()) {
            continue;
        }
        let rest = &declaration[colon + 1..];
        let end = rest
            .find([';', '}'])
            .ok_or("a custom property is not closed")?;
        let value = rest[..end].trim();
        match values.iter().find(|(n, _)| *n == name) {
            Some((_, v)) if *v != value => {
                return Err(format!("the drawing gives {name} two values"));
            }
            Some(_) => {}
            None => values.push((name, value)),
        }
    }

    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some(at) = rest.find("var(") {
        out.push_str(&rest[..at]);
        let after = &rest[at + "var(".len()..];
        let end = after.find(')').ok_or("a `var(` is not closed")?;
        let name = after[..end].trim();
        let value = values
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, v)| *v)
            .ok_or_else(|| format!("the drawing uses {name}, which its `:root` does not give"))?;
        out.push_str(value);
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

/// `svg` with the family names its `@font-face` rules give the embedded
/// fonts replaced by the names the font files carry, which are the ones
/// loaded; resvg reads no `@font-face`.
fn with_fonts_named(svg: &str) -> String {
    svg.replace("\"archgram-geist-mono\"", "\"Geist Mono\"")
        .replace("\"archgram-geist\"", "\"Geist\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_var_takes_the_value_root_gives_it() {
        let svg = "<style>:root { --card: #ffffff; --edge: #8f8f8f; } .c { fill: var(--card); stroke: var( --edge ); }</style>";
        let out = with_values(svg).unwrap();
        assert!(out.contains("fill: #ffffff;"), "{out}");
        assert!(out.contains("stroke: #8f8f8f;"), "{out}");
        assert!(!out.contains("var("), "{out}");
    }

    #[test]
    fn a_property_given_on_another_rule_counts_and_two_values_do_not() {
        let svg = "<style>:root { --card: #fff; } .seq { --mono: \"archgram-geist-mono\", monospace; } .m { font-family: var(--mono); }</style>";
        assert!(
            with_values(svg)
                .unwrap()
                .contains("font-family: \"archgram-geist-mono\", monospace;")
        );
        let twice =
            "<style>:root { --card: #fff; } .x { --card: #000; } .c { fill: var(--card); }</style>";
        assert!(with_values(twice).unwrap_err().contains("two values"));
    }

    #[test]
    fn a_var_with_no_value_is_an_error() {
        let svg = "<style>:root { --card: #fff; } .c { fill: var(--missing); }</style>";
        assert!(with_values(svg).unwrap_err().contains("--missing"));
        assert!(with_values("<svg/>").is_err(), "no :root rule");
    }

    #[test]
    fn a_drawing_past_the_pixels_a_preview_draws_is_refused() {
        let side = 6000;
        assert!(side * side > MAX_PIXELS);
        let svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{side}" height="{side}"><style>:root {{ --card: #fff; }}</style></svg>"#
        );
        let Err(e) = png(&svg) else {
            panic!("the preview was drawn");
        };
        assert!(e.contains("million pixels"), "{e}");
    }

    #[test]
    fn the_embedded_families_take_the_files_names() {
        let svg = r#"text { font-family: "archgram-geist", sans-serif; } .m { font-family: "archgram-geist-mono"; }"#;
        let out = with_fonts_named(svg);
        assert!(out.contains(r#""Geist", sans-serif"#), "{out}");
        assert!(out.contains(r#""Geist Mono""#), "{out}");
    }
}
