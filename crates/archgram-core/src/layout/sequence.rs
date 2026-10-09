//! Laying out a sequence diagram (docs/features/sequence.md; DESIGN.md,
//! Layout): the participants' heads in a row, each at the top of its band,
//! one row for each message in time's order, a frame round each fragment,
//! and in the `cards` look a bar while a participant answers a call. Every
//! position comes from the spec's order and the measured text, so the same
//! spec lays out the same everywhere.

use std::f64::consts::FRAC_1_SQRT_2;

use crate::font::text_width;
use crate::geometry::{Point, Rect, Size};
use crate::logos::Logos;
use crate::measure::{card_sizes_with, label_lines, lines_in, note_line, text_width_of};
use crate::render::refusal::mark_reach;
use crate::sequence::{Look, Operator, Sequence, SequenceStill, Step};
use crate::spec::{LogoPlace, Spec, Variant};
use crate::tokens::{
    ACTIVATION_OFFSET, ACTIVATION_WIDTH, ARROWHEAD_GAP, ARROWHEAD_LENGTH, AVATAR_LABEL_GAP,
    AVATAR_SIZE, CARD_LOGO_CHIP_RING, CARD_MULTI_OFFSET, LABEL_PILL_PAD_X, LABEL_PILL_PAD_Y,
    SPACING_BAND_GAP, SPACING_BAND_PAD, SPACING_EDGE_EDGE, SPACING_FRAGMENT_PADDING,
    SPACING_FRAGMENT_TAG, SPACING_FRAGMENT_TAG_PAD, SPACING_LABEL_GAP, SPACING_MESSAGE_ROW,
    SPACING_SELF_WIDTH, TYPOGRAPHY_FRAME_LABEL, TYPOGRAPHY_LEGEND, TYPOGRAPHY_SUBTITLE,
    TYPOGRAPHY_TITLE,
};

use super::legend;

pub use crate::sequence::Sort;

/// A message's label in its pill on the line: its number, then its words.
#[derive(Debug, Clone, PartialEq)]
pub struct Pill {
    pub rect: Rect,
    /// The number it leads with, when the still image numbers the messages.
    pub number: Option<u32>,
    /// The label's lines, one or two (two past `label.max-width`); none for
    /// a pill with its number alone.
    pub lines: Vec<String>,
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
    /// Its line, from where it leaves the sender's lifeline (or bar) to
    /// where it meets the receiver's: the drawing stops it
    /// `arrowhead.gap` short of its end.
    pub path: Vec<Point>,
    /// Its pill, when it has a label or a number to show.
    pub pill: Option<Pill>,
    /// The participant that turns the request away: who replies with a
    /// refusal, or who refuses a call.
    pub refuser: usize,
}

/// One fragment as laid out.
#[derive(Debug, Clone, PartialEq)]
pub struct Fragment {
    pub operator: Operator,
    pub frame: Rect,
    /// Its pills, each with its lines (two past `label.max-width`): the
    /// first holds its operator and first guard, on its top edge; each
    /// other a later operand's guard, on that operand's line.
    pub pills: Vec<(Vec<String>, Rect)>,
    /// Where each operand after the first starts: a dashed line across.
    pub separators: Vec<f64>,
}

/// A sequence laid out.
#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    /// The spec the heads are drawn from: the participants as nodes.
    pub spec: Spec,
    pub look: Look,
    /// Each head's footprint, in the participants' order.
    pub heads: Vec<Rect>,
    /// Each participant's band, its lifeline.
    pub bands: Vec<Rect>,
    /// The middle of each band, where messages start and end.
    pub lifelines: Vec<f64>,
    /// The bars on the lifelines, in the order their calls are made.
    pub activations: Vec<Rect>,
    pub messages: Vec<Message>,
    /// Outermost first, then in time's order.
    pub fragments: Vec<Fragment>,
    pub legend: Vec<legend::Entry>,
    pub credit: Option<Rect>,
    pub size: Size,
}

/// A line of `typography.subtitle`, a label's.
fn line_height() -> f64 {
    TYPOGRAPHY_SUBTITLE.size * TYPOGRAPHY_SUBTITLE.line_height
}

/// A line of `typography.frame-label`, a fragment pill's.
fn tag_line() -> f64 {
    TYPOGRAPHY_FRAME_LABEL.size * TYPOGRAPHY_FRAME_LABEL.line_height
}

/// The room between a pill and what is above or below it.
const LABEL_GAP: f64 = SPACING_LABEL_GAP;

/// How far apart, at least, two neighbouring bands are.
const BAND_GAP: f64 = SPACING_BAND_GAP;

/// The room inside a fragment's pill on either side of its words.
const TAG_PAD: f64 = SPACING_FRAGMENT_TAG_PAD;

/// A fragment's pill, its lines and its size: the operator and the first
/// guard, `alt · password matches`, or the operator alone; a later
/// operand's guard alone. The guard wraps as a label does, the operator
/// leading its first line; `spacing.fragment-tag` high for one line, a
/// line higher for two. None for an operand without a guard.
fn tag_pill(operator: Option<Operator>, when: Option<&str>) -> Option<(Vec<String>, Size)> {
    let mut lines: Vec<String> = when
        .map(|w| {
            lines_in(w.trim(), &TYPOGRAPHY_FRAME_LABEL)
                .into_iter()
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();
    match (operator, lines.first_mut()) {
        (Some(o), Some(first)) => *first = format!("{} · {first}", o.name()),
        (Some(o), None) => lines.push(o.name().to_owned()),
        (None, Some(_)) => {}
        (None, None) => return None,
    }
    let widest = lines
        .iter()
        .map(|l| text_width(l, &TYPOGRAPHY_FRAME_LABEL))
        .fold(0.0, f64::max);
    #[allow(clippy::cast_precision_loss)] // one or two lines
    let more = (lines.len().max(1) - 1) as f64;
    let size = Size {
        w: (widest + 2.0 * TAG_PAD).ceil(),
        h: SPACING_FRAGMENT_TAG + more * tag_line(),
    };
    Some((lines, size))
}

/// A message's pill: its number, `spacing.label-gap` before its words,
/// `label.pill-pad-x` in from its ends and `label.pill-pad-y` from its top
/// and bottom. None when there is neither a number nor a label to show.
fn message_pill(number: Option<u32>, label: Option<&str>) -> Option<(Vec<String>, Size)> {
    if number.is_none() && label.is_none() {
        return None;
    }
    let lines: Vec<String> = label
        .map(|l| label_lines(l).into_iter().map(str::to_owned).collect())
        .unwrap_or_default();
    let words = lines
        .iter()
        .map(|l| text_width(l, &TYPOGRAPHY_SUBTITLE))
        .fold(0.0, f64::max);
    let digits = number.map_or(0.0, |n| text_width(&n.to_string(), &TYPOGRAPHY_LEGEND));
    let gap = if number.is_some() && !lines.is_empty() {
        LABEL_GAP
    } else {
        0.0
    };
    #[allow(clippy::cast_precision_loss)] // one or two lines
    let rows = lines.len().max(1) as f64;
    let size = Size {
        w: (digits + gap + words + 2.0 * LABEL_PILL_PAD_X).ceil(),
        h: rows * line_height() + 2.0 * LABEL_PILL_PAD_Y,
    };
    Some((lines, size))
}

/// How far a message to itself reaches out from where it leaves: a refused
/// one further, so its ✕ sits on its way back, clear of the bend, before
/// its arrowhead (DESIGN.md, Components: Sequence, Refusal).
fn self_width(refused: bool) -> f64 {
    if refused {
        ARROWHEAD_GAP + mark_reach() + SPACING_MESSAGE_ROW / 4.0 + LABEL_GAP
    } else {
        SPACING_SELF_WIDTH
    }
}

/// The room a lifeline's bars may take to its right: a bar moved over one
/// other. Kept in `avatars` too, which draws no bars, so both looks set
/// the same columns.
const BAR_ROOM: f64 = ACTIVATION_WIDTH / 2.0 + ACTIVATION_OFFSET;

/// Each head's footprint and how far it reaches left of the lifeline, as
/// its look draws it: in `cards` a node card, every front card as wide as
/// the widest so the bands line up; in `avatars` the circle, the logo's
/// chip on its edge, and the name and note under it, centred on the
/// lifeline.
fn heads(look: Look, seq: &Sequence, spec: &Spec, logos: &dyn Logos) -> Vec<(Size, f64)> {
    let copies = |v: Variant| {
        if v == Variant::Multi {
            2.0 * CARD_MULTI_OFFSET
        } else {
            0.0
        }
    };
    match look {
        Look::Cards => {
            let sizes = card_sizes_with(spec, logos);
            let widest = seq
                .participants
                .iter()
                .zip(&sizes)
                .map(|(p, s)| s.w - copies(p.variant))
                .fold(0.0, f64::max);
            seq.participants
                .iter()
                .zip(&sizes)
                .map(|(p, s)| {
                    let size = Size {
                        w: widest + copies(p.variant),
                        h: s.h,
                    };
                    (size, widest / 2.0)
                })
                .collect()
        }
        Look::Avatars => spec
            .nodes
            .iter()
            .map(|n| {
                let radius = AVATAR_SIZE / 2.0;
                let chip = n
                    .tech
                    .as_deref()
                    .and_then(|t| logos.path(t))
                    .filter(|_| matches!(spec.logo, LogoPlace::Corner | LogoPlace::Chip))
                    .map_or(0.0, |_| radius * FRAC_1_SQRT_2 + CARD_LOGO_CHIP_RING / 2.0);
                let text = text_width_of(n, spec.logo, logos);
                let half = (radius + copies(n.variant))
                    .max(chip)
                    .max(text / 2.0)
                    .ceil();
                let note = if note_line(n, spec.logo, logos).text.is_some() {
                    line_height()
                } else {
                    0.0
                };
                let h = copies(n.variant)
                    + AVATAR_SIZE
                    + AVATAR_LABEL_GAP
                    + TYPOGRAPHY_TITLE.size * TYPOGRAPHY_TITLE.line_height
                    + note;
                (Size { w: 2.0 * half, h }, half)
            })
            .collect(),
    }
}

/// How far out from the bands it covers a fragment's frame reaches: a
/// padding, and that again for each level of fragments nested inside it.
fn padding(f: usize, inner: &[Vec<usize>]) -> f64 {
    fn depth(f: usize, inner: &[Vec<usize>]) -> usize {
        inner[f]
            .iter()
            .map(|&i| 1 + depth(i, inner))
            .max()
            .unwrap_or(0)
    }
    #[allow(clippy::cast_precision_loss)] // a few levels
    let levels = (1 + depth(f, inner)) as f64;
    SPACING_FRAGMENT_PADDING * levels
}

/// The width a frame needs for its pills: the widest, with a padding on
/// either side.
fn frame_width(pills: &[Size]) -> f64 {
    pills.iter().map(|p| p.w).fold(0.0, f64::max) + 2.0 * SPACING_FRAGMENT_PADDING
}

/// A fragment as the gaps between lifelines see it, before they are set.
struct Span {
    lo: usize,
    hi: usize,
    pills: Vec<Size>,
    inner: Vec<usize>,
    /// How many fragments it is nested in.
    within: usize,
    /// How far right of its rightmost lifeline a message to itself there
    /// reaches, its pill included.
    reach: f64,
}

/// What each fragment needs of the gaps between lifelines `half` a band
/// either side, so its frame covers only its own bands: on its left, its
/// padding; on its right, its padding, or a message to itself there, and a
/// padding for each frame it sits in; across its bands and the gap after
/// them, its pills. Each with half `spacing.band-gap` to spare before the
/// next band.
fn frame_needs(
    steps: &[Step<'_>],
    count: usize,
    numbered: bool,
    half: f64,
) -> Vec<(usize, usize, f64)> {
    let mut spans: Vec<Span> = Vec::new();
    let mut open: Vec<usize> = Vec::new();
    let mut number = 0u32;
    for step in steps {
        match step {
            Step::Message(m) => {
                number += 1;
                let reach = (m.from == m.to).then(|| {
                    let pill = message_pill(numbered.then_some(number), m.label)
                        .map_or(0.0, |(_, s)| LABEL_GAP + s.w);
                    BAR_ROOM + self_width(m.refused) + pill
                });
                for &f in &open {
                    let s = &mut spans[f];
                    for p in [m.from, m.to] {
                        if s.lo > s.hi {
                            s.lo = p;
                            s.hi = p;
                        }
                        s.lo = s.lo.min(p);
                        s.hi = s.hi.max(p);
                    }
                    if let Some(r) = reach {
                        // Kept per span's rightmost lifeline, checked below.
                        if m.from >= s.hi {
                            s.reach = s.reach.max(r);
                        }
                    }
                }
            }
            Step::Open(operator, when) => {
                let f = spans.len();
                for &o in &open {
                    spans[o].inner.push(f);
                }
                let pill = tag_pill(Some(*operator), *when).map(|(_, s)| s);
                spans.push(Span {
                    lo: usize::MAX,
                    hi: 0,
                    pills: pill.into_iter().collect(),
                    inner: Vec::new(),
                    within: open.len(),
                    reach: 0.0,
                });
                open.push(f);
            }
            Step::Operand(when) => {
                let f = *open.last().expect("an operand inside a fragment");
                if let Some((_, s)) = tag_pill(None, *when) {
                    spans[f].pills.push(s);
                }
            }
            Step::Close => {
                open.pop();
            }
        }
    }
    let inner: Vec<Vec<usize>> = spans.iter().map(|s| s.inner.clone()).collect();
    let mut needs = Vec::new();
    for (f, s) in spans.iter().enumerate() {
        if s.lo > s.hi {
            continue;
        }
        let pad = padding(f, &inner);
        #[allow(clippy::cast_precision_loss)] // a few levels
        let spare = BAND_GAP / 2.0 + SPACING_FRAGMENT_PADDING * s.within as f64;
        if s.lo > 0 {
            needs.push((s.lo - 1, s.lo, 2.0 * half + pad + BAND_GAP / 2.0));
        }
        if s.hi + 1 < count {
            let right = (half + pad).max(s.reach + SPACING_FRAGMENT_PADDING);
            needs.push((s.hi, s.hi + 1, right + half + spare));
            needs.push((s.lo, s.hi + 1, frame_width(&s.pills) - pad + spare));
        }
    }
    needs
}

/// A message down the page, before its line is placed across.
struct Row {
    /// Where its line leaves, and for a message to itself where it comes
    /// back (the same y for any other).
    y: f64,
    back: f64,
    pill: Option<(Vec<String>, Size)>,
}

/// A fragment while it is laid out: its rows, its participants, and the
/// fragments inside it.
struct Open {
    operator: Operator,
    top: f64,
    /// Each pill's lines, size and the y of the line it sits on.
    pills: Vec<(Vec<String>, Size, f64)>,
    separators: Vec<f64>,
    /// The participants its messages name.
    covered: Vec<usize>,
    /// The messages to themselves inside it, by their place in time.
    selves: Vec<usize>,
    /// The fragments inside it, by their index in the list.
    inner: Vec<usize>,
    /// Where it ends, once closed.
    bottom: f64,
}

/// A bar while a call is answered: on whose lifeline, from where to
/// where, and over how many other bars there.
type Bar = (usize, f64, f64, usize);

/// The bars of the `cards` look (UML 2.5.1, 17.2.4.4): for each call that
/// waits, on its receiver's lifeline, from where it arrives to the last
/// reply that answers it. A call never answered runs to the last message
/// the receiver sends before its caller calls it again, and at least half
/// a row; a call to itself never answered, half a row: what it does is
/// done in its loop.
/// Each with how many bars it is drawn over, so it is moved right by as
/// many offsets.
fn bars(steps: &[Step<'_>], rows: &[Row]) -> Vec<Bar> {
    let messages: Vec<_> = steps
        .iter()
        .filter_map(|s| match s {
            Step::Message(m) => Some(m),
            _ => None,
        })
        .collect();
    let mut out: Vec<Bar> = Vec::new();
    for (k, m) in messages.iter().enumerate() {
        if m.sort != Sort::Call || m.refused {
            continue;
        }
        let start = rows[k].back;
        let answered = messages
            .iter()
            .zip(rows)
            .filter(|(r, _)| r.answers == Some(k))
            .map(|(_, row)| row.y)
            .fold(f64::NEG_INFINITY, f64::max);
        let least = start + SPACING_MESSAGE_ROW / 2.0;
        let end = if answered.is_finite() {
            answered
        } else if m.from == m.to {
            least
        } else {
            messages
                .iter()
                .zip(rows)
                .skip(k + 1)
                .take_while(|(r, _)| !(r.from == m.from && r.to == m.to && r.sort == Sort::Call))
                .filter(|(r, _)| r.from == m.to)
                .map(|(_, row)| row.y)
                .fold(least, f64::max)
        };
        // Over every earlier bar on the same lifeline still drawn here.
        let depth = out
            .iter()
            .filter(|(p, s, e, _)| *p == m.to && *s <= start && start <= *e)
            .map(|(_, _, _, d)| d + 1)
            .max()
            .unwrap_or(0);
        out.push((m.to, start, end, depth));
    }
    out
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
    let look = seq.look;
    let measured = heads(look, seq, &spec, logos);
    let count = measured.len();
    let steps = seq.steps();
    let numbered = seq.still == SequenceStill::Numbers;

    // Every band as wide as the widest head of either look needs either
    // side of its lifeline, with `spacing.band-pad` beyond: both looks set
    // the same columns.
    let half = [Look::Cards, Look::Avatars]
        .into_iter()
        .flat_map(|l| heads(l, seq, &spec, logos))
        .map(|(s, left)| left.max(s.w - left))
        .fold(0.0, f64::max)
        + SPACING_BAND_PAD;

    // The room each message needs between its lifelines, and each message
    // to itself to the right of its own.
    let mut gaps: Vec<f64> = vec![2.0 * half + BAND_GAP; count.saturating_sub(1)];
    let mut needs: Vec<(usize, usize, f64)> = Vec::new();
    let mut number = 0u32;
    for step in &steps {
        let Step::Message(msg) = step else { continue };
        number += 1;
        let pill = message_pill(numbered.then_some(number), msg.label).map_or(0.0, |(_, s)| s.w);
        if msg.from == msg.to {
            let reach = BAR_ROOM + self_width(msg.refused) + LABEL_GAP + pill;
            if msg.from + 1 < count {
                needs.push((msg.from, msg.from + 1, reach + SPACING_EDGE_EDGE));
            }
            continue;
        }
        let tail = ARROWHEAD_GAP
            + BAR_ROOM
            + if msg.refused {
                mark_reach()
            } else {
                ARROWHEAD_LENGTH
            }
            + SPACING_EDGE_EDGE / 2.0;
        needs.push((
            msg.from.min(msg.to),
            msg.from.max(msg.to),
            pill + 2.0 * tail,
        ));
    }
    needs.extend(frame_needs(&steps, count, numbered, half));
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
    let lifelines: Vec<f64> = std::iter::once(half)
        .chain(gaps.iter().scan(half, |at, g| {
            *at += g;
            Some(*at)
        }))
        .take(count)
        .collect();

    // The heads in a row inside their bands, their feet on one line.
    let head_h = measured.iter().map(|(s, _)| s.h).fold(0.0, f64::max);
    let heads: Vec<Rect> = measured
        .iter()
        .zip(&lifelines)
        .map(|((s, left), &c)| Rect {
            x: c - left,
            y: SPACING_BAND_PAD + head_h - s.h,
            w: s.w,
            h: s.h,
        })
        .collect();

    // Down the page: a row for each message, a frame round each fragment.
    let row = SPACING_MESSAGE_ROW;
    let drop = row / 2.0;
    let mut cursor = SPACING_BAND_PAD + head_h + SPACING_BAND_PAD;
    let mut rows: Vec<Row> = Vec::new();
    let mut opened: Vec<usize> = Vec::new();
    let mut frames: Vec<Open> = Vec::new();
    for step in &steps {
        match step {
            Step::Message(msg) => {
                let k = rows.len();
                #[allow(clippy::cast_possible_truncation)] // a few hundred messages
                let number = k as u32 + 1;
                let pill = message_pill(numbered.then_some(number), msg.label);
                let pill_h = pill.as_ref().map_or(0.0, |(_, s)| s.h);
                let to_self = msg.from == msg.to;
                let down = if to_self { drop } else { 0.0 };
                let height = (row + down).max(pill_h + 2.0 * LABEL_GAP);
                let y = cursor + (height - down) / 2.0;
                rows.push(Row {
                    y,
                    back: y + down,
                    pill,
                });
                cursor += height;
                for &f in &opened {
                    let frame = &mut frames[f];
                    for p in [msg.from, msg.to] {
                        if !frame.covered.contains(&p) {
                            frame.covered.push(p);
                        }
                    }
                    if to_self {
                        frame.selves.push(k);
                    }
                }
            }
            Step::Open(operator, when) => {
                let f = frames.len();
                for &outer in &opened {
                    frames[outer].inner.push(f);
                }
                let (lines, size) =
                    tag_pill(Some(*operator), *when).expect("an operator has a pill");
                let top = cursor + size.h / 2.0;
                frames.push(Open {
                    operator: *operator,
                    top,
                    pills: vec![(lines, size, top)],
                    separators: Vec::new(),
                    covered: Vec::new(),
                    selves: Vec::new(),
                    inner: Vec::new(),
                    bottom: top,
                });
                opened.push(f);
                cursor = top + size.h / 2.0 + LABEL_GAP;
            }
            Step::Operand(when) => {
                let f = *opened.last().expect("an operand inside a fragment");
                if let Some((lines, size)) = tag_pill(None, *when) {
                    let at = cursor + size.h / 2.0;
                    frames[f].separators.push(at);
                    frames[f].pills.push((lines, size, at));
                    cursor = at + size.h / 2.0 + LABEL_GAP;
                } else {
                    frames[f].separators.push(cursor);
                    cursor += LABEL_GAP;
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
    let bottom = cursor + SPACING_BAND_PAD;

    // The bars, and where a message meets a lifeline: the middle of the
    // bar drawn there that is moved furthest, over the others.
    let bars = match look {
        Look::Cards => bars(&steps, &rows),
        Look::Avatars => Vec::new(),
    };
    #[allow(clippy::cast_precision_loss)] // a few bars deep
    let at_x = |p: usize, y: f64| {
        lifelines[p]
            + bars
                .iter()
                .filter(|(q, s, e, _)| *q == p && *s <= y && y <= *e)
                .map(|(_, _, _, d)| *d as f64 * ACTIVATION_OFFSET)
                .fold(0.0, f64::max)
    };
    #[allow(clippy::cast_precision_loss)] // a few bars deep
    let activations: Vec<Rect> = bars
        .iter()
        .map(|&(p, start, end, depth)| Rect {
            x: lifelines[p] + depth as f64 * ACTIVATION_OFFSET - ACTIVATION_WIDTH / 2.0,
            y: start,
            w: ACTIVATION_WIDTH,
            h: end - start,
        })
        .collect();

    // Across: each message's line and its pill.
    let mut messages = Vec::new();
    let mut self_right: Vec<f64> = Vec::new();
    let resolved = steps.iter().filter_map(|s| match s {
        Step::Message(m) => Some(m),
        _ => None,
    });
    for (k, (msg, r)) in resolved.zip(&rows).enumerate() {
        #[allow(clippy::cast_possible_truncation)] // a few hundred messages
        let number = k as u32 + 1;
        let start = at_x(msg.from, r.y);
        let to_self = msg.from == msg.to;
        let (path, rect) = if to_self {
            let out = start + self_width(msg.refused);
            let rect = r.pill.as_ref().map(|(_, s)| Rect {
                x: out + LABEL_GAP,
                y: f64::midpoint(r.y, r.back) - s.h / 2.0,
                w: s.w,
                h: s.h,
            });
            let path = vec![
                Point { x: start, y: r.y },
                Point { x: out, y: r.y },
                Point { x: out, y: r.back },
                Point {
                    x: at_x(msg.to, r.back),
                    y: r.back,
                },
            ];
            (path, rect)
        } else {
            let end = at_x(msg.to, r.y);
            let rect = r.pill.as_ref().map(|(_, s)| Rect {
                x: f64::midpoint(start, end) - s.w / 2.0,
                y: r.y - s.h / 2.0,
                w: s.w,
                h: s.h,
            });
            (
                vec![Point { x: start, y: r.y }, Point { x: end, y: r.y }],
                rect,
            )
        };
        self_right.push(if to_self {
            rect.map_or(path[1].x, |p| p.right())
        } else {
            f64::NEG_INFINITY
        });
        let pill = rect.zip(r.pill.as_ref()).map(|(rect, (lines, _))| Pill {
            rect,
            number: numbered.then_some(number),
            lines: lines.clone(),
        });
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
            pill,
            refuser,
        });
    }

    // Each frame round its participants' bands, wider by a padding for
    // each frame nested inside it, and wide enough for its pills and any
    // message to itself inside it. Inner frames first, so an outer one
    // holds them. The gaps were widened for all of it (`frame_needs`), so
    // no frame reaches a band it does not cover.
    let inner: Vec<Vec<usize>> = frames.iter().map(|fr| fr.inner.clone()).collect();
    let mut boxes: Vec<Rect> = vec![Rect::default(); frames.len()];
    for f in (0..frames.len()).rev() {
        let fr = &frames[f];
        let pad = padding(f, &inner);
        let xs: Vec<f64> = fr.covered.iter().map(|&p| lifelines[p]).collect();
        let lo = xs.iter().copied().fold(f64::INFINITY, f64::min);
        let hi = xs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let reach = fr
            .selves
            .iter()
            .map(|&k| self_right[k])
            .fold(f64::NEG_INFINITY, f64::max);
        let mut l = lo - half - pad;
        let mut r = (hi + half + pad).max(reach + SPACING_FRAGMENT_PADDING);
        for &i in &fr.inner {
            l = l.min(boxes[i].x - SPACING_FRAGMENT_PADDING);
            r = r.max(boxes[i].right() + SPACING_FRAGMENT_PADDING);
        }
        let sizes: Vec<Size> = fr.pills.iter().map(|(_, s, _)| *s).collect();
        r = r.max(l + frame_width(&sizes));
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
        .map(|(fr, &frame)| Fragment {
            operator: fr.operator,
            frame,
            pills: fr
                .pills
                .iter()
                .map(|(lines, s, y)| {
                    let x = match look {
                        Look::Cards => frame.x + SPACING_FRAGMENT_PADDING,
                        Look::Avatars => frame.centre_x() - s.w / 2.0,
                    };
                    (
                        lines.clone(),
                        Rect {
                            x,
                            y: y - s.h / 2.0,
                            w: s.w,
                            h: s.h,
                        },
                    )
                })
                .collect(),
            separators: fr.separators.clone(),
        })
        .collect();

    let bands: Vec<Rect> = lifelines
        .iter()
        .map(|&c| Rect {
            x: c - half,
            y: 0.0,
            w: 2.0 * half,
            h: bottom,
        })
        .collect();

    // Everything from the origin: the leftmost of bands, frames and pills
    // at 0.
    let pills: Vec<Rect> = messages
        .iter()
        .filter_map(|m| m.pill.as_ref().map(|p| p.rect))
        .collect();
    let lefts = bands.iter().chain(&boxes).chain(&pills).map(|r| r.x);
    let shift = -lefts.fold(0.0, f64::min);
    let rights = bands
        .iter()
        .chain(&boxes)
        .chain(&pills)
        .map(Rect::right)
        .chain(messages.iter().flat_map(|m| m.path.iter().map(|p| p.x)));
    let width = rights.fold(0.0, f64::max) + shift;
    let moved = |r: Rect| Rect {
        x: r.x + shift,
        ..r
    };
    let mut layout = Layout {
        look,
        heads: heads.into_iter().map(moved).collect(),
        bands: bands.into_iter().map(moved).collect(),
        lifelines: lifelines.iter().map(|c| c + shift).collect(),
        activations: activations.into_iter().map(moved).collect(),
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
                pill: m.pill.map(|p| Pill {
                    rect: moved(p.rect),
                    ..p
                }),
                ..m
            })
            .collect(),
        fragments: fragments
            .into_iter()
            .map(|f| Fragment {
                frame: moved(f.frame),
                pills: f.pills.into_iter().map(|(g, r)| (g, moved(r))).collect(),
                ..f
            })
            .collect(),
        legend: Vec::new(),
        credit: None,
        size: Size {
            w: width,
            h: bottom,
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
