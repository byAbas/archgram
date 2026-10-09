//! Laying out a sequence diagram (docs/features/sequence.md; DESIGN.md,
//! Layout): the participants' heads in a row, a lifeline down from each,
//! one row for each message in time's order, and a frame round each
//! fragment. Every position comes from the spec's order and the measured
//! text, so the same spec lays out the same everywhere.

use crate::font::text_width;
use crate::geometry::{Point, Rect, Size};
use crate::logos::Logos;
use crate::measure::{card_sizes_with, label_lines, label_size};
use crate::sequence::{Item, Operator, Sequence, SequenceStill, What};
use crate::spec::{Spec, Variant};
use crate::tokens::{
    ARROWHEAD_GAP, ARROWHEAD_LENGTH, CARD_MULTI_OFFSET, REFUSAL_MARK, REFUSAL_MARK_GAP,
    SIGNAL_NUMBER, SPACING_EDGE_EDGE, SPACING_FRAGMENT_PADDING, SPACING_FRAGMENT_TAG,
    SPACING_LIFELINE_GAP, SPACING_MESSAGE_ROW, SPACING_SELF_WIDTH, STROKE_CONNECTOR,
    TYPOGRAPHY_FRAME_LABEL, TYPOGRAPHY_LEGEND, TYPOGRAPHY_SUBTITLE,
};

use super::legend;

/// How a message is drawn (UML 2.5.1, 17.4.4.1): a call waits for its
/// reply, a send does not, a reply answers a call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sort {
    Call,
    Send,
    Reply,
}

/// One message as laid out.
#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub from: usize,
    pub to: usize,
    pub sort: Sort,
    pub refused: bool,
    /// Its number, counted in time's order from 1.
    pub number: u32,
    /// Its line, from the sender's lifeline to the receiver's: the drawing
    /// stops it `arrowhead.gap` short of its end.
    pub path: Vec<Point>,
    /// Its label's lines and the box they fill.
    pub label: Option<(Vec<String>, Rect)>,
    /// Its number's pill, when the still image numbers the messages.
    pub pill: Option<Rect>,
    /// The participant that turns the request away: who replies with a
    /// refusal, or who refuses a call.
    pub refuser: usize,
}

/// One fragment as laid out.
#[derive(Debug, Clone, PartialEq)]
pub struct Fragment {
    pub operator: Operator,
    pub frame: Rect,
    /// The pentagon at its top left, the operator in it.
    pub tag: Rect,
    /// Each guard written, as `[when]`, and its box.
    pub guards: Vec<(String, Rect)>,
    /// Where each operand after the first starts: a dashed line across.
    pub separators: Vec<f64>,
}

/// A sequence laid out.
#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    /// The spec the heads are drawn from: the participants as nodes.
    pub spec: Spec,
    /// Each head's footprint, in the participants' order.
    pub heads: Vec<Rect>,
    /// Each lifeline's x.
    pub lifelines: Vec<f64>,
    /// Where every lifeline starts (the heads' foot) and ends.
    pub lifeline_span: (f64, f64),
    pub messages: Vec<Message>,
    /// Outermost first, then in time's order.
    pub fragments: Vec<Fragment>,
    pub legend: Vec<legend::Entry>,
    pub credit: Option<Rect>,
    pub size: Size,
}

/// A line of `typography.subtitle`.
fn line_height() -> f64 {
    TYPOGRAPHY_SUBTITLE.size * TYPOGRAPHY_SUBTITLE.line_height
}

/// The gap between a label and the pill or line under it.
const LABEL_GAP: f64 = 4.0;

/// The room inside a tag round its operator, either side; its lower right
/// corner is cut by as much again.
const TAG_PAD: f64 = 6.0;

/// A guard's text and the room it needs.
fn guard_text(when: &str) -> String {
    format!("[{}]", when.trim())
}

/// The pill a number needs: as wide as its text and the room either side a
/// single digit has, as a step's number (DESIGN.md, Components: Signal).
#[must_use]
pub fn pill_width(number: u32) -> f64 {
    let h = SIGNAL_NUMBER;
    h.max((text_width(&number.to_string(), &TYPOGRAPHY_LEGEND) + h - TYPOGRAPHY_LEGEND.size).ceil())
}

/// How far back from a line's tip its ✕ reaches, its patch of canvas
/// included (DESIGN.md, Components: Refusal).
fn mark_reach() -> f64 {
    ARROWHEAD_LENGTH + REFUSAL_MARK_GAP + REFUSAL_MARK + 1.5 * STROKE_CONNECTOR
}

/// A message read in time's order, its reply's receiver resolved.
struct Read {
    from: usize,
    to: usize,
    sort: Sort,
    refused: bool,
    label: Option<String>,
}

/// One step of the sequence read in order.
enum Step {
    Message(Read),
    /// A fragment opens, with its first operand's guard.
    Open(Operator, Option<String>),
    /// Its next operand starts, with its guard.
    Operand(Option<String>),
    Close,
}

/// The items in time's order, each reply sent to the caller of the call it
/// answers: within `alt`, `opt` and `loop` each way through starts from the
/// calls waiting before it, as validation reads them.
fn read(
    items: &[Item],
    index: &dyn Fn(&str) -> usize,
    open: &mut Vec<(usize, usize)>,
    out: &mut Vec<Step>,
) {
    for item in items {
        match item.what() {
            Ok(What::Message { from, to }) => {
                let (from, to) = (index(from), index(to));
                let sort = if item.sends { Sort::Send } else { Sort::Call };
                if sort == Sort::Call && !item.refused {
                    open.push((from, to));
                }
                out.push(Step::Message(Read {
                    from,
                    to,
                    sort,
                    refused: item.refused,
                    label: item.label.clone(),
                }));
            }
            Ok(What::Reply { by, to }) => {
                let by = index(by);
                let caller = open
                    .iter()
                    .rposition(|&(_, callee)| callee == by)
                    .map(|k| open.remove(k).0);
                let to = to.map(index).or(caller).unwrap_or(by);
                out.push(Step::Message(Read {
                    from: by,
                    to,
                    sort: Sort::Reply,
                    refused: item.refused,
                    label: item.label.clone(),
                }));
            }
            Ok(What::Fragment { operator, operands }) => {
                let before = open.clone();
                let mut after: Vec<(usize, usize)> = match operator {
                    Operator::Opt | Operator::Loop => before.clone(),
                    Operator::Alt | Operator::Par => Vec::new(),
                };
                for (j, operand) in operands.iter().enumerate() {
                    let when = operand.when.clone();
                    out.push(if j == 0 {
                        Step::Open(operator, when)
                    } else {
                        Step::Operand(when)
                    });
                    if operator == Operator::Par {
                        read(&operand.messages, index, open, out);
                        continue;
                    }
                    let mut way = before.clone();
                    read(&operand.messages, index, &mut way, out);
                    for call in way {
                        if !after.contains(&call) {
                            after.push(call);
                        }
                    }
                }
                if operator != Operator::Par {
                    let mut kept: Vec<(usize, usize)> = before
                        .iter()
                        .filter(|c| after.contains(c))
                        .copied()
                        .collect();
                    kept.extend(after.into_iter().filter(|c| !before.contains(c)));
                    *open = kept;
                }
                out.push(Step::Close);
            }
            // Validation has refused the spec.
            Err(_) => {}
        }
    }
}

/// A fragment while it is laid out: its rows, its participants, and the
/// fragments inside it.
struct Open {
    operator: Operator,
    top: f64,
    guards: Vec<(String, f64)>,
    separators: Vec<f64>,
    /// The participants its messages name.
    covered: Vec<usize>,
    /// How far right a message to itself inside it reaches, its label
    /// included.
    reach: f64,
    /// The fragments inside it, by their index in the list.
    inner: Vec<usize>,
    /// Where it ends, once closed.
    bottom: f64,
}

/// Lays a sequence out.
///
/// # Panics
///
/// When `seq` has not passed validation: a message names a participant
/// that does not exist.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn place(seq: &Sequence, logos: &dyn Logos) -> Layout {
    let spec = seq.as_spec();
    let sizes = card_sizes_with(&spec, logos);
    let count = sizes.len();
    let index = |id: &str| {
        seq.participants
            .iter()
            .position(|p| p.id == id)
            .expect("validated: every message names a participant")
    };
    let mut steps = Vec::new();
    read(&seq.messages, &index, &mut Vec::new(), &mut steps);
    let numbered = seq.still == SequenceStill::Numbers;

    // Each head's front card, and how far its footprint reaches either side
    // of the lifeline at the front card's middle.
    let front_w: Vec<f64> = seq
        .participants
        .iter()
        .zip(&sizes)
        .map(|(p, s)| {
            if p.variant == Variant::Multi {
                s.w - 2.0 * CARD_MULTI_OFFSET
            } else {
                s.w
            }
        })
        .collect();
    let left: Vec<f64> = front_w.iter().map(|w| w / 2.0).collect();
    let right: Vec<f64> = sizes
        .iter()
        .zip(&front_w)
        .map(|(s, w)| s.w - w / 2.0)
        .collect();

    // The room each message needs between its lifelines, and each message
    // to itself to the right of its own.
    let mut gaps: Vec<f64> = (0..count.saturating_sub(1))
        .map(|i| right[i] + SPACING_LIFELINE_GAP + left[i + 1])
        .collect();
    let mut needs: Vec<(usize, usize, f64)> = Vec::new();
    let mut self_reach: Vec<f64> = vec![0.0; count];
    let mut number = 0u32;
    for step in &steps {
        let Step::Message(msg) = step else { continue };
        number += 1;
        // A label's box holds its text and the padding of its patch.
        let label_w = msg.label.as_deref().map_or(0.0, |l| label_size(l).w);
        let pill = if numbered {
            pill_width(number) + SPACING_EDGE_EDGE
        } else {
            0.0
        };
        if msg.from == msg.to {
            let reach =
                SPACING_SELF_WIDTH + SPACING_EDGE_EDGE / 2.0 + pill + label_w + SPACING_EDGE_EDGE;
            self_reach[msg.from] = self_reach[msg.from].max(reach);
            if msg.from + 1 < count {
                needs.push((
                    msg.from,
                    msg.from + 1,
                    reach + left[msg.from + 1].min(SPACING_EDGE_EDGE),
                ));
            }
            continue;
        }
        let tail = ARROWHEAD_GAP
            + if msg.refused {
                mark_reach()
            } else {
                ARROWHEAD_LENGTH
            }
            + SPACING_EDGE_EDGE
            + pill;
        let need = (label_w + 2.0 * SPACING_EDGE_EDGE).max(tail + SPACING_EDGE_EDGE);
        needs.push((msg.from.min(msg.to), msg.from.max(msg.to), need));
    }
    // Shorter spans first, then from the left: each widens its own gaps
    // evenly, so a span of three never makes room a span of one needed.
    needs.sort_by(|a, b| {
        (a.1 - a.0, a.0)
            .cmp(&(b.1 - b.0, b.0))
            .then(b.2.total_cmp(&a.2))
    });
    for (lo, hi, need) in needs {
        let have: f64 = gaps[lo..hi].iter().sum();
        if have < need {
            #[allow(clippy::cast_precision_loss)] // a few participants
            let each = (need - have) / (hi - lo) as f64;
            for g in &mut gaps[lo..hi] {
                *g += each;
            }
        }
    }
    let first = left.first().copied().unwrap_or_default();
    let lifelines: Vec<f64> = std::iter::once(first)
        .chain(gaps.iter().scan(first, |at, g| {
            *at += g;
            Some(*at)
        }))
        .take(count)
        .collect();

    // The heads in a row, their front cards' feet on one line.
    let head_h = sizes.iter().map(|s| s.h).fold(0.0, f64::max);
    let heads: Vec<Rect> = sizes
        .iter()
        .zip(&lifelines)
        .zip(&left)
        .map(|((s, &c), &l)| Rect {
            x: c - l,
            y: head_h - s.h,
            w: s.w,
            h: s.h,
        })
        .collect();

    // Down the page: a row for each message, a frame round each fragment.
    let line = line_height();
    let half_pill = if numbered { SIGNAL_NUMBER / 2.0 } else { 0.0 };
    let drop = SPACING_MESSAGE_ROW / 2.0;
    let mut cursor = head_h + SPACING_EDGE_EDGE;
    let mut messages = Vec::new();
    let mut opened: Vec<usize> = Vec::new();
    let mut frames: Vec<Open> = Vec::new();
    let mut number = 0u32;
    for step in steps {
        match step {
            Step::Message(msg) => {
                number += 1;
                let lines: Vec<String> = msg
                    .label
                    .as_deref()
                    .map(|l| label_lines(l).into_iter().map(str::to_owned).collect())
                    .unwrap_or_default();
                #[allow(clippy::cast_precision_loss)] // one or two lines
                let label_h = lines.len() as f64 * line;
                let label_w = msg.label.as_deref().map_or(0.0, |l| label_size(l).w);
                let to_self = msg.from == msg.to;
                let above = if to_self {
                    SPACING_EDGE_EDGE / 2.0
                } else {
                    half_pill + LABEL_GAP + if lines.is_empty() { 0.0 } else { label_h }
                };
                let line_y = cursor + above;
                let start = lifelines[msg.from];
                let (path, label, pill) = if to_self {
                    let back = line_y + drop;
                    let out = start + SPACING_SELF_WIDTH;
                    let pill = numbered.then(|| {
                        let w = pill_width(number);
                        Rect {
                            x: out + SPACING_EDGE_EDGE / 2.0,
                            y: line_y + drop / 2.0 - SIGNAL_NUMBER / 2.0,
                            w,
                            h: SIGNAL_NUMBER,
                        }
                    });
                    let text_x = pill.map_or(out + SPACING_EDGE_EDGE / 2.0, |p| {
                        p.right() + SPACING_EDGE_EDGE / 2.0
                    });
                    let label = (!lines.is_empty()).then(|| Rect {
                        x: text_x,
                        y: line_y + drop / 2.0 - label_h / 2.0,
                        w: label_w,
                        h: label_h,
                    });
                    (
                        vec![
                            Point {
                                x: start,
                                y: line_y,
                            },
                            Point { x: out, y: line_y },
                            Point { x: out, y: back },
                            Point { x: start, y: back },
                        ],
                        label,
                        pill,
                    )
                } else {
                    let to = lifelines[msg.to];
                    let dir = (to - start).signum();
                    let tip = to - dir * ARROWHEAD_GAP;
                    let pill = numbered.then(|| {
                        let w = pill_width(number);
                        let back = if msg.refused {
                            mark_reach()
                        } else {
                            ARROWHEAD_LENGTH
                        } + SPACING_EDGE_EDGE / 2.0
                            + w / 2.0;
                        let cx = tip - dir * back;
                        Rect {
                            x: cx - w / 2.0,
                            y: line_y - SIGNAL_NUMBER / 2.0,
                            w,
                            h: SIGNAL_NUMBER,
                        }
                    });
                    let label = (!lines.is_empty()).then(|| Rect {
                        x: f64::midpoint(start, to) - label_w / 2.0,
                        y: line_y - half_pill - LABEL_GAP - label_h,
                        w: label_w,
                        h: label_h,
                    });
                    (
                        vec![
                            Point {
                                x: start,
                                y: line_y,
                            },
                            Point { x: to, y: line_y },
                        ],
                        label,
                        pill,
                    )
                };
                let below = if to_self {
                    drop + SPACING_EDGE_EDGE
                } else {
                    half_pill + SPACING_EDGE_EDGE
                };
                let row = if to_self {
                    SPACING_MESSAGE_ROW + drop
                } else {
                    SPACING_MESSAGE_ROW
                };
                cursor += row.max(above + below);
                for &f in &opened {
                    let frame = &mut frames[f];
                    for p in [msg.from, msg.to] {
                        if !frame.covered.contains(&p) {
                            frame.covered.push(p);
                        }
                    }
                    if to_self {
                        let end = label
                            .map_or(path[1].x, |l| l.right())
                            .max(pill.map_or(0.0, |p| p.right()));
                        frame.reach = frame.reach.max(end);
                    }
                }
                let refuser = if msg.sort == Sort::Reply {
                    msg.from
                } else {
                    msg.to
                };
                messages.push(Message {
                    from: msg.from,
                    to: msg.to,
                    sort: msg.sort,
                    refused: msg.refused,
                    number,
                    path,
                    label: label.map(|r| (lines, r)),
                    pill,
                    refuser,
                });
            }
            Step::Open(operator, when) => {
                let f = frames.len();
                for &outer in &opened {
                    frames[outer].inner.push(f);
                }
                let top = cursor;
                frames.push(Open {
                    operator,
                    top,
                    guards: when.map(|w| (guard_text(&w), top)).into_iter().collect(),
                    separators: Vec::new(),
                    covered: Vec::new(),
                    reach: f64::MIN,
                    inner: Vec::new(),
                    bottom: top,
                });
                opened.push(f);
                cursor = top + SPACING_FRAGMENT_TAG + LABEL_GAP;
            }
            Step::Operand(when) => {
                let f = *opened.last().expect("an operand inside a fragment");
                let at = cursor;
                frames[f].separators.push(at);
                cursor = at + LABEL_GAP;
                if let Some(w) = when {
                    frames[f].guards.push((guard_text(&w), cursor));
                    cursor += line + LABEL_GAP;
                }
            }
            Step::Close => {
                let f = opened.pop().expect("a fragment to close");
                cursor += SPACING_FRAGMENT_PADDING / 2.0;
                frames[f].bottom = cursor;
                cursor += LABEL_GAP;
            }
        }
    }
    let lifeline_end = cursor + SPACING_EDGE_EDGE / 2.0;

    // Across: each frame round its participants' lifelines, wider by a
    // padding for each frame nested inside it, and wide enough for its tag,
    // its guards and any message to itself inside it. Inner frames first,
    // so an outer one holds them.
    let tag_w = |op: Operator| text_width(op.name(), &TYPOGRAPHY_FRAME_LABEL) + 3.0 * TAG_PAD;
    let guard_w = |g: &str| text_width(g, &TYPOGRAPHY_SUBTITLE);
    let mut boxes: Vec<Rect> = vec![Rect::default(); frames.len()];
    let depth = |f: usize, frames: &[Open]| -> usize {
        fn d(f: usize, frames: &[Open]) -> usize {
            frames[f]
                .inner
                .iter()
                .map(|&i| 1 + d(i, frames))
                .max()
                .unwrap_or(0)
        }
        d(f, frames)
    };
    for f in (0..frames.len()).rev() {
        let fr = &frames[f];
        #[allow(clippy::cast_precision_loss)] // a few levels
        let pad = SPACING_FRAGMENT_PADDING * (1 + depth(f, &frames)) as f64;
        let xs: Vec<f64> = fr.covered.iter().map(|&p| lifelines[p]).collect();
        let lo = xs.iter().copied().fold(f64::INFINITY, f64::min);
        let hi = xs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let mut l = lo - pad;
        let mut r = (hi + pad).max(fr.reach + SPACING_FRAGMENT_PADDING);
        for &i in &fr.inner {
            l = l.min(boxes[i].x - SPACING_FRAGMENT_PADDING);
            r = r.max(boxes[i].right() + SPACING_FRAGMENT_PADDING);
        }
        let first = fr
            .guards
            .first()
            .filter(|(_, y)| (*y - fr.top).abs() < f64::EPSILON);
        let top_row =
            l + tag_w(fr.operator) + first.map_or(0.0, |(g, _)| SPACING_EDGE_EDGE + guard_w(g));
        r = r.max(top_row + SPACING_EDGE_EDGE);
        for (g, _) in &fr.guards {
            r = r.max(l + TAG_PAD + guard_w(g) + SPACING_EDGE_EDGE);
        }
        boxes[f] = Rect {
            x: l,
            y: fr.top,
            w: r - l,
            h: fr.bottom - fr.top,
        };
    }
    let fragments: Vec<Fragment> = frames
        .iter()
        .zip(&boxes)
        .map(|(fr, &frame)| {
            let tag = Rect {
                x: frame.x,
                y: frame.y,
                w: tag_w(fr.operator),
                h: SPACING_FRAGMENT_TAG,
            };
            let guards = fr
                .guards
                .iter()
                .map(|(g, y)| {
                    let on_tag = (*y - fr.top).abs() < f64::EPSILON;
                    let x = if on_tag {
                        tag.right() + SPACING_EDGE_EDGE
                    } else {
                        frame.x + TAG_PAD
                    };
                    let y = if on_tag {
                        tag.y + (tag.h - line) / 2.0
                    } else {
                        *y
                    };
                    (
                        g.clone(),
                        Rect {
                            x,
                            y,
                            w: guard_w(g),
                            h: line,
                        },
                    )
                })
                .collect();
            Fragment {
                operator: fr.operator,
                frame,
                tag,
                guards,
                separators: fr.separators.clone(),
            }
        })
        .collect();

    // Everything from the origin: the leftmost of heads, frames and labels
    // at 0.
    let lefts = heads
        .iter()
        .map(|h| h.x)
        .chain(boxes.iter().map(|b| b.x))
        .chain(
            messages
                .iter()
                .filter_map(|m| m.label.as_ref().map(|(_, r)| r.x)),
        );
    let shift = -lefts.fold(0.0, f64::min);
    let rights = heads
        .iter()
        .map(Rect::right)
        .chain(boxes.iter().map(Rect::right))
        .chain(
            messages
                .iter()
                .filter_map(|m| m.label.as_ref().map(|(_, r)| r.right())),
        )
        .chain(messages.iter().filter_map(|m| m.pill.map(|p| p.right())))
        .chain(lifelines.iter().zip(&self_reach).map(|(c, r)| c + r));
    let width = rights.fold(0.0, f64::max) + shift;
    let moved = |r: Rect| Rect {
        x: r.x + shift,
        ..r
    };
    let mut layout = Layout {
        heads: heads.into_iter().map(moved).collect(),
        lifelines: lifelines.iter().map(|c| c + shift).collect(),
        lifeline_span: (head_h, lifeline_end),
        messages: messages
            .into_iter()
            .map(|m| Message {
                path: m
                    .path
                    .iter()
                    .map(|p| Point {
                        x: p.x + shift,
                        y: p.y,
                    })
                    .collect(),
                label: m.label.map(|(l, r)| (l, moved(r))),
                pill: m.pill.map(moved),
                ..m
            })
            .collect(),
        fragments: fragments
            .into_iter()
            .map(|f| Fragment {
                frame: moved(f.frame),
                tag: moved(f.tag),
                guards: f.guards.into_iter().map(|(g, r)| (g, moved(r))).collect(),
                ..f
            })
            .collect(),
        legend: Vec::new(),
        credit: None,
        size: Size {
            w: width,
            h: lifeline_end,
        },
        spec,
    };
    let (entries, _, size) = legend::place(&layout.spec, layout.size);
    layout.legend = entries;
    layout.size = size;
    if seq.credit {
        let (at, size) = legend::credit(layout.size);
        layout.credit = Some(at);
        layout.size = size;
    }
    layout
}
