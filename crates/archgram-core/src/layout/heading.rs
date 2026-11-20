//! The heading (DESIGN.md, Components: Heading): the drawing's visible
//! title, "<Kind> diagram of <title>", in a node's title type, and the
//! spec's subtitle under it in a node's subtitle type, above everything,
//! `spacing.legend` over the diagram as the legend is under it.

use crate::font::text_width;
use crate::geometry::Rect;
use crate::spec::DiagramKind;
use crate::tokens::{SPACING_LABEL_GAP, SPACING_LEGEND, TYPOGRAPHY_SUBTITLE, TYPOGRAPHY_TITLE};

/// The heading's texts and their boxes, from its own top left.
#[derive(Debug, Clone, PartialEq)]
pub struct Heading {
    pub title: String,
    pub title_box: Rect,
    pub subtitle: Option<(String, Rect)>,
    /// How far everything else is moved down to make room for it: the
    /// texts and `spacing.legend` under them, in whole pixels so a line
    /// keeps covering whole pixels.
    pub depth: f64,
    /// Its width, the wider of its lines, in whole pixels, so a drawing
    /// it widens keeps a size the SVG writes as it is.
    pub width: f64,
}

/// The heading of a diagram of `kind` named `title`, with its `subtitle`,
/// when `shown`.
#[must_use]
pub fn place(
    kind: DiagramKind,
    title: &str,
    subtitle: Option<&str>,
    shown: bool,
) -> Option<Heading> {
    if !shown {
        return None;
    }
    let title = kind.heading(title);
    let line = |style: &crate::tokens::TextStyle| style.size * style.line_height;
    let title_box = Rect {
        x: 0.0,
        y: 0.0,
        w: text_width(&title, &TYPOGRAPHY_TITLE),
        h: line(&TYPOGRAPHY_TITLE),
    };
    let subtitle = subtitle.map(|s| {
        let s = s.trim().to_owned();
        let at = Rect {
            x: 0.0,
            y: title_box.bottom() + SPACING_LABEL_GAP,
            w: text_width(&s, &TYPOGRAPHY_SUBTITLE),
            h: line(&TYPOGRAPHY_SUBTITLE),
        };
        (s, at)
    });
    let last = subtitle.as_ref().map_or(title_box, |(_, at)| *at);
    let width = subtitle
        .as_ref()
        .map_or(title_box.w, |(_, at)| at.w.max(title_box.w));
    Some(Heading {
        title,
        title_box,
        subtitle,
        depth: (last.bottom() + SPACING_LEGEND).ceil(),
        width: width.ceil(),
    })
}

/// The heading's texts with their weights, for the font subset.
#[must_use]
pub fn runs(heading: Option<&Heading>) -> Vec<(u16, String)> {
    let Some(h) = heading else {
        return Vec::new();
    };
    let mut runs = vec![(TYPOGRAPHY_TITLE.weight, h.title.clone())];
    if let Some((s, _)) = &h.subtitle {
        runs.push((TYPOGRAPHY_SUBTITLE.weight, s.clone()));
    }
    runs
}
