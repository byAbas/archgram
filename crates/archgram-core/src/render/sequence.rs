//! Drawing a laid-out sequence diagram (DESIGN.md, Components: Sequence):
//! each participant's band and head, as its look draws it, each fragment's
//! frame and pills, each message's line with UML's arrowhead for its sort
//! and its label in a pill, the bars while a call is answered, and a
//! refused message's ✕. A still drawing: the messages' motion is drawn by
//! a later change.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::color::Rgb;
use crate::font;
use crate::geometry::{Point, Rect};
use crate::layout::sequence::{Layout, Message, Pill, Sort};
use crate::sequence::{Look, Operator, Sequence, Step};
use crate::tokens::{
    ARROWHEAD_LENGTH, AVATAR_SIZE, FONT_SANS, LABEL_PILL_PAD_X, LABEL_PILL_PAD_Y, REFUSAL_MARK,
    REFUSAL_MARK_GAP, ROUNDED_CANVAS, ROUNDED_CARD, ROUNDED_FRAME, SPACING_LABEL_GAP,
    SPACING_MARGIN, TYPOGRAPHY_FRAME_LABEL, TYPOGRAPHY_LEGEND, TYPOGRAPHY_SUBTITLE,
};

use super::scene::{Anchor, GroupOf, Item, Scene};
use super::svg::num;
use super::{EDGE_OFFSET, OFFSET, Options, Sheet, card, credit, edge, legend, styles};

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

    // The bands, the lifelines, under everything.
    items.push(Item::Group {
        of: GroupOf::Class("lifelines"),
        items: layout
            .bands
            .iter()
            .map(|&b| {
                let r = at(b);
                Item::Rect {
                    class: "band".into(),
                    x: r.x,
                    y: r.y,
                    w: r.w,
                    h: r.h,
                    rx: Some(ROUNDED_FRAME),
                }
            })
            .collect(),
    });

    // The fragments, outermost first, each with its pills on its lines.
    if !layout.fragments.is_empty() {
        let class = match layout.look {
            Look::Cards => "fragment",
            Look::Avatars => "fragment dashed",
        };
        let mut frames = Vec::new();
        for f in &layout.fragments {
            let r = at(f.frame);
            frames.push(Item::Rect {
                class: class.into(),
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
        }
        items.push(Item::Group {
            of: GroupOf::Class("fragments"),
            items: frames,
        });
    }

    // The messages: each line and its arrowhead.
    let mut lines = Vec::new();
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
        if m.refused {
            marks.extend(cross(&drawn));
        }
    }
    items.push(Item::Group {
        of: GroupOf::Class("edges"),
        items: lines,
    });

    // The bars, over the lines that start and end at them.
    if !layout.activations.is_empty() {
        items.push(Item::Group {
            of: GroupOf::Class("activations"),
            items: layout
                .activations
                .iter()
                .map(|&b| {
                    let r = at(b);
                    Item::Rect {
                        class: "activation".into(),
                        x: r.x,
                        y: r.y,
                        w: r.w,
                        h: r.h,
                        rx: Some(r.w / 2.0),
                    }
                })
                .collect(),
        });
    }

    // Each fragment's pills, over the bars that pass them.
    let tags: Vec<Item> = layout
        .fragments
        .iter()
        .flat_map(|f| f.pills.iter().flat_map(|(lines, b)| tag(lines, at(*b))))
        .collect();
    if !tags.is_empty() {
        items.push(Item::Group {
            of: GroupOf::Class("fragment-pills"),
            items: tags,
        });
    }

    // Each label in its pill, over its line.
    let pills: Vec<Item> = layout
        .messages
        .iter()
        .filter_map(|m| m.pill.as_ref().map(|p| pill(m, p, at(p.rect))))
        .flatten()
        .collect();
    if !pills.is_empty() {
        items.push(Item::Group {
            of: GroupOf::Class("labels"),
            items: pills,
        });
    }
    if !marks.is_empty() {
        items.push(Item::Group {
            of: GroupOf::Class("refusals"),
            items: marks,
        });
    }

    // The heads, each at the top of its band; a refusing one edged in the
    // refusal colour.
    for (node, r) in layout.spec.nodes.iter().zip(&layout.heads) {
        let brand = card::Brand {
            class: node
                .tech
                .as_deref()
                .filter(|t| brands.contains_key(*t))
                .map(super::brand_class),
        };
        items.push(match layout.look {
            Look::Cards => card::card(
                node,
                at(*r),
                (layout.spec.card, layout.spec.logo),
                logos,
                Some(&brand),
            ),
            Look::Avatars => card::avatar(node, at(*r), layout.spec.logo, logos, Some(&brand)),
        });
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
        let node = &layout.spec.nodes[p];
        let head = at(layout.heads[p]);
        items.push(match layout.look {
            Look::Cards => {
                let c = card::front(node, head);
                Item::Rect {
                    class: "refused-head".into(),
                    x: c.x,
                    y: c.y,
                    w: c.w,
                    h: c.h,
                    rx: Some(ROUNDED_CARD.min(c.w / 2.0).min(c.h / 2.0)),
                }
            }
            Look::Avatars => {
                let (cx, cy) = card::avatar_centre(node, head);
                Item::Circle {
                    class: "refused-head".into(),
                    cx,
                    cy,
                    r: AVATAR_SIZE / 2.0,
                }
            }
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
/// arrowhead, over a patch of the band so the line does not cross it.
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

/// A fragment's pill on its frame's line, fully round, its operator and
/// guard centred in it (UML 2.5.1's pentagon and brackets give way to it).
fn tag(lines: &[String], r: Rect) -> Vec<Item> {
    let line = TYPOGRAPHY_FRAME_LABEL.size * TYPOGRAPHY_FRAME_LABEL.line_height;
    #[allow(clippy::cast_precision_loss)] // one or two lines
    let top = r.y + (r.h - lines.len() as f64 * line) / 2.0;
    let mut out = vec![Item::Rect {
        class: "fragment-tag".into(),
        x: r.x,
        y: r.y,
        w: r.w,
        h: r.h,
        rx: Some(r.h / 2.0),
    }];
    out.extend(lines.iter().enumerate().map(|(i, l)| {
        #[allow(clippy::cast_precision_loss)] // one or two lines
        let down = i as f64 * line;
        Item::Text {
            class: "fragment-op".into(),
            x: r.centre_x(),
            y: top + down + font::baseline_in_line(&TYPOGRAPHY_FRAME_LABEL),
            anchor: Anchor::Middle,
            text: l.clone(),
        }
    }));
    out
}

/// A message's label in its pill (DESIGN.md, Components: Sequence, Message
/// label): its number, muted, then its words; a reply's words quieter than
/// a call's, a refused message's in the refusal colour.
fn pill(m: &Message, p: &Pill, r: Rect) -> Vec<Item> {
    let line = TYPOGRAPHY_SUBTITLE.size * TYPOGRAPHY_SUBTITLE.line_height;
    let refused = if m.refused { " refused" } else { "" };
    let mut out = vec![Item::Rect {
        class: format!("message-pill{refused}"),
        x: r.x,
        y: r.y,
        w: r.w,
        h: r.h,
        rx: Some(r.h / 2.0),
    }];
    let mut x = r.x + LABEL_PILL_PAD_X;
    if let Some(n) = p.number {
        let text = n.to_string();
        out.push(Item::Text {
            class: format!("message-number{refused}"),
            x,
            y: r.y + (r.h - line) / 2.0 + font::baseline_in_line(&TYPOGRAPHY_LEGEND),
            anchor: Anchor::Start,
            text: text.clone(),
        });
        x += font::text_width(&text, &TYPOGRAPHY_LEGEND) + SPACING_LABEL_GAP;
    }
    let sort = if m.sort == Sort::Reply { " reply" } else { "" };
    out.extend(p.lines.iter().enumerate().map(|(i, l)| {
        #[allow(clippy::cast_precision_loss)] // one or two lines
        let down = i as f64 * line;
        Item::Text {
            class: format!("message-text{sort}{refused}"),
            x,
            y: r.y + LABEL_PILL_PAD_Y + down + font::baseline_in_line(&TYPOGRAPHY_SUBTITLE),
            anchor: Anchor::Start,
            text: l.clone(),
        }
    }));
    out
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
    for p in layout.messages.iter().filter_map(|m| m.pill.as_ref()) {
        runs.extend(
            p.lines
                .iter()
                .map(|l| (TYPOGRAPHY_SUBTITLE.weight, l.clone())),
        );
        if let Some(n) = p.number {
            runs.push((TYPOGRAPHY_LEGEND.weight, n.to_string()));
        }
    }
    for f in &layout.fragments {
        for (lines, _) in &f.pills {
            runs.extend(
                lines
                    .iter()
                    .map(|l| (TYPOGRAPHY_FRAME_LABEL.weight, l.clone())),
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
