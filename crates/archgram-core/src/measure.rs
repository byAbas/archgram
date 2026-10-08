//! Box sizes (ARCHITECTURE.md, Measure). A card's height comes from its
//! style's template in the tokens; its width grows with its measured text,
//! never below the template's minimum (DESIGN.md, Layout).

use crate::font::text_width;
use crate::geometry::Size;
use crate::logos::{Logos, NoLogos};
use crate::spec::{CardStyle, LogoPlace, Node, Spec, Variant};
use crate::tokens::{
    CARD_HORIZONTAL_BADGE, CARD_HORIZONTAL_HEIGHT, CARD_HORIZONTAL_MIN_WIDTH, CARD_LOGO_CORNER,
    CARD_LOGO_INLINE, CARD_LOGO_INLINE_GAP, CARD_MULTI_OFFSET, CARD_PADDING, CARD_VERTICAL_HEIGHT,
    CARD_VERTICAL_MIN_WIDTH, LABEL_MAX_WIDTH, TYPOGRAPHY_FRAME_LABEL, TYPOGRAPHY_SUBTITLE,
    TYPOGRAPHY_TITLE,
};

/// What a card's second line shows.
#[derive(Debug, Clone, Copy)]
pub struct NoteLine<'a> {
    /// The logo leading it, when logos go inline.
    pub logo: Option<&'a str>,
    /// Its text: the note, or with an inline logo and no note, the
    /// technology's name (empty when the logos give none).
    pub text: Option<&'a str>,
}

/// A card's second line: its note, led by its technology's logo when logos
/// go inline; with no note, an inline logo brings the technology's name.
#[must_use]
pub fn note_line<'a>(node: &'a Node, place: LogoPlace, logos: &'a dyn Logos) -> NoteLine<'a> {
    let tech = node.tech.as_deref();
    let logo = if place == LogoPlace::Inline {
        tech.and_then(|t| logos.path(t))
    } else {
        None
    };
    let text = node
        .note
        .as_deref()
        .or_else(|| logo.map(|_| tech.and_then(|t| logos.title(t)).unwrap_or_default()));
    NoteLine { logo, text }
}

/// The width of a card's widest line of text, an inline logo included.
fn text_width_of(node: &Node, place: LogoPlace, logos: &dyn Logos) -> f64 {
    let title_width = text_width(&node.label, &TYPOGRAPHY_TITLE);
    let line = note_line(node, place, logos);
    let lead = if line.logo.is_some() {
        CARD_LOGO_INLINE + CARD_LOGO_INLINE_GAP
    } else {
        0.0
    };
    let note_width = line
        .text
        .map_or(0.0, |n| text_width(n, &TYPOGRAPHY_SUBTITLE))
        + lead;
    title_width.max(note_width)
}

/// Where a horizontal card's text starts, from the card's left edge: padding,
/// the badge, padding.
#[must_use]
pub fn horizontal_text_inset() -> f64 {
    CARD_PADDING + CARD_HORIZONTAL_BADGE + CARD_PADDING
}

/// The size of one node's footprint: its card, and for several instances
/// the two copies stepping up and to the right behind it, `card.multi-offset`
/// apart (DESIGN.md, Components: Variants).
#[must_use]
pub fn card_size(node: &Node, style: CardStyle, logo: LogoPlace, logos: &dyn Logos) -> Size {
    let card = front_card_size(node, style, logo, logos);
    if node.variant == Variant::Multi {
        Size {
            w: card.w + 2.0 * CARD_MULTI_OFFSET,
            h: card.h + 2.0 * CARD_MULTI_OFFSET,
        }
    } else {
        card
    }
}

/// The size of the card a reader sees first.
fn front_card_size(node: &Node, style: CardStyle, logo: LogoPlace, logos: &dyn Logos) -> Size {
    let text = text_width_of(node, logo, logos);
    // The text's right side gets twice the padding its left side has, so the
    // card does not look full; with a logo in the corner, room for the logo
    // between two paddings, so it never meets the title.
    let after = if node.tech.is_some() && logo == LogoPlace::Corner {
        CARD_PADDING + CARD_LOGO_CORNER + CARD_PADDING
    } else {
        2.0 * CARD_PADDING
    };
    match style {
        CardStyle::Horizontal => Size {
            w: CARD_HORIZONTAL_MIN_WIDTH
                .max(horizontal_text_inset() + text + after)
                .ceil(),
            h: CARD_HORIZONTAL_HEIGHT,
        },
        CardStyle::Vertical => Size {
            w: CARD_VERTICAL_MIN_WIDTH
                .max(text + 2.0 * CARD_PADDING)
                .ceil(),
            h: CARD_VERTICAL_HEIGHT,
        },
    }
}

/// An edge label's lines (DESIGN.md, Components: Connector): the label
/// itself while it fits `label.max-width` ([`LABEL_MAX_WIDTH`]), and
/// otherwise two lines split at the space that makes the longer of them
/// shortest. A label with no space stays one line.
#[must_use]
pub fn label_lines(label: &str) -> Vec<&str> {
    let width = |s: &str| text_width(s, &TYPOGRAPHY_SUBTITLE);
    if width(label) <= LABEL_MAX_WIDTH {
        return vec![label];
    }
    label
        .match_indices(' ')
        .map(|(i, _)| (label[..i].trim_end(), label[i + 1..].trim_start()))
        .filter(|(a, b)| !a.is_empty() && !b.is_empty())
        .min_by(|x, y| {
            width(x.0)
                .max(width(x.1))
                .total_cmp(&width(y.0).max(width(y.1)))
        })
        .map_or_else(|| vec![label], |(a, b)| vec![a, b])
}

/// The patch an edge's label sits on: its lines ([`label_lines`]), with
/// half a card's padding on either side (DESIGN.md, Components: Connector).
#[must_use]
pub fn label_size(label: &str) -> Size {
    let lines = label_lines(label);
    let widest = lines
        .iter()
        .map(|l| text_width(l, &TYPOGRAPHY_SUBTITLE))
        .fold(0.0, f64::max);
    #[allow(clippy::cast_precision_loss)] // one or two lines
    let count = lines.len() as f64;
    Size {
        w: widest + CARD_PADDING,
        h: count * TYPOGRAPHY_SUBTITLE.size * TYPOGRAPHY_SUBTITLE.line_height,
    }
}

/// Every piece of text the drawing shows, with the weight it is set in, in
/// drawing order: node titles and notes, edge labels, frame names in the
/// capitals they are drawn in, then the legend's entries. The font subset
/// and the check for characters the font lacks both read this list, so
/// neither can miss a text the other covers.
#[must_use]
pub fn text_runs(spec: &Spec) -> Vec<(u16, std::borrow::Cow<'_, str>)> {
    text_runs_with(spec, &NoLogos)
}

/// [`text_runs`], with the logos the drawing carries, whose names an inline
/// logo may show.
#[must_use]
pub fn text_runs_with<'a>(
    spec: &'a Spec,
    logos: &'a dyn Logos,
) -> Vec<(u16, std::borrow::Cow<'a, str>)> {
    let mut runs = Vec::new();
    for node in &spec.nodes {
        runs.push((TYPOGRAPHY_TITLE.weight, node.label.as_str().into()));
        if let Some(note) = note_line(node, spec.logo, logos).text {
            runs.push((TYPOGRAPHY_SUBTITLE.weight, note.into()));
        }
    }
    for edge in &spec.edges {
        if let Some(label) = &edge.label {
            runs.push((TYPOGRAPHY_SUBTITLE.weight, label.as_str().into()));
        }
    }
    for frame in &spec.frames {
        runs.push((
            TYPOGRAPHY_FRAME_LABEL.weight,
            frame.label.to_uppercase().into(),
        ));
    }
    for (_, text) in crate::layout::legend::entries(spec) {
        runs.push((crate::tokens::TYPOGRAPHY_LEGEND.weight, text.into()));
    }
    for text in crate::layout::legend::flow_texts(spec) {
        runs.push((crate::tokens::TYPOGRAPHY_LEGEND.weight, text.into()));
    }
    if spec.credit {
        runs.push((
            crate::tokens::TYPOGRAPHY_LEGEND.weight,
            crate::layout::legend::CREDIT.into(),
        ));
    }
    if spec.still == crate::spec::Still::Numbers {
        for numbers in crate::motion::step_numbers(spec)
            .iter()
            .filter(|n| !n.is_empty())
        {
            runs.push((
                crate::tokens::TYPOGRAPHY_LEGEND.weight,
                crate::render::step_label(numbers).into(),
            ));
        }
    }
    runs
}

/// The size of every node's card, in spec order.
#[must_use]
pub fn card_sizes(spec: &Spec) -> Vec<Size> {
    card_sizes_with(spec, &NoLogos)
}

/// [`card_sizes`], with the logos the drawing carries, which an inline logo
/// takes room for.
#[must_use]
pub fn card_sizes_with(spec: &Spec, logos: &dyn Logos) -> Vec<Size> {
    spec.nodes
        .iter()
        .map(|n| card_size(n, spec.card, spec.logo, logos))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::{Kind, Variant};

    fn node(label: &str, note: Option<&str>) -> Node {
        Node {
            id: "n".into(),
            kind: Kind::Service,
            label: label.into(),
            note: note.map(Into::into),
            tech: None,
            variant: Variant::Single,
            frame: None,
            source: None,
        }
    }

    #[test]
    fn a_label_wraps_onto_two_lines_only_past_the_token() {
        assert_eq!(label_lines("on a miss"), ["on a miss"]);
        // Split where the longer line is shortest, not at the first space.
        assert_eq!(
            label_lines("response + Cache-Control"),
            ["response +", "Cache-Control"]
        );
        // With no space there is nowhere to split.
        assert_eq!(
            label_lines("response+Cache-Control+ETag"),
            ["response+Cache-Control+ETag"]
        );
        for lines in [
            label_lines("fetches on first render"),
            label_lines("route exists in new app"),
        ] {
            assert_eq!(lines.len(), 2, "{lines:?}");
            assert!(
                lines
                    .iter()
                    .all(|l| text_width(l, &TYPOGRAPHY_SUBTITLE) <= LABEL_MAX_WIDTH),
                "{lines:?}"
            );
        }
    }

    #[test]
    #[allow(clippy::float_cmp)] // both sides are the same token, unchanged
    fn short_titles_keep_the_minimum_width() {
        assert_eq!(
            card_size(
                &node("API", None),
                CardStyle::Horizontal,
                LogoPlace::Corner,
                &NoLogos
            )
            .w,
            CARD_HORIZONTAL_MIN_WIDTH
        );
        assert_eq!(
            card_size(
                &node("API", None),
                CardStyle::Vertical,
                LogoPlace::Corner,
                &NoLogos
            )
            .w,
            CARD_VERTICAL_MIN_WIDTH
        );
    }

    #[test]
    fn long_titles_widen_the_card_to_fit() {
        let label = "count_candidates_by_seniority_and_city";
        let size = card_size(
            &node(label, None),
            CardStyle::Horizontal,
            LogoPlace::Corner,
            &NoLogos,
        );
        assert!(size.w > CARD_HORIZONTAL_MIN_WIDTH);
        assert!(
            size.w >= horizontal_text_inset() + text_width(label, &TYPOGRAPHY_TITLE) + CARD_PADDING
        );
    }

    #[test]
    fn a_long_note_widens_the_card_too() {
        let short = card_size(
            &node("API", Some("x")),
            CardStyle::Horizontal,
            LogoPlace::Corner,
            &NoLogos,
        )
        .w;
        let long = card_size(
            &node(
                "API",
                Some("three replicas behind the load balancer in eu-west-1"),
            ),
            CardStyle::Horizontal,
            LogoPlace::Corner,
            &NoLogos,
        )
        .w;
        assert!(long > short);
    }

    #[test]
    fn a_corner_logo_keeps_its_room_beside_the_title() {
        let long = "Billing service with a long name";
        let plain = card_size(
            &node(long, None),
            CardStyle::Horizontal,
            LogoPlace::Corner,
            &NoLogos,
        )
        .w;
        let mut with_logo = node(long, None);
        with_logo.tech = Some("postgresql".into());
        let corner = card_size(
            &with_logo,
            CardStyle::Horizontal,
            LogoPlace::Corner,
            &NoLogos,
        )
        .w;
        let chip = card_size(&with_logo, CardStyle::Horizontal, LogoPlace::Chip, &NoLogos).w;
        // Padding, logo, padding instead of two paddings: the logo's width more.
        assert!(
            (corner - plain - CARD_LOGO_CORNER).abs() <= 1.0,
            "{plain} {corner}"
        );
        assert!((chip - plain).abs() < 1e-9);
    }

    #[test]
    fn an_inline_logo_leads_the_note_or_brings_the_name() {
        use std::collections::BTreeMap;
        let logos: BTreeMap<String, String> =
            [("redis".to_owned(), "M0 0h24v24H0z".to_owned())].into();
        let mut n = node("Jobs", None);
        n.tech = Some("redis".into());
        // No note, and a set without names: the logo alone leads the line.
        let line = note_line(&n, LogoPlace::Inline, &logos);
        assert!(line.logo.is_some() && line.text == Some(""));
        // With a note, the logo leads it, and the card grows by the logo.
        n.note = Some("a long stream of every click".into());
        let chip = card_size(&n, CardStyle::Horizontal, LogoPlace::Chip, &logos).w;
        let inline = card_size(&n, CardStyle::Horizontal, LogoPlace::Inline, &logos).w;
        assert!(
            (inline - chip - (CARD_LOGO_INLINE + CARD_LOGO_INLINE_GAP)).abs() <= 1.0,
            "{chip} {inline}"
        );
        assert!(note_line(&n, LogoPlace::Chip, &logos).logo.is_none());
    }
}
