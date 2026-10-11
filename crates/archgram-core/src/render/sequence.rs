//! Drawing a laid-out sequence diagram (DESIGN.md, Components: Sequence),
//! as a blueprint: a faint grid, each phase's band, each participant's head
//! and lifeline as its look draws them, each fragment's frame and pills,
//! each message's line with UML's arrowhead for its sort in its kind's
//! colour and its label above it on a patch, the bars while a call is
//! answered, a refused message's ✕, and the key; and the motion, one phase
//! at a time, a point travelling each message, the drawing readable
//! throughout (DESIGN.md, Layout: Motion). Under reduced motion it is still.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::color::Rgb;
use crate::font;
use crate::geometry::{Point, Rect};
use crate::layout::sequence::{Label, Layout, Message, Sort};
use crate::motion::{SequenceMotion, fade, hop_gap};
use crate::sequence::{Look, Operator, Sequence, Step};
use crate::tokens::{
    ACTIVATION_WIDTH, ARROWHEAD_LENGTH, AVATAR_SIZE, FONT_MONO, FONT_SANS, GRID_DOT, GRID_SPACING,
    LABEL_PATCH_PAD, MOTION_DIM, MOTION_TINT, PULSE_RADIUS, REFUSAL_MARK, REFUSAL_MARK_GAP,
    ROUNDED_CANVAS, ROUNDED_CARD, ROUNDED_FRAME, SPACING_FRAGMENT_TAG, SPACING_FRAGMENT_TAG_PAD,
    SPACING_LABEL_GAP, SPACING_MARGIN, STROKE_MESSAGE, TYPOGRAPHY_FRAGMENT, TYPOGRAPHY_LEGEND,
    TYPOGRAPHY_MESSAGE, TYPOGRAPHY_MESSAGE_NUMBER, TYPOGRAPHY_MESSAGE_REPLY, TYPOGRAPHY_PHASE,
};

use super::scene::{Anchor, GroupOf, Item, Scene};
use super::signal::KeyTrack;
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
    let width = layout.size.w + 2.0 * SPACING_MARGIN;
    let height = layout.size.h + 2.0 * SPACING_MARGIN;
    let at = |r: Rect| Rect {
        x: r.x + OFFSET,
        y: r.y + OFFSET,
        ..r
    };
    let on_line = |p: Point| Point {
        x: p.x + EDGE_OFFSET,
        y: p.y + EDGE_OFFSET,
    };
    let brands = super::brands(&layout.spec, logos);
    let style = style(seq, layout, options, logos, &brands);
    // Every message's line as drawn, on the canvas, and the motion's timing.
    let drawn: Vec<edge::Drawn> = layout
        .messages
        .iter()
        .map(|m| edge::drawn(&m.path.iter().map(|&p| on_line(p)).collect::<Vec<_>>()))
        .collect();
    let lengths: Vec<f64> = drawn.iter().map(edge::Drawn::length).collect();
    // Each line up to its arrowhead's base, so a drawn line and its head
    // meet as one.
    let shafts: Vec<String> = layout
        .messages
        .iter()
        .map(|m| {
            let mut points: Vec<Point> = m.path.iter().map(|&p| on_line(p)).collect();
            let last = points.len().saturating_sub(1);
            if last >= 1 {
                // A message's last stretch runs straight across the page.
                let (from, tip) = (points[last - 1], points[last]);
                let length = ((tip.x - from.x).abs() + (tip.y - from.y).abs()).max(f64::EPSILON);
                let back = ARROWHEAD_LENGTH.min(length);
                points[last] = Point {
                    x: tip.x - (tip.x - from.x) / length * back,
                    y: tip.y - (tip.y - from.y) / length * back,
                };
            }
            edge::drawn(&points).d()
        })
        .collect();
    let motion = crate::motion::sequence_motion(seq, &lengths);
    let period = motion.period;
    let in_phase: Vec<bool> = (0..layout.messages.len())
        .map(|k| {
            layout
                .phases
                .iter()
                .any(|p| p.messages.0 <= k && k < p.messages.1)
        })
        .collect();
    let mut items = vec![Item::Canvas {
        width,
        height,
        rx: ROUNDED_CANVAS,
    }];

    // The phases' bands, and the tint that grows across the one that
    // plays.
    let mut names = Vec::new();
    let mut patches = Vec::new();
    if !layout.phases.is_empty() {
        let mut drawn_phases = Vec::new();
        for (n, p) in layout.phases.iter().enumerate() {
            let r = at(p.band);
            drawn_phases.push(Item::Rect {
                class: "phase".into(),
                x: r.x,
                y: r.y,
                w: r.w,
                h: r.h,
                rx: Some(ROUNDED_FRAME),
            });
            // Its name, drawn later over the lifelines on a patch.
            let name = p.name.to_uppercase();
            let top = p.name_at.y + OFFSET;
            let y = top + font::baseline_in_line(&TYPOGRAPHY_PHASE);
            let x = p.name_at.x + OFFSET;
            let line = TYPOGRAPHY_PHASE.size * TYPOGRAPHY_PHASE.line_height;
            patches.push(patch(
                Rect {
                    x,
                    y: top,
                    w: font::text_width(&name, &TYPOGRAPHY_PHASE),
                    h: line,
                },
                true,
            ));
            names.push(Item::Text {
                class: "phase-label".into(),
                x,
                y,
                anchor: Anchor::Start,
                text: name.clone(),
            });
            if let Some(&(start, end)) = motion.phases.get(n) {
                names.push(Item::Motion(format!(
                    r#"<text class="phase-label lit motion" x="{}" y="{}" opacity="0">{}{}</text>"#,
                    num(x),
                    num(y),
                    shown(period, start, end),
                    crate::render::svg::escape(&name)
                )));
                drawn_phases.push(tint(layout, &motion, n, end, at));
            }
        }
        items.push(Item::Group {
            of: GroupOf::Class("phases"),
            items: drawn_phases,
        });
    }

    // The grid, over the canvas and the bands, under all else.
    items.push(grid(width, height));

    // The lifelines, under the messages.
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

    // The fragments' frames, outermost first.
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

    // A patch under each label and each phase's name, so a lifeline stops
    // at its words; then the phases' names.
    for (k, m) in layout.messages.iter().enumerate() {
        if let Some(l) = &m.label {
            patches.push(patch(at(l.rect), in_phase[k]));
        }
    }
    if !patches.is_empty() {
        items.push(Item::Group {
            of: GroupOf::Class("patches"),
            items: patches,
        });
    }
    if !names.is_empty() {
        items.push(Item::Group {
            of: GroupOf::Class("phase-names"),
            items: names,
        });
    }

    // Each message as one group, its line, arrowhead, ✕ and label, dimmed
    // while another phase plays.
    let mut messages = Vec::new();
    for (k, (m, d)) in layout.messages.iter().zip(&drawn).enumerate() {
        let mut parts = vec![
            Item::Path {
                id: None,
                class: format!("edge {}", line_class(m)),
                place: None,
                d: d.d(),
                arrowhead: false,
            },
            arrowhead(m, d),
        ];
        if m.refused {
            parts.extend(cross(d, in_phase[k]));
        }
        if let Some(l) = &m.label {
            parts.extend(label(m, l, at(l.rect)));
        }
        messages.push(Item::Motion(format!(
            r#"<g class="dims">{}{}</g>"#,
            dims(period, &motion.shown[k]),
            crate::render::svg::inline(&parts)
        )));
    }
    items.push(Item::Group {
        of: GroupOf::Class("messages"),
        items: messages,
    });

    // The bars, over the lines that start and end at them.
    if !layout.activations.is_empty() {
        items.push(Item::Group {
            of: GroupOf::Class("activations"),
            items: layout
                .activations
                .iter()
                .map(|a| {
                    let r = at(a.rect);
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

    // Each message's line drawn as it plays, its arrowhead taking the
    // line's colour the moment the line reaches it.
    items.push(Item::Group {
        of: GroupOf::Class("motion"),
        items: layout
            .messages
            .iter()
            .zip(&drawn)
            .enumerate()
            .map(|(k, (m, d))| drawing(m, (d, &shafts[k]), period, motion.hops[k], motion.held[k]))
            .collect(),
    });

    // Each fragment's pills, then each "or" by an `alt`'s later ways.
    let mut tags: Vec<Item> = layout
        .fragments
        .iter()
        .flat_map(|f| f.pills.iter().flat_map(|(lines, b)| tag(lines, at(*b))))
        .collect();
    for &(f, j, from, until) in &motion.ors {
        if let Some((_, b)) = layout.fragments.get(f).and_then(|fr| fr.pills.get(j)) {
            tags.push(or(at(*b), period, (from, until)));
        }
    }
    if !tags.is_empty() {
        items.push(Item::Group {
            of: GroupOf::Class("fragment-pills"),
            items: tags,
        });
    }

    // The heads, each at the top of its lifeline; a refusing one's edge in
    // the refusal colour while its refusal plays.
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
    for (k, m) in layout.messages.iter().enumerate() {
        if !m.refused {
            continue;
        }
        let node = &layout.spec.nodes[m.refuser];
        let head = at(layout.heads[m.refuser]);
        let (s, e) = motion.hops[k];
        let while_shown = shown(period, s, motion.held[k].max(e + hop_gap()).min(period));
        items.push(Item::Motion(match layout.look {
            Look::Cards => {
                let c = card::front(node, head);
                format!(
                    r#"<rect class="refused-head motion" x="{}" y="{}" width="{}" height="{}" rx="{}" opacity="0">{while_shown}</rect>"#,
                    num(c.x),
                    num(c.y),
                    num(c.w),
                    num(c.h),
                    num(ROUNDED_CARD.min(c.w / 2.0).min(c.h / 2.0))
                )
            }
            Look::Avatars => {
                let (cx, cy) = card::avatar_centre(node, head);
                format!(
                    r#"<circle class="refused-head motion" cx="{}" cy="{}" r="{}" opacity="0">{while_shown}</circle>"#,
                    num(cx),
                    num(cy),
                    num(AVATAR_SIZE / 2.0)
                )
            }
        }));
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
        width,
        height,
        title: seq.title.clone(),
        description: description(seq),
        style,
        items,
    }
}

/// The classes of a message's line, by its kind: a call's, a send's or a
/// reply's; and a refused one's.
fn line_class(m: &Message) -> String {
    let sort = match m.sort {
        Sort::Call => "call",
        Sort::Send => "send",
        Sort::Reply => "reply",
    };
    if m.refused {
        format!("{sort} refused")
    } else {
        sort.into()
    }
}

/// A message's arrowhead, drawn on its own (DESIGN.md, Components:
/// Sequence): a call's filled, a send's and a reply's open, in its line's
/// colour.
fn arrowhead(m: &Message, drawn: &edge::Drawn) -> Item {
    let chevron = drawn.chevron();
    let filled = m.sort == Sort::Call && !m.refused;
    Item::Path {
        id: None,
        class: format!(
            "arrowhead {}{}",
            line_class(m),
            if filled { " filled" } else { "" }
        ),
        place: None,
        d: if filled {
            format!("{chevron}Z")
        } else {
            chevron
        },
        arrowhead: false,
    }
}

/// The ✕ on a refused message's line, `refusal.mark-gap` before its
/// arrowhead, over a patch of what it sits on (a phase's band, or the
/// canvas) so the line does not cross it.
fn cross(drawn: &edge::Drawn, on_band: bool) -> [Item; 2] {
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
    [
        path(if on_band {
            "refused-patch band"
        } else {
            "refused-patch"
        }),
        path("refused-mark"),
    ]
}

/// The faint grid of dots behind a sequence (DESIGN.md, Components:
/// Sequence, Grid): a pattern of dots `grid.spacing` apart, from the
/// drawing's margin, filling the canvas in its rounded corners.
fn grid(width: f64, height: f64) -> Item {
    Item::Motion(format!(
        r#"<defs><pattern id="archgram-grid" x="{m}" y="{m}" width="{g}" height="{g}" patternUnits="userSpaceOnUse"><circle class="grid-dot" cx="{d}" cy="{d}" r="{d}"/></pattern></defs><rect class="grid" width="{}" height="{}" rx="{}" fill="url(#archgram-grid)"/>"#,
        num(width),
        num(height),
        num(ROUNDED_CANVAS),
        m = num(SPACING_MARGIN - GRID_DOT),
        g = num(GRID_SPACING),
        d = num(GRID_DOT),
    ))
}

/// A patch of what a label or a phase's name sits on, `label.patch-pad`
/// beyond its box: a phase's band, or the canvas.
fn patch(r: Rect, on_band: bool) -> Item {
    Item::Rect {
        class: if on_band {
            "label-patch band".into()
        } else {
            "label-patch".into()
        },
        x: r.x - LABEL_PATCH_PAD,
        y: r.y - LABEL_PATCH_PAD,
        w: r.w + 2.0 * LABEL_PATCH_PAD,
        h: r.h + 2.0 * LABEL_PATCH_PAD,
        rx: Some(LABEL_PATCH_PAD),
    }
}

/// A fragment's pill on its frame's line, fully round, its operator and
/// guard centred in it (UML 2.5.1's pentagon and brackets give way to it).
fn tag(lines: &[String], r: Rect) -> Vec<Item> {
    let line = TYPOGRAPHY_FRAGMENT.size * TYPOGRAPHY_FRAGMENT.line_height;
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
            y: top + down + font::baseline_in_line(&TYPOGRAPHY_FRAGMENT),
            anchor: Anchor::Middle,
            text: l.clone(),
        }
    }));
    out
}

/// A message's label above its line (DESIGN.md, Components: Sequence,
/// Message label): its number in its kind's colour, then its words; a
/// reply's words quieter than a call's, a refused message's in the
/// refusal colour.
fn label(m: &Message, l: &Label, r: Rect) -> Vec<Item> {
    // The number leads the first line; a second line starts under the
    // first's words.
    let line = TYPOGRAPHY_MESSAGE.size * TYPOGRAPHY_MESSAGE.line_height;
    let refused = if m.refused { " refused" } else { "" };
    let reply = m.sort == Sort::Reply;
    let words = if reply {
        TYPOGRAPHY_MESSAGE_REPLY
    } else {
        TYPOGRAPHY_MESSAGE
    };
    let mut out = Vec::new();
    let mut x = r.x;
    if let Some(n) = &l.number {
        out.push(Item::Text {
            class: format!("message-number {}", line_class(m)),
            x,
            y: r.y + font::baseline_in_line(&TYPOGRAPHY_MESSAGE_NUMBER),
            anchor: Anchor::Start,
            text: n.clone(),
        });
        x += font::text_width(n, &TYPOGRAPHY_MESSAGE_NUMBER) + SPACING_LABEL_GAP;
    }
    let sort = if reply { " reply" } else { "" };
    out.extend(l.lines.iter().enumerate().map(|(i, text)| {
        #[allow(clippy::cast_precision_loss)] // one or two lines
        let down = i as f64 * line;
        Item::Text {
            class: format!("message-text{sort}{refused}"),
            x,
            y: r.y + down + font::baseline_in_line(&words),
            anchor: Anchor::Start,
            text: text.clone(),
        }
    }));
    out
}

/// Opacity for something shown from `start` to `end` in a cycle `period`
/// long, fading in and out over `motion.fade`.
fn shown(period: u32, start: u32, end: u32) -> String {
    let f = fade();
    let mut t = KeyTrack::new(period);
    t.key(0, "0")
        .key(start.saturating_sub(f), "0")
        .key(start, "1")
        .key(end, "1")
        .key((end + f).min(period), "0")
        .key(period, "0");
    format!(
        r#"<animate attributeName="opacity" {}/>"#,
        t.attributes("values")
    )
}

/// A message's strength through the cycle: full while `full` says, dimmed
/// to `motion.dim` otherwise, each change over `motion.fade`.
fn dims(period: u32, full: &[(u32, u32)]) -> String {
    let f = fade();
    let dim = num(MOTION_DIM);
    let mut spans: Vec<(u32, u32)> = full.to_vec();
    spans.sort_unstable();
    let mut t = KeyTrack::new(period);
    let first = if spans.first().is_some_and(|s| s.0 == 0) {
        "1"
    } else {
        dim.as_str()
    };
    t.key(0, first);
    let mut last = 0;
    for (a, b) in spans {
        let a = a.max(last);
        let b = b.max(a).min(period);
        t.key(
            a.saturating_sub(f).max(last),
            if a == 0 { "1" } else { dim.as_str() },
        );
        t.key(a, "1");
        t.key(b, "1");
        let out = (b + f).min(period);
        t.key(out, if b >= period { "1" } else { dim.as_str() });
        last = out;
    }
    t.key(period, if last >= period { "1" } else { dim.as_str() });
    format!(
        r#"<animate attributeName="opacity" {}/>"#,
        t.attributes("values")
    )
}

/// A message's line drawn from its sender over its hop, at its own width,
/// with a point travelling it while it is drawn, and its arrowhead in that
/// colour from the moment the line reaches it; all held until `held`, then
/// faded.
fn drawing(
    m: &Message,
    (d, shaft): (&edge::Drawn, &str),
    period: u32,
    (s, e): (u32, u32),
    held: u32,
) -> Item {
    let class = line_class(m);
    let mut drawn_in = KeyTrack::new(period);
    drawn_in
        .key(0, "1000")
        .key(s, "1000")
        .key(e, "0")
        .key(period, "0");
    let mut lit_head = KeyTrack::new(period);
    lit_head
        .key(0, "0")
        .key(e.saturating_sub(1), "0")
        .key(e, "1")
        .key(period, "1");
    // The point: at the line's start until it leaves, at its end as it
    // arrives, seen only between.
    let mut along = KeyTrack::new(period);
    along.key(0, "0").key(s, "0").key(e, "1").key(period, "1");
    let mut seen = KeyTrack::new(period);
    seen.key(0, "0")
        .key(s, "0")
        .key(s, "1")
        .key(e, "1")
        .key(e, "0")
        .key(period, "0");
    let held = held.max(e + hop_gap()).min(period);
    let filled = m.sort == Sort::Call && !m.refused;
    let chevron = d.chevron();
    let head_d = if filled {
        format!("{chevron}Z")
    } else {
        chevron
    };
    Item::Motion(format!(
        r#"<g class="drawn {class}" opacity="0">{}<path class="shaft" d="{shaft}" pathLength="1000" stroke-dasharray="1000 1000"><animate attributeName="stroke-dashoffset" {}/></path><circle class="pulse {class}" r="{}" opacity="0"><animate attributeName="opacity" {}/><animateMotion calcMode="linear" path="{shaft}" {}/></circle><path class="head{}" d="{head_d}" opacity="0"><animate attributeName="opacity" {}/></path></g>"#,
        shown(period, s, held),
        drawn_in.attributes("values"),
        num(PULSE_RADIUS),
        seen.attributes("values"),
        along.attributes("keyPoints"),
        if filled { " filled" } else { "" },
        lit_head.attributes("values"),
    ))
}

/// The tint across phase `n`'s band, between the lifelines its messages
/// join: grown with each message from its sender's toward its receiver's,
/// kept until the phase ends, started again at each later way of an
/// `alt`.
fn tint(
    layout: &Layout,
    motion: &SequenceMotion,
    index: usize,
    phase_end: u32,
    at: impl Fn(Rect) -> Rect,
) -> Item {
    let period = motion.period;
    let phase = &layout.phases[index];
    let band = at(phase.band);
    let fading = fade();
    let mut left = KeyTrack::new(period);
    let mut width = KeyTrack::new(period);
    let mut seen = KeyTrack::new(period);
    let (first, after) = phase.messages;
    let mut order: Vec<usize> = (first..after).collect();
    order.sort_by_key(|&k| motion.hops[k].0);
    // Where a message starts, and how far across it reaches: its
    // receiver, or for a message to itself its loop.
    let side = |k: usize| {
        let m = &layout.messages[k];
        let from = m.path[0].x;
        let reach = if m.from == m.to {
            m.path.iter().map(|p| p.x).fold(from, f64::max)
        } else {
            m.path.last().map_or(from, |p| p.x)
        };
        (from + OFFSET, reach + OFFSET)
    };
    let (origin, _) = side(order[0]);
    left.key(0, &num(origin));
    width.key(0, "0");
    seen.key(0, "0");
    let mut span: Option<(f64, f64)> = None;
    let mut last = 0;
    for &k in &order {
        let (from, reach) = side(k);
        if let Some(&(_, rewind)) = motion.resets.iter().find(|r| r.0 == k) {
            // The way before dims back: the tint goes, and starts again.
            let rewind = rewind.max(last);
            let gone = (rewind + fading).min(motion.hops[k].0.max(rewind));
            seen.key(rewind, "1").key(gone, "0");
            span = None;
            last = gone;
        }
        let leaves = motion.hops[k].0.max(last);
        let arrives = motion.hops[k].1.max(leaves);
        let (lo, hi) = span.unwrap_or((from, from));
        if span.is_none() {
            left.key(leaves, &num(lo));
            width.key(leaves, "0");
            seen.key(leaves, "0").key(leaves, "1");
        } else {
            left.key(leaves, &num(lo));
            width.key(leaves, &num(hi - lo));
        }
        let grown = (lo.min(from).min(reach), hi.max(from).max(reach));
        left.key(arrives, &num(grown.0));
        width.key(arrives, &num(grown.1 - grown.0));
        span = Some(grown);
        last = arrives;
    }
    let end = phase_end.max(last);
    seen.key(end, "1")
        .key((end + fading).min(period), "0")
        .key(period, "0");
    left.key(period, &span.map_or(num(origin), |s| num(s.0)));
    width.key(period, &span.map_or("0".into(), |s| num(s.1 - s.0)));
    Item::Motion(format!(
        r#"<rect class="phase-tint motion" x="{}" y="{}" width="0" height="{}" opacity="0"><animate attributeName="x" {}/><animate attributeName="width" {}/><animate attributeName="opacity" {}/></rect>"#,
        num(origin),
        num(band.y + 3.0),
        num((band.h - 6.0).max(0.0)),
        left.attributes("values"),
        width.attributes("values"),
        seen.attributes("values")
    ))
}

/// The "or" after the pill `b` of an `alt`'s later way, shown from its
/// rewind to its phase's end.
fn or(b: Rect, period: u32, (from, until): (u32, u32)) -> Item {
    let width = font::text_width("or", &TYPOGRAPHY_FRAGMENT) + 2.0 * SPACING_FRAGMENT_TAG_PAD;
    let h = SPACING_FRAGMENT_TAG;
    let x = b.right() + SPACING_LABEL_GAP;
    let y = b.centre_y() - h / 2.0;
    let line = TYPOGRAPHY_FRAGMENT.size * TYPOGRAPHY_FRAGMENT.line_height;
    Item::Motion(format!(
        r#"<g class="motion" opacity="0">{}<rect class="fragment-or" x="{}" y="{}" width="{}" height="{}" rx="{}"/><text class="fragment-or-text" x="{}" y="{}" text-anchor="middle">or</text></g>"#,
        shown(period, from, until.max(from)),
        num(x),
        num(y),
        num(width),
        num(h),
        num(h / 2.0),
        num(x + width / 2.0),
        num(y + (h - line) / 2.0 + font::baseline_in_line(&TYPOGRAPHY_FRAGMENT))
    ))
}

/// The style sheet: the theme, the fonts with every text the drawing sets,
/// the shapes every drawing has, then a sequence's own and its motion.
fn style(
    seq: &Sequence,
    layout: &Layout,
    options: Options,
    logos: &dyn crate::logos::Logos,
    brands: &BTreeMap<String, Rgb>,
) -> Vec<super::scene::StyleLine> {
    let (light, dark) = super::colors(&seq.palette, options);
    let refused = layout.messages.iter().any(|m| m.refused);
    let mut sheet = Sheet::default();
    // The refusal colour is a signal's role, written only where it is used;
    // the messages' own colours, a sequence's alone.
    super::theme_vars(&mut sheet, options.mode, (&light, &dark), (refused, true));
    let runs = text_runs(layout, logos);
    if options.embed_font && super::embed_runs(&mut sheet, runs.into_iter()) {
        sheet.line(&format!(
            "text {{ font-family: \"{}\", {FONT_SANS}; font-kerning: none; --mono: \"{}\", {FONT_MONO}; }}",
            font::FAMILY,
            font::FAMILY_MONO
        ));
    } else {
        sheet.line(&format!(
            "text {{ font-family: {FONT_SANS}; --mono: {FONT_MONO}; }}"
        ));
    }
    sheet.rules(styles::shapes());
    super::brand_rules(&mut sheet, brands, (&light, &dark, options.mode));
    sheet.rules(styles::sequence());
    if refused {
        sheet.rules(styles::sequence_refusal());
    }
    // The motion: a drawn line at its own width, its head in its colour;
    // the tint; the "or". Under reduced motion, none of it, and every
    // message at full strength.
    sheet.line(&format!(
        ".drawn path {{ fill: none; stroke: var(--message-call); stroke-linecap: butt; stroke-linejoin: round; }} .drawn .shaft {{ stroke-width: {}; }} .drawn .head {{ stroke-width: {}; }} .drawn .head.filled {{ fill: var(--message-call); }} .drawn.send path {{ stroke: var(--message-send); }} .drawn.reply path {{ stroke: var(--connector); }}",
        num(STROKE_MESSAGE),
        num(STROKE_MESSAGE)
    ));
    sheet.line(".pulse { fill: var(--message-call); } .pulse.send { fill: var(--message-send); } .pulse.reply { fill: var(--connector); }");
    if refused {
        sheet.line(".drawn.refused path { stroke: var(--signal-refusal); } .pulse.refused { fill: var(--signal-refusal); }");
    }
    sheet.line(&format!(
        ".phase-tint {{ fill: var(--text); fill-opacity: {}; }}",
        num(MOTION_TINT)
    ));
    sheet.line(".fragment-or { fill: var(--text); }");
    sheet.line(&format!(
        ".fragment-or-text {{ font-family: var(--mono); font-size: {}px; font-weight: {}; fill: var(--canvas); }}",
        num(TYPOGRAPHY_FRAGMENT.size),
        TYPOGRAPHY_FRAGMENT.weight
    ));
    sheet.line("@media (prefers-reduced-motion: reduce) { .motion, .drawn { display: none; } .dims { opacity: 1 !important; } }");
    if seq.credit {
        sheet.rules(styles::credit());
    }
    if !layout.legend.is_empty() {
        sheet.rules(styles::legend());
    }
    let _ = ACTIVATION_WIDTH;
    sheet.0
}

/// A face to embed or measure: its family and weight.
type Face = (font::Family, u16);

/// The face a style is set in.
fn face(style: &crate::tokens::TextStyle) -> Face {
    (style.family, style.weight)
}

/// Every text the drawing sets, with its face: the heads' and the key's in
/// the sans face, and each label, number, operator, guard and phase's name
/// in the mono one. For the fonts' subsets, and for the characters they
/// lack.
#[must_use]
pub fn text_runs(layout: &Layout, logos: &dyn crate::logos::Logos) -> Vec<(Face, String)> {
    let mut runs: Vec<(Face, String)> = crate::measure::text_runs_with(&layout.spec, logos)
        .into_iter()
        .map(|(w, t)| ((font::Family::Sans, w), t.into_owned()))
        .collect();
    runs.extend(
        layout
            .legend
            .iter()
            .map(|e| (face(&TYPOGRAPHY_LEGEND), e.text.to_owned())),
    );
    for m in &layout.messages {
        let Some(l) = &m.label else { continue };
        let words = if m.sort == Sort::Reply {
            face(&TYPOGRAPHY_MESSAGE_REPLY)
        } else {
            face(&TYPOGRAPHY_MESSAGE)
        };
        runs.extend(l.lines.iter().map(|t| (words, t.clone())));
        if let Some(n) = &l.number {
            runs.push((face(&TYPOGRAPHY_MESSAGE_NUMBER), n.clone()));
        }
    }
    for f in &layout.fragments {
        for (lines, _) in &f.pills {
            runs.extend(
                lines
                    .iter()
                    .map(|l| (face(&TYPOGRAPHY_FRAGMENT), l.clone())),
            );
        }
    }
    for p in &layout.phases {
        runs.push((face(&TYPOGRAPHY_PHASE), p.name.to_uppercase()));
    }
    runs.push((face(&TYPOGRAPHY_FRAGMENT), "or".into()));
    runs
}

/// The description, then each message in words, in order, for a screen
/// reader, which sees neither the lines nor their order down the page: a
/// phase's name before its messages, a message's number, who sends it to
/// whom and its label, each fragment's guard before its messages. Read
/// from the same steps the layout draws, so the words and the drawing
/// never disagree.
fn description(seq: &Sequence) -> String {
    let label = |p: usize| seq.participants[p].label.trim().to_owned();
    let mut out = seq.description.trim().to_owned();
    let mut open: Vec<Operator> = Vec::new();
    let steps = seq.steps();
    let numbers = crate::layout::sequence::numbers(&steps);
    let mut n = 0usize;
    for step in steps {
        match step {
            Step::Message(m) => {
                let verb = match m.sort {
                    Sort::Call => "calls",
                    Sort::Send => "sends to",
                    Sort::Reply => "replies to",
                };
                let _ = write!(
                    out,
                    " {}. {} {verb} {}",
                    numbers[n],
                    label(m.from),
                    label(m.to)
                );
                n += 1;
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
            Step::Phase(name) => {
                let _ = write!(out, " {}:", name.trim());
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
