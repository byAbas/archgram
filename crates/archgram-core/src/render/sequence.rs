//! Drawing a laid-out sequence diagram (DESIGN.md, Components: Sequence):
//! the heads as cards, the lifelines, each message's line with UML's
//! arrowhead for its sort, its label and number, each fragment's frame, tag
//! and guards, and a refused message's ✕. A still drawing: the messages'
//! motion is drawn by a later change.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::color::Rgb;
use crate::font;
use crate::geometry::{Point, Rect};
use crate::layout::sequence::{Layout, Message, Sort};
use crate::sequence::{Operator, Sequence, SequenceStill, Step};
use crate::tokens::{
    ARROWHEAD_LENGTH, FONT_SANS, REFUSAL_MARK, REFUSAL_MARK_GAP, ROUNDED_CANVAS, ROUNDED_CARD,
    ROUNDED_FRAME, SPACING_FRAGMENT_TAG_PAD, SPACING_MARGIN, TYPOGRAPHY_FRAME_LABEL,
    TYPOGRAPHY_LEGEND, TYPOGRAPHY_SUBTITLE,
};

use super::scene::{Anchor, GroupOf, Item, Scene};
use super::svg::num;
use super::{EDGE_OFFSET, OFFSET, Options, Sheet, card, credit, edge, legend, styles};

/// The room inside a tag before its operator, and the cut of its corner.
const TAG_PAD: f64 = SPACING_FRAGMENT_TAG_PAD;

/// A laid-out sequence as a scene: every shape in drawing order, with its
/// style sheet. Long, as the architecture's: one drawing's parts in the
/// order they are drawn.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn scene(
    seq: &Sequence,
    layout: &Layout,
    options: Options,
    logos: &dyn crate::logos::Logos,
) -> Scene {
    let w = layout.size.w + 2.0 * SPACING_MARGIN;
    let h = layout.size.h + 2.0 * SPACING_MARGIN;
    let at = |r: Rect| Rect {
        x: r.x + OFFSET,
        y: r.y + OFFSET,
        ..r
    };
    let on_line = |p: Point| Point {
        x: p.x + EDGE_OFFSET,
        y: p.y + EDGE_OFFSET,
    };
    let refused = layout.messages.iter().any(|m| m.refused);
    let brands = super::brands(&layout.spec, logos);
    let style = style(seq, layout, options, logos, (&brands, refused));
    let mut items = vec![Item::Canvas {
        width: w,
        height: h,
        rx: ROUNDED_CANVAS,
    }];

    // The lifelines, under everything.
    let (top, bottom) = layout.lifeline_span;
    items.push(Item::Group {
        of: GroupOf::Class("lifelines"),
        items: layout
            .lifelines
            .iter()
            .map(|&x| Item::Path {
                id: None,
                class: "lifeline".into(),
                place: None,
                d: format!(
                    "M{} {}V{}",
                    num(x + OFFSET),
                    num(top + OFFSET),
                    num(bottom + OFFSET)
                ),
                arrowhead: false,
            })
            .collect(),
    });

    // The fragments, outermost first, their tags and guards over them.
    if !layout.fragments.is_empty() {
        let mut frames = Vec::new();
        for f in &layout.fragments {
            let r = at(f.frame);
            frames.push(Item::Rect {
                class: "fragment".into(),
                x: r.x,
                y: r.y,
                w: r.w,
                h: r.h,
                rx: Some(ROUNDED_FRAME),
            });
            for &y in &f.separators {
                frames.push(Item::Path {
                    id: None,
                    class: "operand".into(),
                    place: None,
                    d: format!("M{} {}H{}", num(r.x), num(y + OFFSET), num(r.right())),
                    arrowhead: false,
                });
            }
            frames.extend(tag(f.operator, at(f.tag)));
            // On a patch of the canvas, as a label is, so a lifeline does
            // not cross its words.
            for (lines, b) in &f.guards {
                let lines: Vec<&str> = lines.iter().map(String::as_str).collect();
                let (patch, words) = edge::label_lines_at(&lines, at(*b), ("label-patch", "sub"));
                frames.push(patch);
                frames.extend(words);
            }
        }
        items.push(Item::Group {
            of: GroupOf::Class("fragments"),
            items: frames,
        });
    }

    // The messages: each line, its arrowhead, its label.
    let mut lines = Vec::new();
    let mut labels = Vec::new();
    let mut marks = Vec::new();
    for m in &layout.messages {
        let points: Vec<Point> = m.path.iter().map(|&p| on_line(p)).collect();
        let drawn = edge::drawn(&points);
        lines.push(Item::Path {
            id: None,
            class: if m.sort == Sort::Reply {
                "edge reply".into()
            } else {
                "edge".into()
            },
            place: None,
            d: drawn.d(),
            arrowhead: false,
        });
        lines.push(arrowhead(m, &drawn));
        if let Some((text, r)) = &m.label {
            // On a patch of the canvas, so a lifeline passing behind a
            // label does not cross its words.
            let lines: Vec<&str> = text.iter().map(String::as_str).collect();
            let (patch, words) = edge::label_lines_at(&lines, at(*r), ("label-patch", "sub"));
            labels.push(patch);
            labels.extend(words);
        }
        if m.refused {
            marks.extend(cross(&drawn));
        }
    }
    lines.extend(labels);
    items.push(Item::Group {
        of: GroupOf::Class("edges"),
        items: lines,
    });
    if !marks.is_empty() {
        items.push(Item::Group {
            of: GroupOf::Class("refusals"),
            items: marks,
        });
    }

    // The heads, each a card over its lifeline; a refusing one edged in the
    // refusal colour.
    for (node, r) in layout.spec.nodes.iter().zip(&layout.heads) {
        let brand = card::Brand {
            class: node
                .tech
                .as_deref()
                .filter(|t| brands.contains_key(*t))
                .map(super::brand_class),
        };
        items.push(card::card(
            node,
            at(*r),
            (layout.spec.card, layout.spec.logo),
            logos,
            Some(&brand),
        ));
    }
    let mut refusers: Vec<usize> = layout
        .messages
        .iter()
        .filter(|m| m.refused)
        .map(|m| m.refuser)
        .collect();
    refusers.sort_unstable();
    refusers.dedup();
    for p in refusers {
        let c = card::front(&layout.spec.nodes[p], at(layout.heads[p]));
        items.push(Item::Rect {
            class: "refused-head".into(),
            x: c.x,
            y: c.y,
            w: c.w,
            h: c.h,
            rx: Some(ROUNDED_CARD.min(c.w / 2.0).min(c.h / 2.0)),
        });
    }

    // The numbers, over the lines they sit on.
    let pills: Vec<Item> = layout
        .messages
        .iter()
        .filter_map(|m| m.pill.map(|p| pill(m.number, at(p))))
        .flatten()
        .collect();
    if !pills.is_empty() {
        items.push(Item::Group {
            of: GroupOf::Class("steps"),
            items: pills,
        });
    }

    let entries: Vec<crate::layout::legend::Entry> = layout
        .legend
        .iter()
        .map(|e| crate::layout::legend::Entry {
            swatch_box: at(e.swatch_box),
            text_box: at(e.text_box),
            ..e.clone()
        })
        .collect();
    items.extend(legend::legend(&entries, &[]));
    if let Some(c) = layout.credit {
        items.push(credit::credit(at(c)));
    }
    Scene {
        width: w,
        height: h,
        title: seq.title.clone(),
        description: description(seq),
        style,
        items,
    }
}

/// A message's arrowhead, drawn on its own (DESIGN.md, Components:
/// Sequence): a call's filled, a send's and a reply's open; a refused
/// message's in the refusal colour.
fn arrowhead(m: &Message, drawn: &edge::Drawn) -> Item {
    let chevron = drawn.chevron();
    let (class, d) = match (m.refused, m.sort) {
        (true, _) => ("refused-mark", chevron),
        (false, Sort::Call) => ("arrowhead filled", format!("{chevron}Z")),
        (false, Sort::Send | Sort::Reply) => ("arrowhead", chevron),
    };
    Item::Path {
        id: None,
        class: class.into(),
        place: None,
        d,
        arrowhead: false,
    }
}

/// The ✕ on a refused message's line, `refusal.mark-gap` before its
/// arrowhead, over a patch of the canvas so the line does not cross it.
fn cross(drawn: &edge::Drawn) -> [Item; 2] {
    let half = REFUSAL_MARK / 2.0;
    let c = drawn.behind_tip(ARROWHEAD_LENGTH + REFUSAL_MARK_GAP + half);
    let d = format!(
        "M{} {}L{} {}M{} {}L{} {}",
        num(c.x - half),
        num(c.y - half),
        num(c.x + half),
        num(c.y + half),
        num(c.x + half),
        num(c.y - half),
        num(c.x - half),
        num(c.y + half)
    );
    let path = |class: &str| Item::Path {
        id: None,
        class: class.into(),
        place: None,
        d: d.clone(),
        arrowhead: false,
    };
    [path("refused-patch"), path("refused-mark")]
}

/// A fragment's tag: a pentagon at the frame's top left, its corner
/// following the frame's and its lower right cut (UML 2.5.1, 17.6.4.3),
/// the operator in it.
fn tag(operator: Operator, r: Rect) -> [Item; 2] {
    // The frame's own corner, as far as the tag's height and width allow.
    let rad = ROUNDED_FRAME.min(r.h).min(r.w - TAG_PAD);
    let d = format!(
        "M{} {}A{r} {r} 0 0 1 {} {}H{}V{}L{} {}H{}Z",
        num(r.x),
        num(r.y + rad),
        num(r.x + rad),
        num(r.y),
        num(r.right()),
        num(r.bottom() - TAG_PAD),
        num(r.right() - TAG_PAD),
        num(r.bottom()),
        num(r.x),
        r = num(rad)
    );
    [
        Item::Path {
            id: None,
            class: "fragment-tag".into(),
            place: None,
            d,
            arrowhead: false,
        },
        Item::Text {
            class: "fragment-op".into(),
            x: r.x + TAG_PAD,
            y: r.y
                + (r.h - TYPOGRAPHY_FRAME_LABEL.size * TYPOGRAPHY_FRAME_LABEL.line_height) / 2.0
                + font::baseline_in_line(&TYPOGRAPHY_FRAME_LABEL),
            anchor: Anchor::Start,
            text: operator.name().into(),
        },
    ]
}

/// A message's number in its pill (DESIGN.md, Components: Signal, a step's
/// number), its outline on half pixels.
fn pill(number: u32, b: Rect) -> [Item; 2] {
    let line = TYPOGRAPHY_LEGEND.size * TYPOGRAPHY_LEGEND.line_height;
    [
        Item::Rect {
            class: "step".into(),
            x: b.x,
            y: b.y,
            w: b.w - 1.0,
            h: b.h - 1.0,
            rx: Some((b.h - 1.0) / 2.0),
        },
        Item::Text {
            class: "step-text".into(),
            x: b.x + (b.w - 1.0) / 2.0,
            y: b.y + (b.h - 1.0) / 2.0 - line / 2.0 + font::baseline_in_line(&TYPOGRAPHY_LEGEND),
            anchor: Anchor::Middle,
            text: number.to_string(),
        },
    ]
}

/// The style sheet: the theme, the fonts with every text the drawing sets,
/// the shapes every drawing has, then a sequence's own.
fn style(
    seq: &Sequence,
    layout: &Layout,
    options: Options,
    logos: &dyn crate::logos::Logos,
    (brands, refused): (&BTreeMap<String, Rgb>, bool),
) -> Vec<super::scene::StyleLine> {
    let (light, dark) = super::colors(&seq.palette, options);
    let mut sheet = Sheet::default();
    // The refusal colour is a signal's role, written only where it is used.
    super::theme_vars(&mut sheet, options.mode, (&light, &dark), refused);
    let runs = text_runs(layout, logos);
    if options.embed_font && super::embed_runs(&mut sheet, runs.into_iter()) {
        sheet.line(&format!(
            "text {{ font-family: \"{}\", {FONT_SANS}; font-kerning: none; }}",
            font::FAMILY
        ));
    } else {
        sheet.line(&format!("text {{ font-family: {FONT_SANS}; }}"));
    }
    sheet.rules(styles::shapes());
    super::brand_rules(&mut sheet, brands, (&light, &dark, options.mode));
    sheet.rules(styles::sequence());
    if seq.still == SequenceStill::Numbers {
        sheet.rules(styles::steps());
    }
    if refused {
        sheet.rules(styles::sequence_refusal());
    }
    if seq.credit {
        sheet.rules(styles::credit());
    }
    if !layout.legend.is_empty() {
        sheet.rules(styles::legend());
    }
    sheet.0
}

/// Every text the drawing sets, with its weight: the heads', and each
/// label, number, operator and guard. For the font's subset, and for the
/// characters it lacks.
#[must_use]
pub fn text_runs(layout: &Layout, logos: &dyn crate::logos::Logos) -> Vec<(u16, String)> {
    let mut runs: Vec<(u16, String)> = crate::measure::text_runs_with(&layout.spec, logos)
        .into_iter()
        .map(|(w, t)| (w, t.into_owned()))
        .collect();
    for m in &layout.messages {
        if let Some((lines, _)) = &m.label {
            runs.extend(
                lines
                    .iter()
                    .map(|l| (TYPOGRAPHY_SUBTITLE.weight, l.clone())),
            );
        }
        if m.pill.is_some() {
            runs.push((TYPOGRAPHY_LEGEND.weight, m.number.to_string()));
        }
    }
    for f in &layout.fragments {
        runs.push((TYPOGRAPHY_FRAME_LABEL.weight, f.operator.name().into()));
        for (lines, _) in &f.guards {
            runs.extend(
                lines
                    .iter()
                    .map(|l| (TYPOGRAPHY_SUBTITLE.weight, l.clone())),
            );
        }
    }
    runs
}

/// The description, then each message in words, in order, for a screen
/// reader, which sees neither the lines nor their order down the page: a
/// message's number, who sends it to whom and its label, each fragment's
/// guard before its messages. Read from the same steps the layout draws, so
/// the words and the drawing never disagree.
fn description(seq: &Sequence) -> String {
    let label = |p: usize| seq.participants[p].label.trim().to_owned();
    let mut out = seq.description.trim().to_owned();
    let mut open: Vec<Operator> = Vec::new();
    let mut n = 0u32;
    for step in seq.steps() {
        match step {
            Step::Message(m) => {
                n += 1;
                let verb = match m.sort {
                    Sort::Call => "calls",
                    Sort::Send => "sends to",
                    Sort::Reply => "replies to",
                };
                let _ = write!(out, " {n}. {} {verb} {}", label(m.from), label(m.to));
                if let Some(l) = m.label {
                    let _ = write!(out, ": {}", l.trim());
                }
                if m.refused {
                    out.push_str(", refused");
                }
                out.push('.');
            }
            Step::Open(operator, when) => {
                open.push(operator);
                let _ = write!(out, " {}", lead(operator, when, true));
            }
            Step::Operand(when) => {
                let operator = *open.last().expect("an operand inside a fragment");
                let _ = write!(out, " {}", lead(operator, when, false));
            }
            Step::Close => {
                open.pop();
            }
        }
    }
    out
}

/// What a screen reader hears before an operand's messages.
fn lead(operator: Operator, when: Option<&str>, first: bool) -> String {
    match (operator, when.map(str::trim)) {
        (Operator::Alt, Some("else")) => "Otherwise:".to_owned(),
        (Operator::Alt, Some(w)) => format!("If {w}:"),
        (Operator::Opt, Some(w)) => format!("Only if {w}:"),
        (Operator::Opt, None) => "Optionally:".to_owned(),
        (Operator::Loop, Some(w)) => format!("Repeated, {w}:"),
        (Operator::Loop, None) => "Repeated:".to_owned(),
        (Operator::Par, _) if first => "In parallel:".to_owned(),
        (Operator::Par | Operator::Alt, _) => "And:".to_owned(),
    }
}
