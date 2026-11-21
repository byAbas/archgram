//! The legend (DESIGN.md, Components: Legend): its entries, from what the
//! diagram uses, run left to right under the diagram, `spacing.section`
//! below it, and wrap into rows no wider than the diagram.

use crate::font::text_width;
use crate::geometry::{Rect, Size};
use crate::spec::{Kind, Spec, Still, Variant};
use crate::tokens::{
    LEGEND_SWATCH, LEGEND_SWATCH_GAP, SPACING_LEGEND, SPACING_LEGEND_ENTRY, SPACING_LEGEND_ROW,
    SPACING_SECTION, TYPOGRAPHY_LEGEND,
};

/// What an entry's swatch shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Swatch {
    /// A kind of node: its icon, as its card shows it.
    Kind(Kind),
    /// A boundary: a small dashed frame.
    Boundary,
    /// An edge taken only sometimes: a short dashed line.
    Sometimes,
    /// Several instances: a small card with two copies behind it.
    Multi,
    /// A system we do not own: a small dashed card.
    External,
    /// A sequence's call that waits: a short line in its colour.
    Call,
    /// A sequence's send that does not wait.
    Send,
    /// A sequence's reply: a short dashed line.
    Reply,
    /// A sequence's refused message: a short dashed line in the refusal
    /// colour.
    Refused,
}

/// One entry: its swatch and its text, each in its box.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub swatch: Swatch,
    pub swatch_box: Rect,
    pub text: &'static str,
    pub text_box: Rect,
}

/// A flow in words under the legend: its name, then its steps.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLine {
    pub text: String,
    pub text_box: Rect,
}

/// The flows in words, when the still image lists them.
#[must_use]
pub fn flow_texts(spec: &Spec) -> Vec<String> {
    if spec.still != Still::Legend {
        return Vec::new();
    }
    spec.flows
        .iter()
        .map(|f| format!("{}: {}", f.name.trim(), spec.flow_words(f)))
        .collect()
}

/// The entries a spec's legend has, in order; none when it draws no legend.
/// Every icon is in the text colour, so the icon's shape tells the kind:
/// the legend names each kind the spec uses, then what a card's or a
/// line's look means, as C4 asks of a legend (DESIGN.md, Components:
/// Legend).
#[must_use]
pub fn entries(spec: &Spec) -> Vec<(Swatch, &'static str)> {
    let has = |v: Variant| spec.nodes.iter().any(|n| n.variant == v);
    if !spec.legend {
        return Vec::new();
    }
    // Each kind the diagram uses, in the order the spec first uses it, so
    // the reader learns every icon the drawing shows.
    let mut out: Vec<(Swatch, &'static str)> = Vec::new();
    for n in &spec.nodes {
        if !out.iter().any(|(s, _)| *s == Swatch::Kind(n.kind)) {
            out.push((Swatch::Kind(n.kind), n.kind.legend_name()));
        }
    }
    if has(Variant::Multi) {
        out.push((Swatch::Multi, "Several instances"));
    }
    if has(Variant::External) {
        out.push((Swatch::External, "External"));
    }
    // Every other look the drawing uses, as C4 asks a key to explain
    // border styles and line types (c4model.com/diagrams/notation).
    if !spec.frames.is_empty() {
        out.push((Swatch::Boundary, "Boundary"));
    }
    if spec
        .edges
        .iter()
        .any(|e| e.style == crate::spec::EdgeStyle::Dashed)
    {
        out.push((Swatch::Sometimes, "Taken only sometimes"));
    }
    out
}

/// How much a small card's copies step up and to the right, a sixth of the
/// swatch, as `card.multi-offset` is to a card.
#[must_use]
pub fn stack_step() -> f64 {
    LEGEND_SWATCH / 6.0
}

/// A swatch's size: a card half as wide again as it is tall, with its
/// copies' reach for several instances.
fn swatch_size(s: Swatch) -> Size {
    match s {
        Swatch::Kind(_) => Size {
            w: LEGEND_SWATCH * 4.0 / 3.0,
            h: LEGEND_SWATCH * 4.0 / 3.0,
        },
        Swatch::External
        | Swatch::Boundary
        | Swatch::Sometimes
        | Swatch::Call
        | Swatch::Send
        | Swatch::Reply
        | Swatch::Refused => Size {
            w: 1.5 * LEGEND_SWATCH,
            h: LEGEND_SWATCH,
        },
        Swatch::Multi => Size {
            w: 1.5 * LEGEND_SWATCH + 2.0 * stack_step(),
            h: LEGEND_SWATCH + 2.0 * stack_step(),
        },
    }
}

/// The credit's words, either side of archgram's mark (DESIGN.md,
/// Components: Credit), and all its characters, for the font subset.
pub const CREDIT_BY: &str = "by";
pub const CREDIT_NAME: &str = "archgram";
pub const CREDIT: &str = "by archgram";

/// The credit's measures in `typography.legend`: the width of "by", of a
/// space, of the mark (as tall as the type is large) and of "archgram".
#[must_use]
pub fn credit_parts() -> [f64; 4] {
    [
        text_width(CREDIT_BY, &TYPOGRAPHY_LEGEND),
        text_width(" ", &TYPOGRAPHY_LEGEND),
        TYPOGRAPHY_LEGEND.size,
        text_width(CREDIT_NAME, &TYPOGRAPHY_LEGEND),
    ]
}

/// The credit's box, "by", the mark and "archgram" a space apart,
/// `spacing.legend` below all else and against the drawing's right edge,
/// and the drawing's size with it.
#[must_use]
pub fn credit(size: Size) -> (Rect, Size) {
    let [by, space, mark, name] = credit_parts();
    let w = by + space + mark + space + name;
    let h = TYPOGRAPHY_LEGEND.size * TYPOGRAPHY_LEGEND.line_height;
    let right = size.w.max(w);
    let at = Rect {
        x: right - w,
        y: size.h + SPACING_LEGEND,
        w,
        h,
    };
    (
        at,
        Size {
            w: right,
            h: at.y + at.h,
        },
    )
}

/// Lays the legend out under a drawing of `size`, left-aligned, in rows no
/// wider than it (or than the widest entry), then the flows in words, one a
/// line. Returns the entries, the flow lines and the drawing's new size.
#[must_use]
pub fn place(spec: &Spec, size: Size) -> (Vec<Entry>, Vec<FlowLine>, Size) {
    let (entries, size, placed) = place_entries(&entries(spec), size);
    let texts = flow_texts(spec);
    if texts.is_empty() {
        return (entries, Vec::new(), size);
    }
    let line = TYPOGRAPHY_LEGEND.size * TYPOGRAPHY_LEGEND.line_height;
    let mut y = size.h
        + if placed {
            SPACING_LEGEND_ROW
        } else {
            SPACING_SECTION
        };
    let mut lines = Vec::with_capacity(texts.len());
    for text in texts {
        let w = text_width(&text, &TYPOGRAPHY_LEGEND);
        lines.push(FlowLine {
            text,
            text_box: Rect {
                x: 0.0,
                y,
                w,
                h: line,
            },
        });
        y += line + SPACING_LEGEND_ROW;
    }
    let right = lines
        .iter()
        .map(|l| l.text_box.right())
        .fold(size.w, f64::max);
    let bottom = y - SPACING_LEGEND_ROW;
    (
        entries,
        lines,
        Size {
            w: right,
            h: bottom,
        },
    )
}

/// A sequence's legend: the variants its heads use, then the key to the
/// kinds of message it draws (DESIGN.md, Components: Sequence, Key); none
/// when the spec draws no legend. Returns the entries and the drawing's
/// new size.
#[must_use]
pub fn place_sequence(spec: &Spec, kinds: &[Swatch], size: Size) -> (Vec<Entry>, Size) {
    let mut list = entries(spec);
    if spec.legend {
        list.extend(kinds.iter().map(|&k| (k, key_text(k))));
    }
    let (entries, size, _) = place_entries(&list, size);
    (entries, size)
}

/// A kind of message's name in the key.
fn key_text(kind: Swatch) -> &'static str {
    match kind {
        Swatch::Call => "call, waits",
        Swatch::Send => "send, no wait",
        Swatch::Reply => "reply",
        Swatch::Refused => "refused",
        Swatch::Multi => "Several instances",
        Swatch::External => "External",
        Swatch::Kind(k) => k.legend_name(),
        Swatch::Boundary => "Boundary",
        Swatch::Sometimes => "Taken only sometimes",
    }
}

/// The entries alone, and whether there were any.
fn place_entries(list: &[(Swatch, &'static str)], size: Size) -> (Vec<Entry>, Size, bool) {
    if list.is_empty() {
        return (Vec::new(), size, false);
    }
    let line = TYPOGRAPHY_LEGEND.size * TYPOGRAPHY_LEGEND.line_height;
    let widths: Vec<f64> = list
        .iter()
        .map(|&(s, text)| {
            swatch_size(s).w + LEGEND_SWATCH_GAP + text_width(text, &TYPOGRAPHY_LEGEND)
        })
        .collect();
    let limit = widths.iter().copied().fold(size.w, f64::max);
    let row_height = list
        .iter()
        .map(|&(s, _)| swatch_size(s).h)
        .fold(line, f64::max);
    let (mut x, mut y) = (0.0, size.h + SPACING_SECTION);
    let mut out = Vec::with_capacity(list.len());
    let mut previous: Option<Swatch> = None;
    for (&(swatch, text), &w) in list.iter().zip(&widths) {
        // The kinds' icons are one group; what a card's or a line's look
        // means (external, several instances, a kind of message) starts a
        // row of its own, so it never reads as one more kind.
        let new_group = previous.is_some_and(|p| matches!(p, Swatch::Kind(_)))
            && !matches!(swatch, Swatch::Kind(_));
        if x > 0.0 && (x + w > limit || new_group) {
            x = 0.0;
            y += row_height + SPACING_LEGEND_ROW;
        }
        previous = Some(swatch);
        let s = swatch_size(swatch);
        let middle = y + row_height / 2.0;
        let swatch_box = Rect {
            x,
            y: (middle - s.h / 2.0).round(),
            w: s.w,
            h: s.h,
        };
        let text_box = Rect {
            x: x + s.w + LEGEND_SWATCH_GAP,
            y: middle - line / 2.0,
            w: w - s.w - LEGEND_SWATCH_GAP,
            h: line,
        };
        out.push(Entry {
            swatch,
            swatch_box,
            text,
            text_box,
        });
        x += w + SPACING_LEGEND_ENTRY;
    }
    let right = out
        .iter()
        .map(|e| e.text_box.right())
        .fold(size.w, f64::max);
    let bottom = y + row_height;
    (
        out,
        Size {
            w: right,
            h: bottom,
        },
        true,
    )
}
